use crate::bindings;
use crate::error::Result;
use std::ffi::CStr;

/// Media size information from CUPS
#[derive(Debug, Clone)]
pub struct MediaSize {
    /// Media name (e.g., "na_letter_8.5x11in")
    pub name: String,
    /// Width in hundredths of millimeters
    pub width: i32,
    /// Length (height) in hundredths of millimeters
    pub length: i32,
    /// Bottom margin in hundredths of millimeters
    pub bottom: i32,
    /// Left margin in hundredths of millimeters
    pub left: i32,
    /// Right margin in hundredths of millimeters
    pub right: i32,
    /// Top margin in hundredths of millimeters
    pub top: i32,
}

impl MediaSize {
    /// Create a MediaSize from a CUPS cups_size_t structure
    pub(crate) unsafe fn from_cups_size(size: &bindings::cups_size_s) -> Result<Self> {
        let name = if size.media[0] == 0 {
            String::new()
        } else {
            unsafe {
                CStr::from_ptr(size.media.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            }
        };

        Ok(MediaSize {
            name,
            width: size.width,
            length: size.length,
            bottom: size.bottom,
            left: size.left,
            right: size.right,
            top: size.top,
        })
    }

    /// Width in inches
    pub fn width_inches(&self) -> f64 {
        self.width as f64 / 2540.0
    }

    /// Length (height) in inches
    pub fn length_inches(&self) -> f64 {
        self.length as f64 / 2540.0
    }

    /// Bottom margin in inches
    pub fn bottom_margin_inches(&self) -> f64 {
        self.bottom as f64 / 2540.0
    }

    /// Left margin in inches
    pub fn left_margin_inches(&self) -> f64 {
        self.left as f64 / 2540.0
    }

    /// Right margin in inches
    pub fn right_margin_inches(&self) -> f64 {
        self.right as f64 / 2540.0
    }

    /// Top margin in inches
    pub fn top_margin_inches(&self) -> f64 {
        self.top as f64 / 2540.0
    }

    /// Width in millimeters
    pub fn width_mm(&self) -> f64 {
        self.width as f64 / 100.0
    }

    /// Length (height) in millimeters
    pub fn length_mm(&self) -> f64 {
        self.length as f64 / 100.0
    }

    /// Bottom margin in millimeters
    pub fn bottom_margin_mm(&self) -> f64 {
        self.bottom as f64 / 100.0
    }

    /// Left margin in millimeters
    pub fn left_margin_mm(&self) -> f64 {
        self.left as f64 / 100.0
    }

    /// Right margin in millimeters
    pub fn right_margin_mm(&self) -> f64 {
        self.right as f64 / 100.0
    }

    /// Top margin in millimeters
    pub fn top_margin_mm(&self) -> f64 {
        self.top as f64 / 100.0
    }

    /// Printable width in hundredths of millimeters
    pub fn printable_width(&self) -> i32 {
        self.width - self.left - self.right
    }

    /// Printable length in hundredths of millimeters
    pub fn printable_length(&self) -> i32 {
        self.length - self.top - self.bottom
    }

    /// Printable width in inches
    pub fn printable_width_inches(&self) -> f64 {
        self.printable_width() as f64 / 2540.0
    }

    /// Printable length in inches
    pub fn printable_length_inches(&self) -> f64 {
        self.printable_length() as f64 / 2540.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_size_conversions() {
        let media = MediaSize {
            name: "na_letter_8.5x11in".to_string(),
            width: 21590,
            length: 27940,
            bottom: 635,
            left: 635,
            right: 635,
            top: 635,
        };

        // Test inch conversions
        assert!((media.width_inches() - 8.5).abs() < 0.01);
        assert!((media.length_inches() - 11.0).abs() < 0.01);
        assert!((media.bottom_margin_inches() - 0.25).abs() < 0.01);

        // Test millimeter conversions
        assert!((media.width_mm() - 215.9).abs() < 0.1);
        assert!((media.length_mm() - 279.4).abs() < 0.1);

        // Test printable area
        assert_eq!(media.printable_width(), 21590 - 635 - 635);
        assert_eq!(media.printable_length(), 27940 - 635 - 635);
    }
}