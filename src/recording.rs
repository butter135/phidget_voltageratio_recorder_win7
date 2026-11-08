use crate::model::{Command, RecordingMode, Sample, UiEvent};
use crate::phidget::Recorder;
use crate::storage;

use chrono::Local;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

pub fn run(cmd_rx: Receiver<Command>, ui_tx: Sender<UiEvent>) {
    while let Ok(command) = cmd_rx.recv() {
        match command {
            Command::Start(mode) => {
                let recorder = match Recorder::open(0) {
                    Ok(recorder) => recorder,

                    Err(error) => {
                        let _ = ui_tx.send(UiEvent::Error(error));
                        continue;
                    }
                };

                let (sample_tx, sample_rx) = mpsc::channel::<Sample>();

                if let Err(error) = recorder.start_stream(sample_tx, mode) {
                    let _ = ui_tx.send(UiEvent::Error(error));
                    continue;
                }

                let started_at = Local::now().format("%Y%m%d_%H%M%S").to_string();

                let mut samples = Vec::<Sample>::with_capacity(300_000);

                let _ = ui_tx.send(UiEvent::Started);

                let mut last_report = Instant::now();

                let mut last_sent = 0;

                let exit_requested = loop {
                    // 受信したサンプルを配列に
                    samples.extend(sample_rx.try_iter());

                    // 100msごとにGUIへ最新状態を通知
                    if last_report.elapsed() >= Duration::from_millis(100) {
                        if let Some(last) = samples.last() {
                            let new_samples = samples[last_sent..].to_vec();
                            last_sent = samples.len();

                            let _ = ui_tx.send(UiEvent::Progress {
                                count: samples.len(),
                                ratio: last.voltage_ratio,
                                new_samples,
                            });
                        }

                        last_report = Instant::now();
                    }

                    // GUIからの命令を待つ
                    match cmd_rx.recv_timeout(Duration::from_millis(10)) {
                        Ok(Command::Stop) => break false,

                        Ok(Command::Exit) => break true,

                        Ok(Command::Start(_)) => {}

                        Err(RecvTimeoutError::Timeout) => {}

                        Err(RecvTimeoutError::Disconnected) => break true,
                    }
                };

                // コールバックを停止
                let stop_result = recorder.stop_stream();

                // チャンネルに残っているサンプルを回収
                samples.extend(sample_rx.try_iter());

                // Recorderを解放
                drop(recorder);

                if let Err(error) = stop_result {
                    let _ = ui_tx.send(UiEvent::Error(error));
                }

                let _ = ui_tx.send(UiEvent::Stopped);

                // save file
                let mode_name = match mode {
                    RecordingMode::Truncate => "truncate",
                    RecordingMode::Round => "round",
                    RecordingMode::Raw => "raw",
                };

                let output_dir = Path::new("output");

                let filename = format!("{}_{}.csv", started_at, mode_name);

                let output_path = output_dir.join(&filename);

                // outputディレクトリを作成（既にあってもOK）
                let result = fs::create_dir_all(output_dir)
                    .and_then(|_| storage::save_csv(&output_path, &samples));

                match result {
                    Ok(()) => {
                        let _ = ui_tx.send(UiEvent::Saved(output_path.display().to_string()));
                    }

                    Err(e) => {
                        let _ = ui_tx.send(UiEvent::Error(e.to_string()));
                    }
                }

                if exit_requested {
                    break;
                }
            }

            Command::Stop => {}

            Command::Exit => break,
        }
    }
}
