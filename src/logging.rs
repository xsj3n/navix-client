use std::fs::{OpenOptions};
use std::io::Write;

pub enum LogLevel {
    Info,
    Warn,
    Debug,
    Error
}

// need to signal if this fails to communicate with the server of the failure 
pub async fn log(level: LogLevel, message: &str) {
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./client.log")
        .unwrap();

    let level = match level {
        LogLevel::Info  => "INFO",
        LogLevel::Warn  => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Debug => "DEBUG"
    };

    let dt = chrono::Local::now().to_rfc3339();    
    match log_file.write_all(format!("{} {}: {}", dt, level, message).as_bytes()) {
        Ok(_) => (),
        Err(e) => panic!("Unable to write to log file: {}", e)
    }
}
