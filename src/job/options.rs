use crate::constants::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PrintOptions {
    options: HashMap<String, String>,
}

impl PrintOptions {
    pub fn new() -> Self {
        Self {
            options: HashMap::new(),
        }
    }

    pub fn copies(mut self, count: u32) -> Self {
        self.options.insert(COPIES.to_string(), count.to_string());
        self
    }

    pub fn media(mut self, media: &str) -> Self {
        self.options.insert(MEDIA.to_string(), media.to_string());
        self
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.options
            .insert(PRINT_COLOR_MODE.to_string(), mode.to_string());
        self
    }

    pub fn quality(mut self, quality: PrintQuality) -> Self {
        self.options
            .insert(PRINT_QUALITY.to_string(), quality.to_string());
        self
    }

    pub fn duplex(mut self, duplex: DuplexMode) -> Self {
        self.options.insert(SIDES.to_string(), duplex.to_string());
        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.options
            .insert(ORIENTATION.to_string(), orientation.to_string());
        self
    }

    pub fn custom_option<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    pub fn as_cups_options(&self) -> Vec<(&str, &str)> {
        self.options
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    Auto,
    Color,
    Monochrome,
}

impl ToString for ColorMode {
    fn to_string(&self) -> String {
        match self {
            ColorMode::Auto => PRINT_COLOR_MODE_AUTO.to_string(),
            ColorMode::Color => PRINT_COLOR_MODE_COLOR.to_string(),
            ColorMode::Monochrome => PRINT_COLOR_MODE_MONOCHROME.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PrintQuality {
    Draft,
    Normal,
    High,
}

impl ToString for PrintQuality {
    fn to_string(&self) -> String {
        match self {
            PrintQuality::Draft => PRINT_QUALITY_DRAFT.to_string(),
            PrintQuality::Normal => PRINT_QUALITY_NORMAL.to_string(),
            PrintQuality::High => PRINT_QUALITY_HIGH.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DuplexMode {
    OneSided,
    TwoSidedPortrait,
    TwoSidedLandscape,
}

impl ToString for DuplexMode {
    fn to_string(&self) -> String {
        match self {
            DuplexMode::OneSided => SIDES_ONE_SIDED.to_string(),
            DuplexMode::TwoSidedPortrait => SIDES_TWO_SIDED_PORTRAIT.to_string(),
            DuplexMode::TwoSidedLandscape => SIDES_TWO_SIDED_LANDSCAPE.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl ToString for Orientation {
    fn to_string(&self) -> String {
        match self {
            Orientation::Portrait => ORIENTATION_PORTRAIT.to_string(),
            Orientation::Landscape => ORIENTATION_LANDSCAPE.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_options_builder() {
        let options = PrintOptions::new()
            .copies(3)
            .media(crate::MEDIA_A4)
            .color_mode(ColorMode::Color)
            .quality(PrintQuality::High)
            .duplex(DuplexMode::TwoSidedPortrait)
            .orientation(Orientation::Landscape);

        let cups_options = options.as_cups_options();
        assert_eq!(cups_options.len(), 6);
        
        // checking all options are present or not
        let option_map: std::collections::HashMap<&str, &str> = cups_options.into_iter().collect();
        assert_eq!(option_map.get("copies"), Some(&"3"));
        assert_eq!(option_map.get("media"), Some(&crate::MEDIA_A4));
        assert_eq!(option_map.get("print-color-mode"), Some(&"color"));
        assert_eq!(option_map.get("print-quality"), Some(&"5"));
        assert_eq!(option_map.get("sides"), Some(&"two-sided-long-edge"));
        assert_eq!(option_map.get("orientation-requested"), Some(&"4"));
    }

    #[test]
    fn test_empty_print_options() {
        let options = PrintOptions::new();
        assert!(options.is_empty());
        assert_eq!(options.len(), 0);
        assert!(options.as_cups_options().is_empty());
    }

    #[test]
    fn test_custom_options() {
        let options = PrintOptions::new()
            .custom_option("custom-key", "custom-value")
            .custom_option("another-key", "another-value");

        assert_eq!(options.len(), 2);
        let cups_options = options.as_cups_options();
        let option_map: std::collections::HashMap<&str, &str> = cups_options.into_iter().collect();
        assert_eq!(option_map.get("custom-key"), Some(&"custom-value"));
        assert_eq!(option_map.get("another-key"), Some(&"another-value"));
    }

    #[test]
    fn test_enum_to_string_conversions() {
        assert_eq!(ColorMode::Auto.to_string(), "auto");
        assert_eq!(ColorMode::Color.to_string(), "color");
        assert_eq!(ColorMode::Monochrome.to_string(), "monochrome");

        assert_eq!(PrintQuality::Draft.to_string(), "3");
        assert_eq!(PrintQuality::Normal.to_string(), "4");
        assert_eq!(PrintQuality::High.to_string(), "5");

        assert_eq!(DuplexMode::OneSided.to_string(), "one-sided");
        assert_eq!(DuplexMode::TwoSidedPortrait.to_string(), "two-sided-long-edge");
        assert_eq!(DuplexMode::TwoSidedLandscape.to_string(), "two-sided-short-edge");

        assert_eq!(Orientation::Portrait.to_string(), "3");
        assert_eq!(Orientation::Landscape.to_string(), "4");
    }
}