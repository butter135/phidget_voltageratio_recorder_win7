use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct Sample {
    time: f64,
    voltage_ratio: f64,
}

fn main() {
    
    let samples = Arc::new(Mutex::new(Vec::<Sample>::new()));
    
    let running = Arc::new(AtomicBool::new(true));
    
    let worker_samples = Arc::clone(&samples);
    let worker_running = Arc::clone(&running);
    
    let worker = thread::spawn(move || {
        let start = Instant::now();
        
        while worker_running.load(Ordering::Relaxed) {
            let time = start.elapsed().as_secs_f64();
            
            let voltage_ratio = 
                0.5 + 0.4 * (time * 2.0 * std::f64::consts::PI).sin();
            {
                let mut buffer = worker_samples.lock().unwrap();
                
                buffer.push(Sample {
                    time,
                    voltage_ratio,
                });
            }
            
            thread::sleep(Duration::from_millis(1));
        }
    });
    
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(500));
        
        let buffer = samples.lock().unwrap();
        
        if let Some(last) = buffer.last() {
            println!(
                "Samples: {}, Time: {:.3}, Voltage: {:.4}",
                buffer.len(),
                last.time,
                last.voltage_ratio
            );
        }
    }
    
    running.store(false, Ordering::Relaxed);
    
    worker.join().unwrap();
    
    println!(
        "Final samples: {}",
        samples.lock().unwrap().len()
    );
}