use crate::model::{RecordingMode, Sample};
use libloading::Library;
use std::ffi::c_void;
use std::ptr;
use std::sync::{mpsc::Sender, Mutex, OnceLock};
use std::time::Instant;

// for phidget.dll
type Handle = *mut c_void;
type CreateFn = unsafe extern "system" fn(*mut Handle) -> i32;
type DeleteFn = unsafe extern "system" fn(*mut Handle) -> i32;
type CloseFn = unsafe extern "system" fn(Handle) -> i32;
type OpenFn = unsafe extern "system" fn(Handle, u32) -> i32;
type SetIntFn = unsafe extern "system" fn(Handle, i32) -> i32;
type GetRatioFn = unsafe extern "system" fn(Handle, *mut f64) -> i32;

// for callback
type VoltageCallback = unsafe extern "system" fn(Handle, *mut c_void, f64);
type SetCallbackFn = unsafe extern "system" fn(Handle, Option<VoltageCallback>, *mut c_void) -> i32;
type SetIntervalFn = unsafe extern "system" fn(Handle, u32) -> i32;
type SetTriggerFn = unsafe extern "system" fn(Handle, f64) -> i32;

// phidget22.dllからの関数読み込み用
fn load<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, String> {
    unsafe {
        lib.get::<T>(name)
            .map(|symbol| *symbol)
            .map_err(|e| e.to_string())
    }
}

// phidgetのエラーコード処理
fn check(code: i32, name: &str) -> Result<(), String> {
    if code == 0 {
        Ok(())
    } else {
        Err(format!("{} failed: code {}", name, code))
    }
}

// ブレ防止
fn apply_mode(value: f64, mode: RecordingMode) -> f64 {
    match mode {
        RecordingMode::Raw => value,

        RecordingMode::Round => (value * 100.0).round() / 100.0,

        RecordingMode::Truncate => (value * 100.0).trunc() / 100.0,
    }
}

// callback
static CALLBACK: Mutex<Option<(Sender<Sample>, Instant, RecordingMode)>> = Mutex::new(None);
static DLL_PIN: OnceLock<Library> = OnceLock::new();

unsafe extern "system" fn on_ratio(_handle: Handle, _ctx: *mut c_void, ratio: f64) {
    if let Ok(state) = CALLBACK.lock() {
        if let Some((tx, start, mode)) = state.as_ref() {
            let sample = Sample {
                time: start.elapsed().as_secs_f64(),
                voltage_ratio: apply_mode(ratio, *mode),
            };

            let _ = tx.send(sample);
        }
    }
}

// ========================
// 本体
//=========================

pub struct Recorder {
    _lib: Library,
    handle: Handle,

    close: CloseFn,
    delete: DeleteFn,
    get_ratio: GetRatioFn,
}

impl Recorder {
    pub fn open(hub_port: i32) -> Result<Self, String> {
        let lib = unsafe { Library::new("phidget22.dll") }.map_err(|e| e.to_string())?;

        // DLL内の関数を取得
        let create: CreateFn = load(&lib, b"PhidgetVoltageRatioInput_create")?;

        let delete: DeleteFn = load(&lib, b"PhidgetVoltageRatioInput_delete")?;

        let open: OpenFn = load(&lib, b"Phidget_openWaitForAttachment")?;

        let close: CloseFn = load(&lib, b"Phidget_close")?;

        let set_port: SetIntFn = load(&lib, b"Phidget_setHubPort")?;

        let set_hub_device: SetIntFn = load(&lib, b"Phidget_setIsHubPortDevice")?;

        let get_ratio: GetRatioFn = load(&lib, b"PhidgetVoltageRatioInput_getVoltageRatio")?;

        // ハンドル生成
        let mut handle: Handle = ptr::null_mut();
        check(unsafe { create(&mut handle) }, "create")?;

        // Recorderに所有権を移す
        let recorder = Self {
            _lib: lib,
            handle,
            close,
            delete,
            get_ratio,
        };

        // 接続先を設定
        check(unsafe { set_hub_device(handle, 1) }, "setIsHubPortDevice")?;

        check(unsafe { set_port(handle, hub_port) }, "setHubPort")?;

        // デバイス接続 (最大5秒待機)
        check(unsafe { open(handle, 5000) }, "openWaitForAttachment")?;

        Ok(recorder)
    }

    pub fn read(&self) -> Result<f64, String> {
        let mut value = 0.0;

        check(
            unsafe { (self.get_ratio)(self.handle, &mut value) },
            "getVoltageRatio",
        )?;

        Ok(value)
    }

    pub fn start_stream(&self, tx: Sender<Sample>, mode: RecordingMode) -> Result<(), String> {
        // DLLをプロセス終了時まで保持
        if DLL_PIN.get().is_none() {
            let pinned = unsafe { Library::new("phidget22.dll") }.map_err(|e| e.to_string())?;

            let _ = DLL_PIN.set(pinned);
        }

        let set_callback: SetCallbackFn = load(
            &self._lib,
            b"PhidgetVoltageRatioInput_setOnVoltageRatioChangeHandler",
        )?;

        let set_interval: SetIntervalFn =
            load(&self._lib, b"PhidgetVoltageRatioInput_setDataInterval")?;

        let set_trigger: SetTriggerFn = load(
            &self._lib,
            b"PhidgetVoltageRatioInput_setVoltageRatioChangeTrigger",
        )?;

        check(unsafe { set_interval(self.handle, 1) }, "setDataInterval")?;

        check(
            unsafe { set_trigger(self.handle, 0.0) },
            "setVoltageRatioChangeTrigger",
        )?;

        {
            let mut state = CALLBACK.lock().map_err(|e| e.to_string())?;

            if state.is_some() {
                return Err("Already recording".into());
            }

            *state = Some((tx, Instant::now(), mode));
        }

        let result = check(
            unsafe { set_callback(self.handle, Some(on_ratio), ptr::null_mut()) },
            "setOnVoltageRatioChangeHandler",
        );

        if result.is_err() {
            if let Ok(mut state) = CALLBACK.lock() {
                *state = None;
            }
        }

        result
    }

    pub fn stop_stream(&self) -> Result<(), String> {
        let set_callback: SetCallbackFn = load(
            &self._lib,
            b"PhidgetVoltageRatioInput_setOnVoltageRatioChangeHandler",
        )?;

        let result = check(
            unsafe { set_callback(self.handle, None, ptr::null_mut()) },
            "stop callback",
        );

        if let Ok(mut state) = CALLBACK.lock() {
            *state = None;
        }

        result
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.stop_stream();

        unsafe {
            (self.close)(self.handle);
            (self.delete)(&mut self.handle);
        }
    }
}
