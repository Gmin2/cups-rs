use cups_rs::*;

pub fn setup_test_environment() {
    std::env::set_var("CUPS_SERVER", "localhost");
}

pub fn create_test_document() -> Vec<u8> {
    format!(
        "Test Document\n=============\n\nGenerated at: {}\n\nThis is a test document for CUPS integration testing.\n",
        chrono::Utc::now()
    ).into_bytes()
}

pub fn cleanup_test_jobs() {
    if let Ok(jobs) = get_active_jobs(None) {
        for job in jobs {
            if job.title.contains("Test") || job.title.contains("Integration") {
                println!("Cleaning up test job: {}", job.id);
                let _ = cancel_job(job.id);
            }
        }
    }
}