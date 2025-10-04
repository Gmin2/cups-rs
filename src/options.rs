use crate::bindings;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;

/// Parse command-line style options into key-value pairs
///
/// Converts space-delimited name/value pairs according to the PAPI text option ABNF specification.
/// Collection values ("name={a=... b=... c=...}") are stored with curly brackets intact.
///
/// # Arguments
/// * `arg` - Command-line argument string to parse (e.g., "copies=2 sides=two-sided-long-edge")
///
/// # Returns
/// * `Ok(Vec<(String, String)>)` - Parsed options as key-value pairs
/// * `Err(Error)` - Parsing failed
///
/// # Example
/// ```
/// let options = parse_options("copies=2 media=a4 sides=two-sided-long-edge")?;
/// // Returns: [("copies", "2"), ("media", "a4"), ("sides", "two-sided-long-edge")]
/// ```
pub fn parse_options(arg: &str) -> Result<Vec<(String, String)>> {
    let arg_c = CString::new(arg)?;

    let mut num_options: c_int = 0;
    let mut options_ptr: *mut bindings::cups_option_s = ptr::null_mut();

    let result = unsafe {
        bindings::cupsParseOptions(arg_c.as_ptr(), num_options, &mut options_ptr)
    };

    if result < 0 {
        return Err(Error::ConfigurationError(format!(
            "Failed to parse options: '{}'",
            arg
        )));
    }

    num_options = result;

    // Convert the options array to a Vec
    let mut parsed_options = Vec::with_capacity(num_options as usize);

    for i in 0..num_options {
        unsafe {
            let option = options_ptr.offset(i as isize);
            if !(*option).name.is_null() && !(*option).value.is_null() {
                let name = CStr::from_ptr((*option).name)
                    .to_string_lossy()
                    .into_owned();
                let value = CStr::from_ptr((*option).value)
                    .to_string_lossy()
                    .into_owned();
                parsed_options.push((name, value));
            }
        }
    }

    // Free the options array
    if !options_ptr.is_null() {
        unsafe {
            bindings::cupsFreeOptions(num_options, options_ptr);
        }
    }

    Ok(parsed_options)
}

/// Add an option to an options array
///
/// This is a low-level function that works with CUPS option arrays.
///
/// # Arguments
/// * `name` - Option name
/// * `value` - Option value
/// * `options` - Current options vector
///
/// # Returns
/// * Updated options vector with the new option added (or replaced if it existed)
pub fn add_option(name: &str, value: &str, mut options: Vec<(String, String)>) -> Vec<(String, String)> {
    // Remove existing option with the same name
    options.retain(|(n, _)| n != name);

    // Add the new option
    options.push((name.to_string(), value.to_string()));

    options
}

/// Add an integer option to an options array
///
/// Convenience function for adding integer-valued options.
///
/// # Arguments
/// * `name` - Option name
/// * `value` - Integer value
/// * `options` - Current options vector
///
/// # Returns
/// * Updated options vector with the new option added
pub fn add_integer_option(name: &str, value: i32, options: Vec<(String, String)>) -> Vec<(String, String)> {
    add_option(name, &value.to_string(), options)
}

/// Remove an option from an options array
///
/// # Arguments
/// * `name` - Option name to remove
/// * `options` - Current options vector
///
/// # Returns
/// * `(updated_options, was_removed)` - Updated vector and boolean indicating if option was found
pub fn remove_option(name: &str, mut options: Vec<(String, String)>) -> (Vec<(String, String)>, bool) {
    let initial_len = options.len();
    options.retain(|(n, _)| n != name);
    let was_removed = options.len() < initial_len;
    (options, was_removed)
}

/// Get the value of an option
///
/// # Arguments
/// * `name` - Option name to look up
/// * `options` - Options vector to search
///
/// # Returns
/// * `Some(value)` - Option value if found
/// * `None` - Option not found
pub fn get_option<'a>(name: &str, options: &'a [(String, String)]) -> Option<&'a str> {
    options
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

/// Get the integer value of an option
///
/// # Arguments
/// * `name` - Option name to look up
/// * `options` - Options vector to search
///
/// # Returns
/// * `Some(value)` - Parsed integer value if found and valid
/// * `None` - Option not found or not a valid integer
pub fn get_integer_option(name: &str, options: &[(String, String)]) -> Option<i32> {
    get_option(name, options).and_then(|v| v.parse::<i32>().ok())
}

/// Encode a single option into an IPP attribute
///
/// This function converts a single option name/value pair into an IPP attribute
/// and adds it to the IPP request/response.
///
/// Note: This requires IPP support which is currently limited. This function
/// provides a basic wrapper but full IPP functionality is not yet implemented.
///
/// # Arguments
/// * `ipp` - IPP request/response pointer
/// * `group_tag` - IPP attribute group tag
/// * `name` - Option name
/// * `value` - Option value
///
/// # Returns
/// * `Ok(())` - Option encoded successfully
/// * `Err(Error)` - Encoding failed
pub fn encode_option(
    ipp: *mut bindings::ipp_s,
    group_tag: bindings::ipp_tag_t,
    name: &str,
    value: &str,
) -> Result<()> {
    if ipp.is_null() {
        return Err(Error::NullPointer);
    }

    let name_c = CString::new(name)?;
    let value_c = CString::new(value)?;

    let attr = unsafe {
        bindings::cupsEncodeOption(ipp, group_tag, name_c.as_ptr(), value_c.as_ptr())
    };

    if attr.is_null() {
        Err(Error::ConfigurationError(format!(
            "Failed to encode option '{}' = '{}'",
            name, value
        )))
    } else {
        Ok(())
    }
}

/// Encode multiple options into IPP attributes
///
/// This function adds operation, job, and subscription attributes in that order.
/// For group-specific encoding, use `encode_options_with_group`.
///
/// Note: This requires IPP support which is currently limited.
///
/// # Arguments
/// * `ipp` - IPP request/response pointer
/// * `options` - Options to encode
///
/// # Returns
/// * `Ok(())` - Options encoded successfully
/// * `Err(Error)` - Encoding failed
pub fn encode_options(
    ipp: *mut bindings::ipp_s,
    options: &[(String, String)],
) -> Result<()> {
    if ipp.is_null() {
        return Err(Error::NullPointer);
    }

    // Convert to cups_option_t array
    let mut cups_options: Vec<bindings::cups_option_s> = Vec::with_capacity(options.len());
    let mut c_strings: Vec<(CString, CString)> = Vec::with_capacity(options.len());

    for (name, value) in options {
        let name_c = CString::new(name.as_str())?;
        let value_c = CString::new(value.as_str())?;

        cups_options.push(bindings::cups_option_s {
            name: name_c.as_ptr() as *mut i8,
            value: value_c.as_ptr() as *mut i8,
        });

        c_strings.push((name_c, value_c));
    }

    unsafe {
        bindings::cupsEncodeOptions(
            ipp,
            cups_options.len() as c_int,
            cups_options.as_mut_ptr(),
        );
    }

    Ok(())
}

/// Encode multiple options into IPP attributes for a specific group
///
/// This function only adds attributes for a single group tag.
///
/// Note: This requires IPP support which is currently limited.
///
/// # Arguments
/// * `ipp` - IPP request/response pointer
/// * `options` - Options to encode
/// * `group_tag` - IPP attribute group tag
///
/// # Returns
/// * `Ok(())` - Options encoded successfully
/// * `Err(Error)` - Encoding failed
pub fn encode_options_with_group(
    ipp: *mut bindings::ipp_s,
    options: &[(String, String)],
    group_tag: bindings::ipp_tag_t,
) -> Result<()> {
    if ipp.is_null() {
        return Err(Error::NullPointer);
    }

    // Convert to cups_option_t array
    let mut cups_options: Vec<bindings::cups_option_s> = Vec::with_capacity(options.len());
    let mut c_strings: Vec<(CString, CString)> = Vec::with_capacity(options.len());

    for (name, value) in options {
        let name_c = CString::new(name.as_str())?;
        let value_c = CString::new(value.as_str())?;

        cups_options.push(bindings::cups_option_s {
            name: name_c.as_ptr() as *mut i8,
            value: value_c.as_ptr() as *mut i8,
        });

        c_strings.push((name_c, value_c));
    }

    unsafe {
        bindings::cupsEncodeOptions2(
            ipp,
            cups_options.len() as c_int,
            cups_options.as_mut_ptr(),
            group_tag,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_options() {
        let result = parse_options("copies=2 media=a4");
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.len(), 2);
        assert!(options.contains(&("copies".to_string(), "2".to_string())));
        assert!(options.contains(&("media".to_string(), "a4".to_string())));
    }

    #[test]
    fn test_add_option() {
        let options = vec![];
        let options = add_option("copies", "2", options);
        assert_eq!(options.len(), 1);
        assert_eq!(options[0], ("copies".to_string(), "2".to_string()));

        // Replace existing option
        let options = add_option("copies", "3", options);
        assert_eq!(options.len(), 1);
        assert_eq!(options[0], ("copies".to_string(), "3".to_string()));
    }

    #[test]
    fn test_add_integer_option() {
        let options = vec![];
        let options = add_integer_option("copies", 5, options);
        assert_eq!(options.len(), 1);
        assert_eq!(options[0], ("copies".to_string(), "5".to_string()));
    }

    #[test]
    fn test_remove_option() {
        let mut options = vec![
            ("copies".to_string(), "2".to_string()),
            ("media".to_string(), "a4".to_string()),
        ];

        let (options, removed) = remove_option("copies", options);
        assert!(removed);
        assert_eq!(options.len(), 1);
        assert_eq!(options[0], ("media".to_string(), "a4".to_string()));

        let (options, removed) = remove_option("nonexistent", options);
        assert!(!removed);
        assert_eq!(options.len(), 1);
    }

    #[test]
    fn test_get_option() {
        let options = vec![
            ("copies".to_string(), "2".to_string()),
            ("media".to_string(), "a4".to_string()),
        ];

        assert_eq!(get_option("copies", &options), Some("2"));
        assert_eq!(get_option("media", &options), Some("a4"));
        assert_eq!(get_option("nonexistent", &options), None);
    }

    #[test]
    fn test_get_integer_option() {
        let options = vec![
            ("copies".to_string(), "2".to_string()),
            ("media".to_string(), "a4".to_string()),
        ];

        assert_eq!(get_integer_option("copies", &options), Some(2));
        assert_eq!(get_integer_option("media", &options), None);
        assert_eq!(get_integer_option("nonexistent", &options), None);
    }
}
