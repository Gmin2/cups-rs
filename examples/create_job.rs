use cups_rs::{create_job, get_destination};
use std::error::Error as StdError;

fn main() -> Result<(), Box<dyn StdError>> {
    let printer_name = std::env::args().nth(1).unwrap_or_else(|| "PDF".to_string());

    println!("Creating job for printer: {}", printer_name);

    let destination = get_destination(&printer_name)?;
    println!("Found printer: {}", destination.full_name());

    // Create a test job
    let job = create_job(&destination, "Test Job from Rust")?;

    println!(" Job created successfully!");
    println!("   Job ID: {}", job.id);
    println!("   Destination: {}", job.dest_name);
    println!("   Title: {}", job.title);

    println!("\nNote: This job is created but no document has been submitted yet.");
    println!("You can check the job status with: lpstat -o");

    Ok(())
}
