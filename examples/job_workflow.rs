use cups_rs::*;

fn main() -> Result<()> {
    println!("CUPS Job Workflow Example");
    println!("========================");

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
        println!("Usage: cargo run --example job_workflow -- cancel <job_id>");
        return handle_list_command();
    }

    let job_id: i32 = args[2].parse()
        .map_err(|_| Error::JobManagementFailed("Invalid job ID".to_string()))?;

    println!("Canceling job {}", job_id);

    match get_job_info(job_id) {
        Ok(info) => {
            println!("Job: {} ({})", info.title, info.status);
            println!("Destination: {}", info.dest);
        }
        Err(_) => println!("Job not found or already completed"),
    }

    cancel_job(job_id)?;
    println!("Job {} canceled", job_id);

    Ok(())
}

fn handle_list_command() -> Result<()> {
    println!("All jobs:");
    let all_jobs = get_jobs(None)?;
    if all_jobs.is_empty() {
        println!("No jobs found");
    } else {
        for job in &all_jobs {
            println!("Job {}: {} on {} ({}, {} bytes)", 
                job.id, job.title, job.dest, job.status, job.size);
        }
    }

    println!("\nActive jobs:");
    let active_jobs = get_active_jobs(None)?;
    if active_jobs.is_empty() {
        println!("No active jobs");
    } else {
        for job in &active_jobs {
            println!("Job {}: {} ({})", job.id, job.title, job.status);
        }
    }

    println!("\nCompleted jobs:");
    let completed_jobs = get_completed_jobs(None)?;
    if completed_jobs.is_empty() {
        println!("No completed jobs");
    } else {
        for job in &completed_jobs {
            println!("Job {}: {} ({})", job.id, job.title, job.status);
        }
    }

    Ok(())
}

fn handle_print_workflow(args: &[String]) -> Result<()> {
    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("No file specified, creating test document");
        let content = "CUPS Rust Library Test\nTest document for job workflow\nPrint date: 2025-07-17";
        std::fs::write("test_workflow.txt", content)?;
        "test_workflow.txt".to_string()
    };

    let printer_name = if args.len() > 2 {
        args[2].clone()
    } else {
        "PDF".to_string()
    };

    println!("File: {}", file_path);
    println!("Printer: {}", printer_name);

    let destination = get_destination(&printer_name)?;
    println!("Found printer: {} ({})", destination.full_name(), destination.state());

    println!("\nStep 1: Create job");
    let job = create_job(&destination, &format!("Workflow test: {}", file_path))?;
    println!("Created job ID: {}", job.id);

    println!("\nStep 2: Submit document");
    job.submit_file(&file_path, FORMAT_TEXT)?;
    println!("Document submitted");

    println!("\nStep 3: Check job status (before close)");
    match get_job_info(job.id) {
        Ok(info) => {
            println!("Job status: {}", info.status);
            println!("Job size: {} bytes", info.size);
        }
        Err(e) => {
            println!("Could not get job info: {}", e);
            println!("Checking all job queues...");
            let active = get_active_jobs(None)?;
            let completed = get_completed_jobs(None)?;
            println!("Active jobs: {}, Completed jobs: {}", active.len(), completed.len());
        }
    }

    println!("\nStep 4: Close job (start printing)");
    job.close()?;
    println!("Job closed - printing started");

    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("\nStep 5: Final status check");
    match get_job_info(job.id) {
        Ok(info) => println!("Final status: {}", info.status),
        Err(e) => {
            println!("Could not get final status: {}", e);
            println!("Job likely completed and removed from queue");
        }
    }

    if file_path == "test_workflow.txt" {
        std::fs::remove_file("test_workflow.txt").ok();
    }

    println!("\nWorkflow completed successfully");
    println!("Verification:");
    println!("- Check: lpstat -o");
    println!("- PDF output: ~/PDF/ (if using PDF printer)");

    Ok(())
}