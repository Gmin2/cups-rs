use cups_rs::{
    create_job, get_destination, get_job_info, Result, FORMAT_TEXT,
};

fn main() -> Result<()> {
    println!("CUPS Multi-Document Job Example");

    let printer_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "PDF".to_string());

    let destination = get_destination(&printer_name)?;
    println!("Using printer: {}", destination.full_name());

    let job = create_job(&destination, "Multi-document job")?;
    println!("Created job ID: {}", job.id);

    std::fs::write("doc1.txt", "Document 1: First page of the multi-document job\n")?;
    std::fs::write("doc2.txt", "Document 2: Second page of the multi-document job\n")?;
    std::fs::write("doc3.txt", "Document 3: Final page of the multi-document job\n")?;

    println!("Submitting document 1 (not last)...");
    job.submit_file_with_options("doc1.txt", FORMAT_TEXT, &[], false)?;

    println!("Submitting document 2 (not last)...");
    let doc2_options = vec![
        ("print-quality".to_string(), "high".to_string()),
    ];
    job.submit_file_with_options("doc2.txt", FORMAT_TEXT, &doc2_options, false)?;

    println!("Submitting document 3 (last document)...");
    job.submit_file_with_options("doc3.txt", FORMAT_TEXT, &[], true)?;

    println!("All documents submitted. Closing job to start printing...");
    job.close()?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    match get_job_info(job.id) {
        Ok(info) => println!("Job status: {} ({} bytes)", info.status, info.size),
        Err(_) => println!("Job completed and removed from queue"),
    }

    std::fs::remove_file("doc1.txt").ok();
    std::fs::remove_file("doc2.txt").ok();
    std::fs::remove_file("doc3.txt").ok();

    println!("Multi-document job completed! Check: lpstat -o");
    Ok(())
}