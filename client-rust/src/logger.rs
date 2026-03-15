use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

type Writer = BufWriter<std::fs::File>;

pub struct Logger {
    writer: Option<Writer>,
}

impl Logger {
    pub fn new(log_file: Option<PathBuf>) -> Result<Self, String> {
        let writer = if let Some(path) = log_file {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("failed to open log file {}: {e}", path.display()))?;
            Some(BufWriter::new(file))
        } else {
            None
        };

        Ok(Self { writer })
    }

    pub fn log_event<T: Serialize>(&mut self, event: &T) -> Result<(), String> {
        if let Some(writer) = self.writer.as_mut() {
            serde_json::to_writer(&mut *writer, event).map_err(|e| e.to_string())?;
            writer.write_all(b"\n").map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
