use cups_rs::{get_destination, MediaSize, Result};
use cups_rs::{MEDIA_LETTER, MEDIA_FLAGS_DEFAULT, MEDIA_FLAGS_DUPLEX, MEDIA_FLAGS_READY};
use std::ptr;

fn main() -> Result<()> {
    println!("=== CUPS Media Sizes Example ===\n");
    
    // Get name of printer from command line or use default
    let printer_name = std::env::args().nth(1)
        .unwrap_or_else(|| "PDF".to_string()); // Default to "PDF" if no argument
    
    println!("Getting media information for printer: {}", printer_name);
    
    // Get the destination
    let destination = get_destination(&printer_name)?;
    
    // Get detailed information
    let info = destination.get_detailed_info(ptr::null_mut())?;
    
    // Get default media size
    println!("\nDefault media size:");
    match info.get_default_media(
        ptr::null_mut(),
        destination.as_ptr(),
        MEDIA_FLAGS_DEFAULT
    ) {
        Ok(media) => print_media_details(&media),
        Err(e) => println!("  Error getting default media: {}", e),
    }
    
    // Check specific media sizes
    println!("\nUS Letter details:");
    match info.get_media_by_name(
        ptr::null_mut(),
        destination.as_ptr(),
        MEDIA_LETTER,
        MEDIA_FLAGS_DEFAULT
    ) {
        Ok(media) => print_media_details(&media),
        Err(e) => println!("  Error: {}", e),
    }
    
    // Try to get duplex media details
    println!("\nUS Letter duplex details:");
    match info.get_media_by_name(
        ptr::null_mut(),
        destination.as_ptr(),
        MEDIA_LETTER,
        MEDIA_FLAGS_DUPLEX
    ) {
        Ok(media) => print_media_details(&media),
        Err(e) => println!("  Error: {}", e),
    }
    
    // List all media
    println!("\nAll supported media:");
    match info.get_all_media(
        ptr::null_mut(),
        destination.as_ptr(),
        MEDIA_FLAGS_DEFAULT
    ) {
        Ok(media_list) => {
            for (i, media) in media_list.iter().enumerate() {
                println!("  {}. {}", i + 1, media.name);
                println!("     Size: {:.2}\" x {:.2}\"", 
                    media.width_inches(),
                    media.length_inches()
                );
            }
        },
        Err(e) => println!("  Error getting media list: {}", e),
    }
    
    // Try to get ready media (what's actually loaded)
    println!("\nReady media (actually loaded):");
    match info.get_all_media(
        ptr::null_mut(),
        destination.as_ptr(),
        MEDIA_FLAGS_READY
    ) {
        Ok(media_list) => {
            if media_list.is_empty() {
                println!("  No ready media reported by printer");
            } else {
                for (i, media) in media_list.iter().enumerate() {
                    println!("  {}. {}", i + 1, media.name);
                    println!("     Size: {:.2}\" x {:.2}\"", 
                        media.width_inches(),
                        media.length_inches()
                    );
                }
            }
        },
        Err(e) => println!("  Error getting ready media list: {}", e),
    }
    
    Ok(())
}

// Helper function to print details about a media size
fn print_media_details(media: &MediaSize) {
    println!("  Name: {}", media.name);
    println!("  Size: {:.2}\" x {:.2}\" ({:.2}mm x {:.2}mm)",
        media.width_inches(), media.length_inches(),
        media.width_mm(), media.length_mm());
    println!("  Printable area: {:.2}\" x {:.2}\"",
        media.printable_width_inches(), media.printable_length_inches());
    println!("  Margins:");
    println!("    Top: {:.2}\"", media.top_margin_inches());
    println!("    Left: {:.2}\"", media.left_margin_inches());
    println!("    Right: {:.2}\"", media.right_margin_inches());
    println!("    Bottom: {:.2}\"", media.bottom_margin_inches());
}