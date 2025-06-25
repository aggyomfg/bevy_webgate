#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerStatus {
    /// Server is in the process of starting up
    Starting,
    /// Server is running and accepting connections
    Running,
    /// Server failed to start (e.g., port binding failed)
    Failed,
    /// Server is waiting to retry after a failed start attempt
    Retrying,
    /// Server keeps serving established connections but does not accept new ones
    Shutdown,
    /// Server is in the process of shutting down with timeout monitoring
    ShuttingDown,
    /// Server is completely stopped
    Stopped,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

impl ServerStatus {
    pub fn shutdown_requested(&self) -> bool {
        matches!(self, Self::Shutdown | Self::ShuttingDown)
    }

    pub(crate) fn can_start(&self) -> bool {
        matches!(self, Self::Stopped | Self::Retrying)
    }

    /// Check if status allows configuration changes
    pub fn can_reconfigure(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Starting => "Server is starting up",
            Self::Running => "Server is running and accepting connections",
            Self::Failed => "Server failed to start",
            Self::Retrying => "Server is waiting to retry startup",
            Self::Shutdown => "Server is shutting down gracefully",
            Self::ShuttingDown => "Server is in the process of shutting down",
            Self::Stopped => "Server is stopped",
        }
    }

    /// Check if this is a terminal state (no automatic transitions)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }
}
