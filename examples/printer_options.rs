use cups_rs::{get_destination, Result};
use cups_rs::{COPIES, MEDIA, SIDES, PRINT_QUALITY, PRINT_COLOR_MODE, ORIENTATION};
use cups_rs::{MEDIA_LETTER, MEDIA_LEGAL, MEDIA_A4, MEDIA_FLAGS_DEFAULT};
use std::error::Error as StdError;
use std::ptr;

fn main() -> Result<(), Box<dyn StdError>> {
    println!("=== CUPS Printer Options Example ===\n");
    
    // Get name of printer from command line or use default
    let printer_name = std::env::args().nth(1)
        .unwrap_or_else(|| "PDF".to_string()); // Default to "PDF" if no argument
    
    println!("Getting capabilities for printer: {}", printer_name);
    
    // Get the destination
    let destination = get_destination(&printer_name)?;
    println!("Found printer: {}", destination.full_name());
    
    // Get detailed information
    println!("\nGetting detailed information...");
    let info = destination.get_detailed_info(ptr::null_mut())?;
    
    // Check if various standard options are supported
    println!("\nChecking standard options:");
    for option in [
        COPIES,
        MEDIA,
        SIDES,
        PRINT_QUALITY,
        PRINT_COLOR_MODE,
        ORIENTATION,
    ] {
        let supported = destination.is_option_supported(ptr::null_mut(), option);
        println!("  {}: {}", option, if supported { "Supported" } else { "Not supported" });
    }
    
    // Check if standard media types are supported
    println!("\nChecking standard media types:");
    let media_check = [
        (MEDIA_LETTER, "US Letter"),
        (MEDIA_LEGAL, "US Legal"),
        (MEDIA_A4, "A4"),
    ];
    
    for (media, name) in &media_check {
        let supported = info.is_value_supported(
            ptr::null_mut(),
            destination.as_ptr(),
            MEDIA,
            media
        );
        println!("  {}: {}", name, if supported { "Supported" } else { "Not supported" });
    }
    
    // Get media sizes
    println!("\nAvailable media sizes:");
    match info.get_all_media(ptr::null_mut(), destination.as_ptr(), MEDIA_FLAGS_DEFAULT) {
        Ok(sizes) => {
            for (i, size) in sizes.iter().enumerate() {
                println!("  {}. {} ({:.2}\" x {:.2}\")",
                    i + 1,
                    size.name,
                    size.width_inches(),
                    size.length_inches()
                );
                
                // Try to get localized name
                match info.localize_media(
                    ptr::null_mut(),
                    destination.as_ptr(),
                    MEDIA_FLAGS_DEFAULT,
                    size
                ) {
                    Ok(localized) => println!("     Localized: {}", localized),
                    Err(_) => {}
                }
            }
        },
        Err(e) => println!("  Error retrieving media sizes: {}", e),
    }
    
    Ok(())
}