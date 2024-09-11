use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::TRACE => "trace".to_string(),
            LogLevel::DEBUG => "debug".to_string(),
            LogLevel::INFO => "info".to_string(),
            LogLevel::WARN => "warn".to_string(),
            LogLevel::ERROR => "error".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapturedLog {
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone)]
pub struct LoggingContainer {
    logs: Vec<CapturedLog>,
}

impl LoggingContainer {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn add_log(&mut self, message: String, level: LogLevel) {
        self.logs.push(CapturedLog { message, level });
    }

    pub fn has_log(&mut self, level: LogLevel) -> bool {
        self.logs.iter().any(|log| log.level == level)
    }

    pub fn has_matching_log(&mut self, level: LogLevel, pattern: &str) -> bool {
        let regex = Regex::new(pattern).unwrap();
        self.logs
            .iter()
            .any(|log| log.level == level && regex.is_match(&log.message))
    }

    pub fn reset(&mut self) {
        self.logs = Vec::new();
    }

    pub fn get_logs(&self) -> Vec<CapturedLog> {
        self.logs.clone()
    }
}
