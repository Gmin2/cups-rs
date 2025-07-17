use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Canceled,
    Aborted,
    Held,
    Stopped,
    Unknown,
}

impl JobStatus {
    pub fn from_cups_state(state: i32) -> Self {
        match state {
            3 => JobStatus::Pending,
            4 => JobStatus::Processing,
            5 => JobStatus::Completed,
            6 => JobStatus::Canceled,
            7 => JobStatus::Aborted,
            8 => JobStatus::Held,
            9 => JobStatus::Stopped,
            _ => JobStatus::Unknown,
        }
    }

    pub fn to_cups_value(&self) -> i32 {
        match self {
            JobStatus::Pending => 3,
            JobStatus::Processing => 4,
            JobStatus::Completed => 5,
            JobStatus::Canceled => 6,
            JobStatus::Aborted => 7,
            JobStatus::Held => 8,
            JobStatus::Stopped => 9,
            JobStatus::Unknown => 0,
        }
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Processing => write!(f, "Processing"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Canceled => write!(f, "Canceled"),
            JobStatus::Aborted => write!(f, "Aborted"),
            JobStatus::Held => write!(f, "Held"),
            JobStatus::Stopped => write!(f, "Stopped"),
            JobStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub id: i32,
    pub title: String,
    pub user: String,
    pub dest: String,
    pub status: JobStatus,
    pub size: i32,
    pub priority: i32,
    pub creation_time: i64,
    pub processing_time: i64,
    pub completed_time: i64,
}