mod dest_info;
mod media_size;
mod printer_state;

pub use dest_info::DestinationInfo;
pub use media_size::MediaSize;
pub use printer_state::PrinterState;

use crate::bindings;
use crate::constants;
use crate::error::{Error, Result};
use crate::error_helpers::cups_error_to_our_error;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint, c_void};
use std::ptr;

pub type DestCallback<T> = dyn FnMut(u32, &Destination, &mut T) -> bool;

/// Represents a printer or class of printers available for printing
#[derive(Debug, Clone)]
pub struct Destination {
    /// Name of the destination
    pub name: String,
    /// Instance name or None for the default instance
    pub instance: Option<String>,
    /// True if this is the default destination
    pub is_default: bool,
    /// Options and attributes for this destination
    pub options: HashMap<String, String>,
}

impl Destination {
    /// Create a new Destination instance from raw cups_dest_t pointer
    pub(crate) unsafe fn from_raw(dest_ptr: *const bindings::cups_dest_s) -> Result<Self> {
        if dest_ptr.is_null() {
            return Err(Error::NullPointer);
        }

        let dest = unsafe { &*dest_ptr };
        // Extract name
        let name = if dest.name.is_null() {
            return Err(Error::NullPointer);
        } else {
            unsafe { CStr::from_ptr(dest.name) }
                .to_string_lossy()
                .into_owned()
        };

        // Extract instance (if any)
        let instance = if dest.instance.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(dest.instance) }
                    .to_string_lossy()
                    .into_owned(),
            )
        };

        // Extract options
        let mut options = HashMap::new();
        if !dest.options.is_null() && dest.num_options > 0 {
            for i in 0..dest.num_options {
                unsafe {
                    let option = &*(dest.options.offset(i as isize));
                    if !option.name.is_null() && !option.value.is_null() {
                        let name = CStr::from_ptr(option.name).to_string_lossy().into_owned();
                        let value = CStr::from_ptr(option.value).to_string_lossy().into_owned();
                        options.insert(name, value);
                    }
                }
            }
        }

        Ok(Destination {
            name,
            instance,
            is_default: dest.is_default != 0,
            options,
        })
    }

    /// Get the state of this destination
    pub fn state(&self) -> PrinterState {
        match self.options.get("printer-state") {
            Some(state) => PrinterState::from_cups_state(state),
            None => PrinterState::Unknown,
        }
    }

    /// Get the reasons for the current state
    pub fn state_reasons(&self) -> Vec<String> {
        match self.options.get("printer-state-reasons") {
            Some(reasons) => reasons.split(',').map(|s| s.trim().to_string()).collect(),
            None => Vec::new(),
        }
    }

    /// Get a human-readable description of this destination
    pub fn info(&self) -> Option<&String> {
        self.options.get("printer-info")
    }

    /// Get the location of this destination
    pub fn location(&self) -> Option<&String> {
        self.options.get("printer-location")
    }

    /// Get the make and model of this destination
    pub fn make_and_model(&self) -> Option<&String> {
        self.options.get("printer-make-and-model")
    }

    /// Check if the destination is accepting jobs
    pub fn is_accepting_jobs(&self) -> bool {
        match self.options.get("printer-is-accepting-jobs") {
            Some(value) => value == "true",
            None => false,
        }
    }

    /// Get the URI associated with this destination
    pub fn uri(&self) -> Option<&String> {
        self.options.get("printer-uri-supported")
    }

    /// Get the device URI for this destination
    pub fn device_uri(&self) -> Option<&String> {
        self.options.get("device-uri")
    }

    /// Get the full name of this destination (including instance if any)
    pub fn full_name(&self) -> String {
        match &self.instance {
            Some(inst) => format!("{}/{}", self.name, inst),
            None => self.name.clone(),
        }
    }

    /// Get an option value by name
    pub fn get_option(&self, name: &str) -> Option<&String> {
        self.options.get(name)
    }

    /// Check if an option is present
    pub fn has_option(&self, name: &str) -> bool {
        self.options.contains_key(name)
    }

    /// Get all options
    pub fn get_options(&self) -> &HashMap<String, String> {
        &self.options
    }

    /// Get detailed information about this destination
    pub fn get_detailed_info(&self, http: *mut bindings::_http_s) -> Result<DestinationInfo> {
        let name_c = CString::new(self.name.as_str())?;
        let instance_c = match &self.instance {
            Some(instance) => Some(CString::new(instance.as_str())?),
            None => None,
        };

        let _instance_ptr = match &instance_c {
            Some(s) => s.as_ptr(),
            None => ptr::null(),
        };

        let mut num_options = 0;
        let mut options_ptr: *mut bindings::cups_option_s = ptr::null_mut();

        for (name, value) in &self.options {
            let name_c = CString::new(name.as_str())?;
            let value_c = CString::new(value.as_str())?;

            unsafe {
                num_options = bindings::cupsAddOption(
                    name_c.as_ptr(),
                    value_c.as_ptr(),
                    num_options,
                    &mut options_ptr,
                );
            }
        }

        let dest = bindings::cups_dest_s {
            name: name_c.into_raw(),
            instance: match instance_c {
                Some(s) => s.into_raw(),
                None => ptr::null_mut(),
            },
            is_default: if self.is_default { 1 } else { 0 },
            num_options,
            options: options_ptr,
        };

        let dinfo = unsafe {
            bindings::cupsCopyDestInfo(
                http,
                &dest as *const bindings::cups_dest_s as *mut bindings::cups_dest_s,
            )
        };

        unsafe {
            if !options_ptr.is_null() {
                bindings::cupsFreeOptions(num_options, options_ptr);
            }

            if !dest.name.is_null() {
                let _ = CString::from_raw(dest.name);
            }

            if !dest.instance.is_null() {
                let _ = CString::from_raw(dest.instance);
            }
        }

        if dinfo.is_null() {
            return Err(cups_error_to_our_error(
                "get destination info",
                Some(&self.name),
            ));
        }

        unsafe { DestinationInfo::from_raw(dinfo) }
    }

    /// Check if a specific option and value is supported by this destination
    pub fn is_option_supported(&self, http: *mut bindings::_http_s, option: &str) -> bool {
        match self.get_detailed_info(http) {
            Ok(info) => {
                // Create a raw cups_dest_t for this destination
                let name_c = match CString::new(self.name.as_str()) {
                    Ok(s) => s,
                    Err(_) => return false,
                };

                let instance_c = match &self.instance {
                    Some(instance) => match CString::new(instance.as_str()) {
                        Ok(s) => Some(s),
                        Err(_) => return false,
                    },
                    None => None,
                };

                let _instance_ptr = match &instance_c {
                    Some(s) => s.as_ptr(),
                    None => ptr::null(),
                };

                let mut num_options = 0;
                let mut options_ptr: *mut bindings::cups_option_s = ptr::null_mut();

                // Add all options
                for (name, value) in &self.options {
                    let name_c = match CString::new(name.as_str()) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    let value_c = match CString::new(value.as_str()) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    unsafe {
                        num_options = bindings::cupsAddOption(
                            name_c.as_ptr(),
                            value_c.as_ptr(),
                            num_options,
                            &mut options_ptr,
                        );
                    }
                }

                let dest = bindings::cups_dest_s {
                    name: name_c.into_raw(),
                    instance: match instance_c {
                        Some(s) => s.into_raw(),
                        None => ptr::null_mut(),
                    },
                    is_default: if self.is_default { 1 } else { 0 },
                    num_options,
                    options: options_ptr,
                };

                // Check if the option is supported
                let result = info.is_option_supported(
                    http,
                    &dest as *const bindings::cups_dest_s as *mut bindings::cups_dest_s,
                    option,
                );

                // Free the resources
                unsafe {
                    if !options_ptr.is_null() {
                        bindings::cupsFreeOptions(num_options, options_ptr);
                    }

                    // Need to free the raw strings we created
                    if !dest.name.is_null() {
                        let _ = CString::from_raw(dest.name);
                    }

                    if !dest.instance.is_null() {
                        let _ = CString::from_raw(dest.instance);
                    }
                }

                result
            }
            Err(_) => false,
        }
    }

    /// Get a pointer to a raw cups_dest_s for this destination
    pub fn as_ptr(&self) -> *mut bindings::cups_dest_s {
        // Create a raw cups_dest_t for this destination
        let name_c = match CString::new(self.name.as_str()) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        let instance_c = match &self.instance {
            Some(instance) => match CString::new(instance.as_str()) {
                Ok(s) => Some(s),
                Err(_) => return ptr::null_mut(),
            },
            None => None,
        };

        let _instance_ptr = match &instance_c {
            Some(s) => s.as_ptr(),
            None => ptr::null(),
        };

        let mut num_options = 0;
        let mut options_ptr: *mut bindings::cups_option_s = ptr::null_mut();

        // Add all options
        for (name, value) in &self.options {
            let name_c = match CString::new(name.as_str()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let value_c = match CString::new(value.as_str()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            unsafe {
                num_options = bindings::cupsAddOption(
                    name_c.as_ptr(),
                    value_c.as_ptr(),
                    num_options,
                    &mut options_ptr,
                );
            }
        }

        // Create the raw cups_dest_s struct
        let dest = Box::new(bindings::cups_dest_s {
            name: name_c.into_raw(),
            instance: match instance_c {
                Some(s) => s.into_raw(),
                None => ptr::null_mut(),
            },
            is_default: if self.is_default { 1 } else { 0 },
            num_options,
            options: options_ptr,
        });

        // Leak the box to keep the memory alive
        Box::into_raw(dest)
    }
}

/// A collection of CUPS destinations with automatic cleanup
pub struct Destinations {
    dests: *mut bindings::cups_dest_s,
    num_dests: c_int,
    _marker: PhantomData<bindings::cups_dest_s>,
}

impl Destinations {
    /// Get all available destinations from the default CUPS server
    pub fn get_all() -> Result<Self> {
        let mut dests: *mut bindings::cups_dest_s = ptr::null_mut();
        let num_dests = unsafe { bindings::cupsGetDests(&mut dests) };

        if num_dests <= 0 || dests.is_null() {
            return Err(Error::DestinationListFailed);
        }

        Ok(Destinations {
            dests,
            num_dests,
            _marker: PhantomData,
        })
    }

    /// Get a specific destination by name
    pub fn get_destination<S: AsRef<str>>(name: S) -> Result<Destination> {
        // Get all destinations first
        let all_dests = Self::get_all()?;

        // Find the specific destination
        let name_c = CString::new(name.as_ref())?;
        let dest_ptr = unsafe {
            bindings::cupsGetDest(
                name_c.as_ptr(),
                ptr::null(),
                all_dests.num_dests,
                all_dests.dests,
            )
        };

        if dest_ptr.is_null() {
            return Err(Error::DestinationNotFound(name.as_ref().to_string()));
        }

        // Convert to our Destination type
        unsafe { Destination::from_raw(dest_ptr) }
    }

    /// Get the default destination
    pub fn get_default() -> Result<Destination> {
        // Get all destinations first
        let all_dests = Self::get_all()?;

        for i in 0..all_dests.num_dests as isize {
            unsafe {
                let dest = &*(all_dests.dests.offset(i));
                if dest.is_default != 0 {
                    return Destination::from_raw(all_dests.dests.offset(i));
                }
            }
        }

        Err(Error::DestinationNotFound("Default printer".to_string()))
    }

    /// Convert to a Vec of Destination objects
    pub fn to_vec(&self) -> Result<Vec<Destination>> {
        let mut destinations = Vec::with_capacity(self.num_dests as usize);

        for i in 0..self.num_dests as isize {
            unsafe {
                match Destination::from_raw(self.dests.offset(i)) {
                    Ok(dest) => destinations.push(dest),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse destination at index {}: {}", i, e)
                    }
                }
            }
        }

        Ok(destinations)
    }

    /// Get the number of destinations
    pub fn len(&self) -> usize {
        self.num_dests as usize
    }

    /// Check if there are no destinations
    pub fn is_empty(&self) -> bool {
        self.num_dests == 0
    }

    /// Get raw pointer to destinations array (for advanced usage)
    pub fn as_ptr(&self) -> *mut bindings::cups_dest_s {
        self.dests
    }

    /// Get number of destinations
    pub fn count(&self) -> c_int {
        self.num_dests
    }
}

impl Drop for Destinations {
    fn drop(&mut self) {
        unsafe {
            if !self.dests.is_null() && self.num_dests > 0 {
                bindings::cupsFreeDests(self.num_dests, self.dests);
                self.dests = ptr::null_mut();
                self.num_dests = 0;
            }
        }
    }
}

/// Enumerate available destinations with a callback function
pub fn enum_destinations<T>(
    flags: u32,
    msec: i32,
    cancel: Option<&mut i32>,
    type_filter: u32,
    mask: u32,
    callback: &mut DestCallback<T>,
    user_data: &mut T,
) -> Result<bool> {
    // We need to create a context that will be passed to the C callback
    let mut context = EnumContext {
        callback,
        user_data,
    };

    let cancel_ptr = match cancel {
        Some(c) => c as *mut c_int,
        None => ptr::null_mut(),
    };

    let result = unsafe {
        bindings::cupsEnumDests(
            flags,
            msec as c_int,
            cancel_ptr,
            type_filter as c_uint,
            mask as c_uint,
            Some(enum_dest_callback::<T>),
            &mut context as *mut _ as *mut c_void,
        )
    };

    if result == 0 {
        Err(Error::EnumerationError(
            "Failed to enumerate destinations".to_string(),
        ))
    } else {
        Ok(true)
    }
}

// Context structure for the C callback
struct EnumContext<'a, T> {
    callback: &'a mut DestCallback<T>,
    user_data: &'a mut T,
}

// C-compatible callback function that bridges to our Rust callback
unsafe extern "C" fn enum_dest_callback<T>(
    user_data: *mut c_void,
    flags: c_uint,
    dest_ptr: *mut bindings::cups_dest_s,
) -> c_int {
    // Reconstruct our context
    let context = unsafe { &mut *(user_data as *mut EnumContext<T>) };

    // Convert the raw destination to our Rust type
    unsafe {
        match Destination::from_raw(dest_ptr) {
            Ok(dest) => {
                // Call the user's callback
                if (context.callback)(flags, &dest, context.user_data) {
                    1 // Continue enumeration
                } else {
                    0 // Stop enumeration
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse destination: {}", e);
                1 // Continue enumeration despite error
            }
        }
    }
}

/// Get all available printer destinations
pub fn get_all_destinations() -> Result<Vec<Destination>> {
    Destinations::get_all()?.to_vec()
}

/// Get a specific destination by name
pub fn get_destination<S: AsRef<str>>(name: S) -> Result<Destination> {
    Destinations::get_destination(name)
}

/// Get the default destination
pub fn get_default_destination() -> Result<Destination> {
    Destinations::get_default()
}

/// Copy a destination from one destination array to another
pub fn copy_dest(
    dest: *const bindings::cups_dest_s,
    num_dests: i32,
    dests: *mut *mut bindings::cups_dest_s,
) -> i32 {
    unsafe { bindings::cupsCopyDest(dest as *mut bindings::cups_dest_s, num_dests, dests) }
}

/// Remove a destination from an array
pub fn remove_dest(
    name: &str,
    instance: Option<&str>,
    num_dests: i32,
    dests: *mut *mut bindings::cups_dest_s,
) -> Result<i32> {
    let name_c = CString::new(name)?;
    let instance_c = match instance {
        Some(i) => Some(CString::new(i)?),
        None => None,
    };

    let instance_ptr = match &instance_c {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    };

    let result =
        unsafe { bindings::cupsRemoveDest(name_c.as_ptr(), instance_ptr, num_dests, dests) };

    Ok(result)
}

/// Find available destinations with specific filter criteria
pub fn find_destinations(type_filter: u32, mask: u32) -> Result<Vec<Destination>> {
    let mut destinations = Vec::new();

    enum_destinations(
        constants::DEST_FLAGS_NONE,
        5000, // 5 second timeout
        None,
        type_filter,
        mask,
        &mut |flags, dest, dests: &mut Vec<Destination>| {
            if (flags & constants::DEST_FLAGS_REMOVED) == 0 {
                dests.push(dest.clone());
            }
            true // Continue enumeration
        },
        &mut destinations,
    )?;

    Ok(destinations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_destination_creation() {
        let mut options = std::collections::HashMap::new();
        options.insert("printer-state".to_string(), "3".to_string());
        options.insert("printer-info".to_string(), "Test Printer".to_string());
        options.insert("printer-is-accepting-jobs".to_string(), "true".to_string());

        let dest = Destination {
            name: "TestPrinter".to_string(),
            instance: None,
            is_default: false,
            options,
        };

        assert_eq!(dest.name, "TestPrinter");
        assert_eq!(dest.full_name(), "TestPrinter");
        assert_eq!(dest.state(), PrinterState::Idle);
        assert!(dest.is_accepting_jobs());
        assert_eq!(dest.info(), Some(&"Test Printer".to_string()));
    }

    #[test]
    fn test_destination_with_instance() {
        let dest = Destination {
            name: "TestPrinter".to_string(),
            instance: Some("instance1".to_string()),
            is_default: true,
            options: std::collections::HashMap::new(),
        };

        assert_eq!(dest.full_name(), "TestPrinter/instance1");
        assert!(dest.is_default);
    }

    #[test]
    fn test_destination_state_parsing() {
        let mut options = std::collections::HashMap::new();
        
        // Test different printer states
        options.insert("printer-state".to_string(), "4".to_string());
        let dest = Destination {
            name: "Test".to_string(),
            instance: None,
            is_default: false,
            options: options.clone(),
        };
        assert_eq!(dest.state(), PrinterState::Processing);

        options.insert("printer-state".to_string(), "5".to_string());
        let dest = Destination {
            name: "Test".to_string(),
            instance: None,
            is_default: false,
            options: options.clone(),
        };
        assert_eq!(dest.state(), PrinterState::Stopped);
    }

    #[test]
    fn test_destination_state_reasons() {
        let mut options = std::collections::HashMap::new();
        options.insert("printer-state-reasons".to_string(), 
                      "media-tray-empty-error,toner-low-warning".to_string());
        
        let dest = Destination {
            name: "Test".to_string(),
            instance: None,
            is_default: false,
            options,
        };

        let reasons = dest.state_reasons();
        assert_eq!(reasons.len(), 2);
        assert!(reasons.contains(&"media-tray-empty-error".to_string()));
        assert!(reasons.contains(&"toner-low-warning".to_string()));
    }
}