
# cups_rs

A safe Rust wrapper for the Common UNIX Printing System (CUPS) API.

[![crates.io](https://img.shields.io/crates/v/cups_rs.svg)](https://crates.io/crates/cups_rs)
[![docs.rs](https://docs.rs/cups_rs/badge.svg)](https://docs.rs/cups_rs)

## Features

- **Safe CUPS Integration**: Memory-safe Rust bindings to the CUPS printing system
- **Printer Discovery**: Enumerate and filter available printers with callbacks
- **Comprehensive Printer Information**: Access printer capabilities, media support, and status
- **Job Management**: Create, submit, monitor, and cancel print jobs
- **Print Options**: Type-safe configuration for copies, quality, color mode, duplex, and media
- **Media Handling**: Query supported paper sizes with detailed margin information
- **Error Handling**: Comprehensive error types with recovery suggestions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cups_rs = "0.1.0"
```

### System Requirements

CUPS development libraries must be installed:

**Ubuntu/Debian:**
```bash
sudo apt-get install libcups2-dev
```

**RHEL/CentOS/Fedora:**
```bash
sudo dnf install cups-devel
# or: sudo yum install cups-devel
```

**macOS:**
```bash
# CUPS is included with macOS
```

## Usage

### Discovering Printers

```rust
use cups_rs::*;

// Get all available printers
let printers = get_all_destinations()?;
println!("Found {} printer(s)", printers.len());

// Get default printer
let default = get_default_destination()?;
println!("Default: {}", default.full_name());

// Get specific printer by name
let printer = get_destination("PDF")?;
```

### Printer Information and Status

```rust
// Check printer state and capabilities
println!("State: {}", printer.state());
println!("Accepting jobs: {}", printer.is_accepting_jobs());

if let Some(info) = printer.info() {
    println!("Description: {}", info);
}

if let Some(location) = printer.location() {
    println!("Location: {}", location);
}

// Check for issues
let reasons = printer.state_reasons();
if !reasons.is_empty() {
    println!("Issues: {}", reasons.join(", "));
}
```

### Printer Capabilities and Media

```rust
use std::ptr;

// Get detailed printer capabilities
let info = printer.get_detailed_info(ptr::null_mut())?;

// Check option support
let supports_duplex = printer.is_option_supported(ptr::null_mut(), SIDES);
let supports_color = printer.is_option_supported(ptr::null_mut(), PRINT_COLOR_MODE);

// Check media support
let supports_a4 = info.is_value_supported(
    ptr::null_mut(),
    printer.as_ptr(),
    MEDIA,
    MEDIA_A4
);

// Get available media sizes
let media_sizes = info.get_all_media(
    ptr::null_mut(),
    printer.as_ptr(),
    MEDIA_FLAGS_DEFAULT
)?;

for media in &media_sizes {
    println!("{}: {:.1}\" × {:.1}\"", 
        media.name, 
        media.width_inches(), 
        media.length_inches()
    );
}
```

### Creating and Managing Print Jobs

```rust
// Create a simple print job
let job = create_job(&printer, "My Document")?;
println!("Created job ID: {}", job.id);

// Submit a document
job.submit_file("document.pdf", FORMAT_PDF)?;

// Close job to start printing
job.close()?;
```

### Print Options and Configuration

```rust
// Create job with specific options
let options = PrintOptions::new()
    .copies(2)
    .color_mode(ColorMode::Color)
    .quality(PrintQuality::High)
    .duplex(DuplexMode::TwoSidedPortrait)
    .media(MEDIA_A4)
    .orientation(Orientation::Landscape);

let job = create_job_with_options(&printer, "Configured Print", &options)?;

// Submit document with custom format
job.submit_file("presentation.pdf", FORMAT_PDF)?;
job.close()?;
```

### Job Monitoring and Management

```rust
// Get job information
let job_info = get_job_info(job.id)?;
println!("Status: {} | Size: {} bytes", job_info.status, job_info.size);

// List active jobs
let active_jobs = get_active_jobs(None)?;
for job in &active_jobs {
    println!("Job {}: {} ({})", job.id, job.title, job.status);
}

// Cancel a specific job
cancel_job(job.id)?;

// Or cancel via job instance
job.cancel()?;
```

### Advanced Printer Discovery

```rust
// Find printers with specific capabilities
let color_printers = find_destinations(PRINTER_COLOR, PRINTER_BW)?;
let local_printers = find_destinations(PRINTER_LOCAL, PRINTER_REMOTE)?;

// Use callback-based enumeration for real-time updates
enum_destinations(
    DEST_FLAGS_NONE,
    5000, // 5 second timeout
    None,
    PRINTER_LOCAL,
    PRINTER_REMOTE,
    &mut |flags, dest, user_data: &mut Vec<String>| {
        if (flags & DEST_FLAGS_REMOVED) == 0 {
            user_data.push(dest.full_name());
            println!("Found: {}", dest.full_name());
        }
        true // Continue enumeration
    },
    &mut printer_names,
)?;
```

### Error Handling

```rust
use cups_rs::{Error, ErrorCategory};

match create_job(&printer, "Test") {
    Ok(job) => println!("Job created: {}", job.id),
    Err(e) => {
        println!("Error: {}", e);
        println!("Category: {:?}", e.error_category());
        println!("Suggestion: {}", e.suggested_action());
        
        if e.is_recoverable() {
            println!("This error may be temporary - consider retrying");
        }
    }
}
```

### Media Size Details

```rust
// Get default media with detailed information
let default_media = info.get_default_media(
    ptr::null_mut(),
    printer.as_ptr(),
    MEDIA_FLAGS_DEFAULT
)?;

println!("Default media: {}", default_media.name);
println!("Size: {:.1}\" × {:.1}\"", 
    default_media.width_inches(), 
    default_media.length_inches()
);
println!("Printable area: {:.1}\" × {:.1}\"",
    default_media.printable_width_inches(),
    default_media.printable_length_inches()
);
println!("Margins: T:{:.1}\" B:{:.1}\" L:{:.1}\" R:{:.1}\"",
    default_media.top_margin_inches(),
    default_media.bottom_margin_inches(),
    default_media.left_margin_inches(),
    default_media.right_margin_inches()
);
```

## Examples

The [examples](examples/) directory contains complete working examples:

- [`discover_printers.rs`](examples/discover_printers.rs): Basic printer discovery and information
- [`printer_capabilities.rs`](examples/printer_capabilities.rs): Exploring printer features and media support  
- [`print_with_options.rs`](examples/print_with_options.rs): Advanced printing with various options
- [`complete_workflow.rs`](examples/complete_workflow.rs): Full job lifecycle management

Run examples with:
```bash
cargo run --example discover_printers
cargo run --example printer_capabilities -- PDF
cargo run --example complete_workflow -- document.pdf MyPrinter
```

## Supported Print Options

| Option | Type | Values |
|--------|------|---------|
| `copies` | `u32` | Number of copies |
| `media` | `&str` | `MEDIA_A4`, `MEDIA_LETTER`, `MEDIA_LEGAL`, etc. |
| `color_mode` | `ColorMode` | `Auto`, `Color`, `Monochrome` |
| `quality` | `PrintQuality` | `Draft`, `Normal`, `High` |
| `duplex` | `DuplexMode` | `OneSided`, `TwoSidedPortrait`, `TwoSidedLandscape` |
| `orientation` | `Orientation` | `Portrait`, `Landscape` |

## Supported Document Formats

- **PDF**: `FORMAT_PDF` (`application/pdf`)
- **PostScript**: `FORMAT_POSTSCRIPT` (`application/postscript`) 
- **Plain Text**: `FORMAT_TEXT` (`text/plain`)
- **JPEG Images**: `FORMAT_JPEG` (`image/jpeg`)

## Error Types

The library provides detailed error information:

- `DestinationNotFound`: Printer not available
- `JobCreationFailed`: Cannot create print job
- `PrinterNotAccepting`: Printer rejecting jobs
- `AuthenticationRequired`: Credentials needed
- `DocumentTooLarge`: File size limits exceeded
- `NetworkError`: CUPS server communication issues

## Thread Safety

CUPS operations are not thread-safe by default. For multi-threaded applications, consider:

- Using a single thread for all CUPS operations
- Implementing proper synchronization around CUPS calls
- Creating separate connections per thread where possible

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Requirements

- Rust 1.70+
- CUPS 2.0+ development libraries
- Linux, macOS, or other UNIX-like system with CUPS support
