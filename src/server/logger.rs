use chrono::Local;
use std::fmt::Display;
use std::sync::OnceLock;

pub struct ServerLogger;

impl ServerLogger {
    pub fn new() -> Self {
        ServerLogger
    }

    fn get_time() -> String {
        Local::now().format("%H:%M:%S%.3f").to_string()
    }

    pub fn println(&self, str: &str) {
        println!(
            "{}",
            self.log_string(str, LogDomain::Server, LogLevel::Info)
        );
    }

    pub fn println_as(&self, str: &str, domain: LogDomain, level: LogLevel) {
        println!("{}", self.log_string(str, domain, level));
    }

    pub fn log_string(&self, str: &str, domain: LogDomain, level: LogLevel) -> String {
        format!(
            "{} [{}{}/{}]: {}",
            Self::get_time(),
            domain,
            Self::format_thread(),
            level,
            str
        )
    }

    fn format_thread() -> String {
        let t = std::thread::current();
        match t.name() {
            Some(str) => {
                if str != "main" {
                    format!(" thread-{}", t.id().as_u64())
                } else {
                    String::from(" main")
                }
            }
            _ => format!(" #{}", t.id().as_u64()),
        }
    }
}

pub enum LogDomain {
    Server,
    Network,
    LudiLoader,
    LudiGen,
}

impl Display for LogDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogDomain::Server => "Server",
                LogDomain::Network => "Network",
                LogDomain::LudiLoader => "Ludi-Load",
                LogDomain::LudiGen => "Ludi-Gen",
            }
        )
    }
}

pub enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Info => "\x1b[92mINFO\x1b[0m",
                LogLevel::Debug => "\x1b[94mDEBUG\x1b[0m",
                LogLevel::Warn => "\x1b[93mWARN\x1b[0m",
                LogLevel::Error => "\x1b[91mERROR\x1b[0m",
            }
        )
    }
}

pub static LOGGER: OnceLock<ServerLogger> = OnceLock::new();
