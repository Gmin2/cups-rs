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
        match state as u32 {
            crate::bindings::ipp_jstate_e_IPP_JSTATE_PENDING => JobStatus::Pending,
            crate::bindings::ipp_jstate_e_IPP_JSTATE_PROCESSING => JobStatus::Processing,
            crate::bindings::ipp_jstate_e_IPP_JSTATE_COMPLETED => JobStatus::Completed,
            crate::bindings::ipp_jstate_e_IPP_JSTATE_CANCELED => JobStatus::Canceled,
            crate::bindings::ipp_jstate_e_IPP_JSTATE_ABORTED => JobStatus::Aborted,
            crate::bindings::ipp_jstate_e_IPP_JSTATE_HELD => JobStatus::Held,
            crate::bindings::ipp_jstate_e_IPP_JSTATE_STOPPED => JobStatus::Stopped,
            _ => JobStatus::Unknown,
        }
    }

    pub fn to_cups_value(&self) -> i32 {
        match self {
            JobStatus::Pending => crate::bindings::ipp_jstate_e_IPP_JSTATE_PENDING as i32,
            JobStatus::Processing => crate::bindings::ipp_jstate_e_IPP_JSTATE_PROCESSING as i32,
            JobStatus::Completed => crate::bindings::ipp_jstate_e_IPP_JSTATE_COMPLETED as i32,
            JobStatus::Canceled => crate::bindings::ipp_jstate_e_IPP_JSTATE_CANCELED as i32,
            JobStatus::Aborted => crate::bindings::ipp_jstate_e_IPP_JSTATE_ABORTED as i32,
            JobStatus::Held => crate::bindings::ipp_jstate_e_IPP_JSTATE_HELD as i32,
            JobStatus::Stopped => crate::bindings::ipp_jstate_e_IPP_JSTATE_STOPPED as i32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_from_cups_state() {
        assert_eq!(JobStatus::from_cups_state(3), JobStatus::Pending);
        assert_eq!(JobStatus::from_cups_state(4), JobStatus::Processing);
        assert_eq!(JobStatus::from_cups_state(5), JobStatus::Completed);
        assert_eq!(JobStatus::from_cups_state(6), JobStatus::Canceled);
        assert_eq!(JobStatus::from_cups_state(7), JobStatus::Aborted);
        assert_eq!(JobStatus::from_cups_state(8), JobStatus::Held);
        assert_eq!(JobStatus::from_cups_state(9), JobStatus::Stopped);
        assert_eq!(JobStatus::from_cups_state(999), JobStatus::Unknown);
    }

    #[test]
    fn test_job_status_to_cups_value() {
        assert_eq!(JobStatus::Pending.to_cups_value(), 3);
        assert_eq!(JobStatus::Processing.to_cups_value(), 4);
        assert_eq!(JobStatus::Completed.to_cups_value(), 5);
        assert_eq!(JobStatus::Canceled.to_cups_value(), 6);
        assert_eq!(JobStatus::Aborted.to_cups_value(), 7);
        assert_eq!(JobStatus::Held.to_cups_value(), 8);
        assert_eq!(JobStatus::Stopped.to_cups_value(), 9);
        assert_eq!(JobStatus::Unknown.to_cups_value(), 0);
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "Pending");
        assert_eq!(JobStatus::Processing.to_string(), "Processing");
        assert_eq!(JobStatus::Completed.to_string(), "Completed");
        assert_eq!(JobStatus::Canceled.to_string(), "Canceled");
        assert_eq!(JobStatus::Aborted.to_string(), "Aborted");
        assert_eq!(JobStatus::Held.to_string(), "Held");
        assert_eq!(JobStatus::Stopped.to_string(), "Stopped");
        assert_eq!(JobStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_job_info_creation() {
        let job_info = JobInfo {
            id: 123,
            title: "Test Job".to_string(),
            user: "testuser".to_string(),
            dest: "TestPrinter".to_string(),
            status: JobStatus::Processing,
            size: 1024,
            priority: 50,
            creation_time: 1640995200,
            processing_time: 1640995260,
            completed_time: 0,
        };

        assert_eq!(job_info.id, 123);
        assert_eq!(job_info.title, "Test Job");
        assert_eq!(job_info.status, JobStatus::Processing);
    }
}
