use crate::model::Sample;

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn save_csv(path: &Path, samples: &[Sample]) -> io::Result<()> 
{
    let file = File::create(path)?;
    
    let mut writer = BufWriter::new(file);
    
    writeln!(writer, "index,callback_time_s,voltage_ratio")?;
    
    for (i, sample) in samples.iter().enumerate() {
        writeln!(writer, "{},{:.9},{:.9}", i, sample.time, sample.voltage_ratio)?;
    }
    
    writer.flush()?;
    
    Ok(())
}