// This logging module is intended to
//   - 


#[derive(PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Debug,
    Error
}

pub static LOG_FILEPATH: &'static str = "./client.log";

#[macro_export]
macro_rules! log {
    ($level:expr, $fmt:literal $(, $args:expr)* ) => {{
        log!(@inner $level, None::<i32>, $fmt $(, $args)*)
    }};
    ($level:expr, $fmt:literal $(, $args:expr)* ; $code:expr) => {{
        log!(@inner $level, Some($code), $fmt $(, $args)*)
    }};
    (@inner $level:expr, $code:expr, $fmt:literal $(, $args:expr)* ) => {{
        use std::io::Write;

        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILEPATH).unwrap();

        let level_str = match $level {
            LogLevel::Info  => "INFO",
            LogLevel::Warn  => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        };

        let dt = chrono::Local::now().to_rfc3339();
        let formatted = format!("{} {}: {}\n", dt, level_str, format!($fmt $(, $args)*));

        log_file.write_all(formatted.as_bytes()).unwrap();

        if matches!($level, LogLevel::Error) {
            std::process::exit($code.unwrap());
        }
        
    }};
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::command_to_string;
    #[tokio::test]
    async fn log_macro_test() -> () {
        let path = "./client.log";
        log!(LogLevel::Info, "{} {}", "hello", "world");
        log!(LogLevel::Info, "{} {} & panick!!", "hello", "world"; 10);

        let s = std::fs::read_to_string(path).unwrap();
        println!("{}", s);
        command_to_string(format!("rm {}", path), true).await.unwrap();
    
    }
}


