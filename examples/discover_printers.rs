use cups_rs::*;

fn main() -> Result<()> {
    println!("CUPS Printer Discovery Example");
    println!("=============================");

    // Get all printers from CUPS server
    let destinations = get_all_destinations()?;
    println!("Found {} printer(s)", destinations.len());

    if destinations.is_empty() {
        println!("No printers found. Try: sudo lpadmin -p TestPDF -E -v file:///tmp/ -m everywhere");
        return Ok(());
    }

    println!("\nBasic printer information:");
    for (i, dest) in destinations.iter().enumerate() {
        println!("\nPrinter #{}: {}", i + 1, dest.name);
        
        // Check printer state and job acceptance
        println!("  State: {} | Accepting jobs: {}", dest.state(), dest.is_accepting_jobs());
        
        // Get optional printer attributes
        if let Some(info) = dest.info() {
            println!("  Description: {}", info);
        }
        if let Some(location) = dest.location() {
            println!("  Location: {}", location);
        }
        
        // Check for any issues
        let reasons = dest.state_reasons();
        if !reasons.is_empty() {
            println!("  Issues: {}", reasons.join(", "));
        }
    }

    // Find the system default printer
    match get_default_destination() {
        Ok(default) => println!("\nDefault printer: {}", default.full_name()),
        Err(_) => println!("\nNo default printer set"),
    }

    // Filter printers by type (local vs network)
    let local_printers = find_destinations(PRINTER_LOCAL, PRINTER_REMOTE)?;
    println!("Local printers found: {}", local_printers.len());

    Ok(())
}