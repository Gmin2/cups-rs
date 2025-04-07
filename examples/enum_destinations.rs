use cups_rs::{enum_destinations, Result};
use cups_rs::{DEST_FLAGS_NONE, DEST_FLAGS_REMOVED, PRINTER_LOCAL, PRINTER_REMOTE};

fn main() -> Result<()> {
    println!("=== CUPS Destination Enumeration Example ===\n");
    
    // Data structure to hold our enumeration results
    struct EnumResults {
        count: usize,
        local_count: usize,
        remote_count: usize,
        default_name: Option<String>,
    }
    
    let mut results = EnumResults {
        count: 0,
        local_count: 0,
        remote_count: 0,
        default_name: None,
    };
    
    // Cancel flag (can be set from another thread to cancel enumeration)
    let mut cancel = 0;
    
    // Enumerate all destinations
    println!("Enumerating all destinations...");
    enum_destinations(
        DEST_FLAGS_NONE,
        5000, // 5 second timeout
        Some(&mut cancel),
        0, // No type filter
        0, // No mask
        &mut |flags, dest, results: &mut EnumResults| {
            // Check if this is a removal notification
            if (flags & DEST_FLAGS_REMOVED) != 0 {
                println!("  Printer removed: {}", dest.full_name());
                return true; // Continue enumeration
            }
            
            // Print basic info about the destination
            println!("  Found printer: {}", dest.full_name());
            
            // Update counts
            results.count += 1;
            
            // Check printer type
            if let Some(type_str) = dest.get_option("printer-type") {
                if let Ok(printer_type) = type_str.parse::<u32>() {
                    if (printer_type & PRINTER_REMOTE) != 0 {
                        results.remote_count += 1;
                    } else {
                        results.local_count += 1;
                    }
                }
            }
            
            // Track default printer
            if dest.is_default {
                results.default_name = Some(dest.full_name());
            }
            
            true // Continue enumeration
        },
        &mut results
    )?;
    
    println!("\nEnumeration results:");
    println!("  Total printers: {}", results.count);
    println!("  Local printers: {}", results.local_count);
    println!("  Remote printers: {}", results.remote_count);
    
    if let Some(name) = results.default_name {
        println!("  Default printer: {}", name);
    } else {
        println!("  No default printer found");
    }
    
    // Demonstrate finding destinations with specific type
    println!("\nFinding only local printers:");
    enum_destinations(
        DEST_FLAGS_NONE,
        5000,
        None,
        PRINTER_LOCAL, // Type filter: only local printers
        PRINTER_REMOTE, // Mask: check remote bit
        &mut |_flags, dest, _: &mut ()| {
            println!("  Local printer: {}", dest.full_name());
            true // Continue enumeration
        },
        &mut ()
    )?;
    
    Ok(())
}