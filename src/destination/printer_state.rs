/// Represents the operational state of a printer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrinterState {
    /// Printer is idle and available for printing
    Idle,
    /// Printer is processing a job
    Processing,
    /// Printer is stopped and unavailable for printing
    Stopped,
    /// State is unknown or undefined
    Unknown,
}

impl PrinterState {
    /// Create a PrinterState from the CUPS state value
    pub fn from_cups_state(state: &str) -> Self {
        match state {
            "3" => PrinterState::Idle,
            "4" => PrinterState::Processing,
            "5" => PrinterState::Stopped,
            _ => PrinterState::Unknown,
        }
    }

    /// Returns true if the printer is available for printing
    pub fn is_available(&self) -> bool {
        matches!(self, PrinterState::Idle | PrinterState::Processing)
    }

    /// Get the raw CUPS state value as a string
    pub fn to_cups_value(&self) -> &'static str {
        match self {
            PrinterState::Idle => "3",
            PrinterState::Processing => "4",
            PrinterState::Stopped => "5",
            PrinterState::Unknown => "0",
        }
    }
}

impl std::fmt::Display for PrinterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrinterState::Idle => write!(f, "Idle"),
            PrinterState::Processing => write!(f, "Processing"),
            PrinterState::Stopped => write!(f, "Stopped"),
            PrinterState::Unknown => write!(f, "Unknown"),
        }
    }
}
