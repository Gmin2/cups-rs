use cups_rs::{
    get_all_destinations, get_default_destination, find_destinations, 
    Destinations, PRINTER_LOCAL, PRINTER_REMOTE, Result,
};

fn main() -> Result<()> {
    println!("CUPS Printer Discovery & Management Example");
    println!("===========================================");

    // Method 1: Simple discovery (existing API)
    let destinations = get_all_destinations()?;
    println!("Found {} printer(s)", destinations.len());

    if destinations.is_empty() {
        println!(
            "No printers found. Try: sudo lpadmin -p TestPDF -E -v file:///tmp/ -m everywhere"
        );
        return Ok(());
    }

    println!("\nBasic printer information:");
    for (i, dest) in destinations.iter().enumerate() {
        println!("\nPrinter #{}: {}", i + 1, dest.name);

        println!(
            "  State: {} | Accepting jobs: {}",
            dest.state(),
            dest.is_accepting_jobs()
        );

        if let Some(info) = dest.info() {
            println!("  Description: {}", info);
        }
        if let Some(location) = dest.location() {
            println!("  Location: {}", location);
        }

        let reasons = dest.state_reasons();
        if !reasons.is_empty() {
            println!("  Issues: {}", reasons.join(", "));
        }
    }

    // Method 2: Advanced management (new API)
    println!("\n--- Advanced Destination Management ---");
    let mut managed_destinations = Destinations::get_all()?;
    println!("Managing {} destinations with new API", managed_destinations.len());

    // Add a test destination to demonstrate management
    println!("Adding test destination 'MyTestPrinter'...");
    managed_destinations.add_destination("MyTestPrinter", None)?;

    // Search for it using new find method
    if let Some(found) = managed_destinations.find_destination("MyTestPrinter", None) {
        println!("  ✓ Found added destination: {}", found.name);
    }

    // Set a new default if we have printers
    if let Some(first_dest) = destinations.first() {
        println!("Setting '{}' as default...", first_dest.name);
        managed_destinations.set_default_destination(&first_dest.name, first_dest.instance.as_deref())?;
        println!("  ✓ Default updated");
    }

    // Clean up test destination
    let removed = managed_destinations.remove_destination("MyTestPrinter", None)?;
    println!("Test destination removed: {}", if removed { "✓" } else { "✗" });

    // Show current default (existing API)
    match get_default_destination() {
        Ok(default) => println!("\nCurrent default printer: {}", default.full_name()),
        Err(_) => println!("\nNo default printer set"),
    }

    // Filter by type (existing API)
    let local_printers = find_destinations(PRINTER_LOCAL, PRINTER_REMOTE)?;
    println!("Local printers found: {}", local_printers.len());

    // Note about persistence
    println!("\nNote: Use managed_destinations.save_to_lpoptions() to persist changes");

    Ok(())
}
