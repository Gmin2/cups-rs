# cups_rs

A safe Rust wrapper for the Common UNIX Printing System (CUPS) API.

[![crates.io](https://img.shields.io/crates/v/cups_rs.svg)](https://crates.io/crates/cups_rs)
[![docs.rs](https://docs.rs/cups_rs/badge.svg)](https://docs.rs/cups_rs)

## Features

- Ergonomic Rust bindings to the CUPS printing system
- Safe abstractions for printer discovery and management
- Detailed printer capabilities and media handling
- Dynamic printer enumeration with callbacks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cups_rs = "0.1.0"
```

## Usage

- Discovering printers
```
use cups_rs::{get_all_destinations, get_default_destination, get_destination};

// Get all available printers
let printers = get_all_destinations()?;

// Get default printer
let default = get_default_destination()?;

// Get printer by name
let printer = get_destination("PrinterName")?;
```

- Printer Information
```
// Basic printer properties
let state = printer.state();              // PrinterState (Idle, Processing, Stopped, etc.)
let reasons = printer.state_reasons();    // Vec<String> of state reasons
let info = printer.info();                // Option<&String> description
let location = printer.location();        // Option<&String> location
let model = printer.make_and_model();     // Option<&String> make and model
let accepting = printer.is_accepting_jobs(); // bool
```

- Printer Capabilities
```
use std::ptr;
use cups_rs::{MEDIA, MEDIA_LETTER, MEDIA_FLAGS_DEFAULT};

// Get detailed printer information
let info = printer.get_detailed_info(ptr::null_mut())?;

// Check if options are supported
let supports_media = printer.is_option_supported(ptr::null_mut(), MEDIA);

// Check specific media support
let supports_letter = info.is_value_supported(
    ptr::null_mut(),
    printer.as_ptr(),
    MEDIA,
    MEDIA_LETTER
);

// Get default media size
let default_media = info.get_default_media(
    ptr::null_mut(),
    printer.as_ptr(),
    MEDIA_FLAGS_DEFAULT
)?;

// Get all supported media sizes
let media_sizes = info.get_all_media(
    ptr::null_mut(),
    printer.as_ptr(),
    MEDIA_FLAGS_DEFAULT
)?;
```

- Working with media sizes
```
// Access media properties
let width_inches = media.width_inches();
let length_inches = media.length_inches();
let printable_width = media.printable_width_inches();
let printable_height = media.printable_length_inches();

// Margins
let top = media.top_margin_inches();
let bottom = media.bottom_margin_inches();
let left = media.left_margin_inches();
let right = media.right_margin_inches();
```

## Examples

See the [examples](https://github.com/Gmin2/cups-rs/tree/main/examples) directory for more examples.

- `list_printers.rs`: Basic printer discovery
- `printer_options.rs`: Printer capabilities and media handling
- `media_sizes.rs`: Working with media sizes
- `enum_destinations.rs`: Dynamic printer enumeration

## License

MIT License