use cups_rs::*;

fn main() -> Result<()> {
    println!("CUPS Complete Workflow Example");
    println!("==============================");

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "cancel" {
        return handle_cancel_command(&args);
    }

    if args.len() > 1 && args[1] == "list" {
        return handle_list_command();
    }

    handle_print_workflow(&args)
}

fn handle_cancel_command(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        println!("Usage: cargo run --example complete_workflow -- cancel <job_id>");
        return handle_list_command();
    }

    let job_id: i32 = args[2].parse()
        .map_err(|_| Error::JobManagementFailed("Invalid job ID".to_string()))?;

    // Get job info before canceling
    match get_job_info(job_id) {
        Ok(info) => println!("Canceling: {} ({})", info.title, info.status),
        Err(_) => println!("Job {} not found", job_id),
    }

    // Cancel the job
    cancel_job(job_id)?;
    println!("Job {} canceled", job_id);

    Ok(())
}

fn handle_list_command() -> Result<()> {
    // Get different job queues
    let active_jobs = get_active_jobs(None)?;
    let completed_jobs = get_completed_jobs(None)?;

    println!("Active jobs: {}", active_jobs.len());
    for job in &active_jobs {
        println!("  Job {}: {} ({})", job.id, job.title, job.status);
    }

    println!("Recent completed jobs: {}", completed_jobs.len());
    for job in completed_jobs.iter().take(5) {
        println!("  Job {}: {} ({})", job.id, job.title, job.status);
    }

    Ok(())
}

fn handle_print_workflow(args: &[String]) -> Result<()> {
    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        let content = "CUPS Workflow Test\nThis demonstrates job creation, document submission, and completion.\n";
        std::fs::write("workflow_test.txt", content)?;
        "workflow_test.txt".to_string()
    };

    let printer_name = args.get(2).cloned().unwrap_or_else(|| "PDF".to_string());

    // Get the target printer
    let destination = get_destination(&printer_name)?;
    println!("Using printer: {} ({})", destination.full_name(), destination.state());

    // Step 1: Create a print job
    let job = create_job(&destination, "Workflow test document")?;
    println!("Created job ID: {}", job.id);

    // Step 2: Submit document to the job
    let format = if file_path.ends_with(".pdf") { FORMAT_PDF } else { FORMAT_TEXT };
    job.submit_file(&file_path, format)?;
    println!("Document submitted");

    // Step 3: Check job status
    if let Ok(info) = get_job_info(job.id) {
        println!("Job status: {} ({} bytes)", info.status, info.size);
    }

    // Step 4: Close job to start printing
    job.close()?;
    println!("Job closed - printing started");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Step 5: Final status check
    match get_job_info(job.id) {
        Ok(info) => println!("Final status: {}", info.status),
        Err(_) => println!("Job completed and removed from queue"),
    }

    if file_path == "workflow_test.txt" {
        std::fs::remove_file("workflow_test.txt").ok();
    }

    println!("Workflow completed! Check: lpstat -o");
    Ok(())
}