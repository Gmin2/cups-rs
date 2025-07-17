use cups_rs::*;

fn main() -> Result<()> {
    println!("CUPS Print with Options Example");
    println!("===============================");

    let args: Vec<String> = std::env::args().collect();
    
    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        let content = "Print Options Test\n\nTesting various print settings:\n- Copies\n- Quality\n- Color mode\n- Duplex\n";
        std::fs::write("options_test.txt", content)?;
        "options_test.txt".to_string()
    };

    let printer_name = args.get(2).cloned().unwrap_or_else(|| "PDF".to_string());
    let destination = get_destination(&printer_name)?;

    // Test different option combinations
    let test_cases = vec![
        ("Basic print", PrintOptions::new()),
        
        ("Multiple copies", 
         PrintOptions::new().copies(2)),
        
        ("High quality color", 
         PrintOptions::new()
             .color_mode(ColorMode::Color)
             .quality(PrintQuality::High)),
        
        ("Duplex on A4", 
         PrintOptions::new()
             .duplex(DuplexMode::TwoSidedPortrait)
             .media(MEDIA_A4)),
        
        ("Custom settings", 
         PrintOptions::new()
             .copies(3)
             .color_mode(ColorMode::Monochrome)
             .quality(PrintQuality::Draft)
             .orientation(Orientation::Landscape)),
    ];

    for (name, options) in test_cases {
        println!("\n--- {} ---", name);
        
        // Show options being used
        if !options.is_empty() {
            println!("Options:");
            for (key, value) in options.as_cups_options() {
                println!("  {}: {}", key, value);
            }
        }

        // Create job with specific options
        match create_job_with_options(&destination, &format!("{} test", name), &options) {
            Ok(job) => {
                println!("Created job {}", job.id);
                
                // Submit and start printing
                if job.submit_file(&file_path, FORMAT_TEXT).is_ok() {
                    job.close().ok();
                    println!("Job submitted and started");
                }
            }
            Err(e) => println!("Failed: {}", e),
        }
    }

    if file_path == "options_test.txt" {
        std::fs::remove_file("options_test.txt").ok();
    }

    println!("\nAll test jobs submitted. Check: lpstat -o");
    Ok(())
}