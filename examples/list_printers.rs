use cups_rs::{get_all_destinations, get_default_destination};
use std::error::Error as StdError;

fn main() -> Result<(), Box<dyn StdError>> {
    println!("=== CUPS Printer List Example ===\n");

    // Get all destinations
    println!("Getting all destinations...");
    let destinations = get_all_destinations()?;
    println!("Found {} destination(s)\n", destinations.len());

    // Display each destination
    for (i, dest) in destinations.iter().enumerate() {
        println!("Destination #{}", i + 1);
        println!("  Name: {}", dest.name);

        if let Some(instance) = &dest.instance {
            println!("  Instance: {}", instance);
        }

        println!(
            "  Is Default: {}",
            if dest.is_default { "Yes" } else { "No" }
        );
        println!("  Number of options: {}", dest.options.len());

        // Print key options
        for opt_name in &[
            "printer-info",
            "printer-location",
            "printer-make-and-model",
            "printer-state",
            "printer-state-reasons",
        ] {
            if let Some(value) = dest.get_option(opt_name) {
                println!("  {}: {}", opt_name, value);
            }
        }

        // Get printer state
        println!("  State: {}", dest.state());

        // Get state reasons
        let reasons = dest.state_reasons();
        if !reasons.is_empty() {
            println!("  State reasons: {}", reasons.join(", "));
        } else {
            println!("  State reasons: none");
        }

        println!(
            "  Accepting jobs: {}",
            if dest.is_accepting_jobs() {
                "Yes"
            } else {
                "No"
            }
        );

        if let Some(uri) = dest.uri() {
            println!("  URI: {}", uri);
        }

        if let Some(device) = dest.device_uri() {
            println!("  Device URI: {}", device);
        }

        println!();
    }

    // Test getting the default destination
    println!("=== Testing Default Destination ===");
    match get_default_destination() {
        Ok(default) => {
            println!("Default printer: {}", default.full_name());
            if let Some(info) = default.info() {
                println!("Info: {}", info);
            }
            if let Some(location) = default.location() {
                println!("Location: {}", location);
            }
            if let Some(model) = default.make_and_model() {
                println!("Make & Model: {}", model);
            }
        }
        Err(e) => {
            println!("No default printer found: {}", e);
        }
    }

    Ok(())
}
