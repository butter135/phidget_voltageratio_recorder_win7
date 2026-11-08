#![windows_subsystem = "windows"]
use phidget_recorder_win7::model::{Command, UiEvent};
use phidget_recorder_win7::{gui, recording};

use std::sync::mpsc;
use std::thread;

fn main() {
    // GUI -> recorder
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

    // recorder -> GUI
    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();

    // 計測スレッド
    let worker = thread::spawn(move || {
        recording::run(cmd_rx, ui_tx);
    });

    // GUIはメインスレッドで
    gui::run(cmd_tx, ui_rx);

    // 終了待ち
    worker.join().unwrap();
}
