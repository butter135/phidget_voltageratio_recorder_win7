use phidget_recorder_win7::model::RecordingMode;
use phidget_recorder_win7::model::Sample;
use phidget_recorder_win7::phidget::Recorder;
use phidget_recorder_win7::storage;

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), String> {
    println!("Connecting...");

    let recorder = Recorder::open(0)?;

    println!("Connected!");

    // 通信チャンネルの作成
    let (tx, rx) = mpsc::channel::<Sample>();

    // データ保存配列
    let mut samples = Vec::<Sample>::with_capacity(300_000);

    //計測開始
    recorder.start_stream(tx, RecordingMode::Raw)?;

    // 5sec計測
    for second in 1..=5 {
        thread::sleep(Duration::from_secs(1));
        samples.extend(rx.try_iter());

        if let Some(last) = samples.last() {
            println!(
                "{}s | Samples: {} | VoltageRatio: {:.5}",
                second,
                samples.len(),
                last.voltage_ratio
            );
        }
    }

    recorder.stop_stream()?;

    samples.extend(rx.try_iter());

    println!("Total samples: {}", samples.len());

    if samples.len() >= 2 {
        let first = samples.first().unwrap();
        let last = samples.last().unwrap();

        let elapsed = last.time - first.time;

        let frequency = (samples.len() - 1) as f64 / elapsed;

        println!("Duration: {:.6} s", elapsed);
        println!("Frequency: {:.3} Hz", frequency);

        println!("Saveing csv...");

        let path = Path::new("recording.csv");

        storage::save_csv(path, &samples).map_err(|e| e.to_string())?;

        println!("Saved: {}", path.display());
    }

    Ok(())
}
