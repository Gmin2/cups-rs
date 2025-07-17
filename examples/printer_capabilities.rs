use cups_rs::*;
use std::ptr;

fn main() -> Result<()> {
    println!("CUPS Printer Capabilities Example");
    println!("=================================");

    let printer_name = std::env::args().nth(1).unwrap_or_else(|| "PDF".to_string());

    // Get printer and its detailed capabilities
    let destination = get_destination(&printer_name)?;
    let info = destination.get_detailed_info(ptr::null_mut())?;

    println!(
        "Printer: {} ({})",
        destination.full_name(),
        destination.state()
    );

    // Check support for standard print options
    let options = [
        (COPIES, "Multiple copies"),
        (MEDIA, "Media/paper size"),
        (SIDES, "Duplex printing"),
        (PRINT_COLOR_MODE, "Color mode"),
        (PRINT_QUALITY, "Print quality"),
    ];

    println!("\nSupported options:");
    for (option, description) in &options {
        let supported = destination.is_option_supported(ptr::null_mut(), option);
        println!(
            "  {}: {}",
            description,
            if supported { "Yes" } else { "No" }
        );
    }

    // Check specific media support
    let media_types = [
        (MEDIA_LETTER, "US Letter"),
        (MEDIA_A4, "A4"),
        (MEDIA_LEGAL, "Legal"),
    ];

    println!("\nMedia support:");
    for (media, name) in &media_types {
        let supported =
            info.is_value_supported(ptr::null_mut(), destination.as_ptr(), MEDIA, media);
        println!("  {}: {}", name, if supported { "Yes" } else { "No" });
    }

    // Get all available media sizes
    match info.get_all_media(ptr::null_mut(), destination.as_ptr(), MEDIA_FLAGS_DEFAULT) {
        Ok(sizes) => {
            println!("\nAvailable media ({} total):", sizes.len());
            for size in sizes.iter().take(5) {
                println!(
                    "  {} ({:.1}\" x {:.1}\")",
                    size.name,
                    size.width_inches(),
                    size.length_inches()
                );
            }
            if sizes.len() > 5 {
                println!("  ... and {} more", sizes.len() - 5);
            }
        }
        Err(e) => println!("Could not get media sizes: {}", e),
    }

    // Get default media with detailed margins
    if let Ok(default_media) =
        info.get_default_media(ptr::null_mut(), destination.as_ptr(), MEDIA_FLAGS_DEFAULT)
    {
        println!("\nDefault media: {}", default_media.name);
        println!(
            "  Printable area: {:.1}\" x {:.1}\"",
            default_media.printable_width_inches(),
            default_media.printable_length_inches()
        );
        println!(
            "  Margins: {:.1}\" (all sides)",
            default_media.top_margin_inches()
        );
    }

    Ok(())
}
