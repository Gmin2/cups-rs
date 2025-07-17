use cups_rs::*;

fn main() -> Result<()> {
    println!("CUPS Print with Options Example");
    println!("==============================");

    let args: Vec<String> = std::env::args().collect();
    
    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        let content = "Print Options Test Document\n\nThis document tests various print options:\n- Multiple copies\n- Color settings\n- Quality settings\n- Duplex printing\n";
        std::fs::write("options_test.txt", content)?;
        "options_test.txt".to_string()
    };

    let printer_name = args.get(2).cloned().unwrap_or_else(|| "PDF".to_string());

    println!("File: {}", file_path);
    println!("Printer: {}", printer_name);

    let destination = get_destination(&printer_name)?;
    println!("Found printer: {} ({})", destination.full_name(), destination.state());

    println!("\nTesting different print options...");

    let test_cases = vec![
        ("Basic print", PrintOptions::new()),
        
        ("2 copies", PrintOptions::new().copies(2)),
        
        ("Color + High quality", 
         PrintOptions::new()
             .color_mode(ColorMode::Color)
             .quality(PrintQuality::High)),
        
        ("Duplex + A4", 
         PrintOptions::new()
             .duplex(DuplexMode::TwoSidedPortrait)
             .media(MEDIA_A4)),
        
        ("All options", 
         PrintOptions::new()
             .copies(3)
             .color_mode(ColorMode::Monochrome)
             .quality(PrintQuality::Draft)
             .duplex(DuplexMode::OneSided)
             .orientation(Orientation::Portrait)
             .media(MEDIA_LETTER)),
    ];

    for (name, options) in test_cases {
        println!("\n--- {} ---", name);
        
        if !options.is_empty() {
            println!("Options:");
            for (key, value) in options.as_cups_options() {
                println!("  {}: {}", key, value);
            }
        }

        match create_job_with_options(&destination, &format!("{} - {}", name, file_path), &options) {
            Ok(job) => {
                println!("Created job ID: {}", job.id);
                
                match job.submit_file(&file_path, FORMAT_TEXT) {
                    Ok(_) => {
                        println!("Document submitted");
                        
                        match job.close() {
                            Ok(_) => println!("Job closed - printing started"),
                            Err(e) => println!("Failed to close job: {}", e),
                        }
                    }
                    Err(e) => println!("Failed to submit document: {}", e),
                }
            }
            Err(e) => println!("Failed to create job: {}", e),
        }
    }

    if file_path == "options_test.txt" {
        std::fs::remove_file("options_test.txt").ok();
    }

    println!("\nAll test jobs submitted");
    println!("Check: lpstat -o");

    Ok(())
}