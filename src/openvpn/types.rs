use chrono::Local;

#[derive(Debug, Clone, PartialEq)]
pub enum VpnState {
    Disconnected,
    Spawning,
    Connecting,
    Authenticating,
    GettingConfig,
    AssigningIp,
    AddingRoutes,
    Connected,
    Reconnecting(String),
    Disconnecting,
    Exiting(String),
    Error(String),
}

impl VpnState {
    pub fn label(&self) -> &str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Spawning => "Spawning...",
            Self::Connecting => "Connecting...",
            Self::Authenticating => "Authenticating...",
            Self::GettingConfig => "Getting config...",
            Self::AssigningIp => "Assigning IP...",
            Self::AddingRoutes => "Adding routes...",
            Self::Connected => "Connected",
            Self::Reconnecting(_) => "Reconnecting...",
            Self::Disconnecting => "Disconnecting...",
            Self::Exiting(_) => "Exiting...",
            Self::Error(_) => "Error",
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Disconnected | Self::Error(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionInfo {
    pub local_ip: Option<String>,
    pub remote_ip: Option<String>,
    pub remote_port: Option<u16>,
    pub connected_since: Option<chrono::DateTime<Local>>,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            local_ip: None,
            remote_ip: None,
            remote_port: None,
            connected_since: None,
            bytes_in: 0,
            bytes_out: 0,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AuthRequest {
    UserPass,
    UserPassWithChallenge { prompt: String, echo: bool },
    PrivateKey,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
    Fatal,
}

impl LogLevel {
    pub fn prefix(&self) -> &str {
        match self {
            Self::Info => "I",
            Self::Warning => "W",
            Self::Error => "E",
            Self::Debug => "D",
            Self::Fatal => "F",
        }
    }
}
