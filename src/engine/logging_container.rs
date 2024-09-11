use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum LOG_LEVEL {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl ToString for LOG_LEVEL {
    fn to_string(&self) -> String {
        match self {
            LOG_LEVEL::TRACE => "trace".to_string(),
            LOG_LEVEL::DEBUG => "debug".to_string(),
            LOG_LEVEL::INFO => "info".to_string(),
            LOG_LEVEL::WARN => "warn".to_string(),
            LOG_LEVEL::ERROR => "error".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapturedLog {
    pub message: String,
    pub level: LOG_LEVEL,
}

#[derive(Debug, Clone)]
pub struct LoggingContainer {
    logs: Vec<CapturedLog>,
}

impl LoggingContainer {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn add_log(&mut self, message: String, level: LOG_LEVEL) {
        self.logs.push(CapturedLog { message, level });
    }

    pub fn has_log(&mut self, level: LOG_LEVEL) -> bool {
        self.logs.iter().any(|log| log.level == level)
    }

    pub fn has_matching_log(&mut self, level: LOG_LEVEL, pattern: &str) -> bool {
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
