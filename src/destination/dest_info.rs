use crate::bindings;
use crate::destination::media_size::MediaSize;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr;

/// Detailed information about a destination, including supported options and values
pub struct DestinationInfo {
    dinfo: *mut bindings::_cups_dinfo_s,
    _phantom: PhantomData<bindings::_cups_dinfo_s>,
}

impl DestinationInfo {
    /// Create a new DestinationInfo from a cups_dinfo_t pointer
    pub(crate) unsafe fn from_raw(dinfo: *mut bindings::_cups_dinfo_s) -> Result<Self> {
        if dinfo.is_null() {
            return Err(Error::DetailedInfoUnavailable);
        }

        Ok(DestinationInfo {
            dinfo,
            _phantom: PhantomData,
        })
    }

    /// Get the raw pointer to the cups_dinfo_t structure
    pub fn as_ptr(&self) -> *mut bindings::_cups_dinfo_s {
        self.dinfo
    }

    /// Check if an option is supported
    pub fn is_option_supported(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        option: &str,
    ) -> bool {
        let option_c = match CString::new(option) {
            Ok(s) => s,
            Err(_) => return false,
        };

        unsafe {
            bindings::cupsCheckDestSupported(http, dest, self.dinfo, option_c.as_ptr(), ptr::null())
                != 0
        }
    }

    /// Check if a specific option and value is supported
    pub fn is_value_supported(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        option: &str,
        value: &str,
    ) -> bool {
        let option_c = match CString::new(option) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let value_c = match CString::new(value) {
            Ok(s) => s,
            Err(_) => return false,
        };

        unsafe {
            bindings::cupsCheckDestSupported(
                http,
                dest,
                self.dinfo,
                option_c.as_ptr(),
                value_c.as_ptr(),
            ) != 0
        }
    }

    /// Get media by name
    pub fn get_media_by_name(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        media: &str,
        flags: u32,
    ) -> Result<MediaSize> {
        let media_c = CString::new(media)?;
        let mut size = bindings::cups_size_s {
            media: [0; 128],
            width: 0,
            length: 0,
            bottom: 0,
            left: 0,
            right: 0,
            top: 0,
        };

        let result = unsafe {
            bindings::cupsGetDestMediaByName(
                http,
                dest,
                self.dinfo,
                media_c.as_ptr(),
                flags,
                &mut size,
            )
        };

        if result == 0 {
            Err(Error::MediaSizeError(format!(
                "Media '{}' not found",
                media
            )))
        } else {
            unsafe { MediaSize::from_cups_size(&size) }
        }
    }

    /// Get media by size
    pub fn get_media_by_size(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        width: i32,
        length: i32,
        flags: u32,
    ) -> Result<MediaSize> {
        let mut size = bindings::cups_size_s {
            media: [0; 128],
            width: 0,
            length: 0,
            bottom: 0,
            left: 0,
            right: 0,
            top: 0,
        };

        let result = unsafe {
            bindings::cupsGetDestMediaBySize(
                http, dest, self.dinfo, width, length, flags, &mut size,
            )
        };

        if result == 0 {
            Err(Error::MediaSizeError(format!(
                "Media with width={} and length={} not found",
                width, length
            )))
        } else {
            unsafe { MediaSize::from_cups_size(&size) }
        }
    }

    /// Get media by index
    pub fn get_media_by_index(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        index: i32,
        flags: u32,
    ) -> Result<MediaSize> {
        let mut size = bindings::cups_size_s {
            media: [0; 128],
            width: 0,
            length: 0,
            bottom: 0,
            left: 0,
            right: 0,
            top: 0,
        };

        let result = unsafe {
            bindings::cupsGetDestMediaByIndex(http, dest, self.dinfo, index, flags, &mut size)
        };

        if result == 0 {
            Err(Error::MediaSizeError(format!(
                "Media at index {} not found",
                index
            )))
        } else {
            unsafe { MediaSize::from_cups_size(&size) }
        }
    }

    /// Get default media
    pub fn get_default_media(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        flags: u32,
    ) -> Result<MediaSize> {
        let mut size = bindings::cups_size_s {
            media: [0; 128],
            width: 0,
            length: 0,
            bottom: 0,
            left: 0,
            right: 0,
            top: 0,
        };

        let result =
            unsafe { bindings::cupsGetDestMediaDefault(http, dest, self.dinfo, flags, &mut size) };

        if result == 0 {
            Err(Error::MediaSizeError("Default media not found".to_string()))
        } else {
            unsafe { MediaSize::from_cups_size(&size) }
        }
    }

    /// Get count of available media
    pub fn get_media_count(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        flags: u32,
    ) -> i32 {
        unsafe { bindings::cupsGetDestMediaCount(http, dest, self.dinfo, flags) }
    }

    /// Get all available media
    pub fn get_all_media(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        flags: u32,
    ) -> Result<Vec<MediaSize>> {
        let count = self.get_media_count(http, dest, flags);
        let mut media_sizes = Vec::with_capacity(count as usize);

        for i in 0..count {
            match self.get_media_by_index(http, dest, i, flags) {
                Ok(size) => media_sizes.push(size),
                Err(e) => eprintln!("Warning: Failed to get media at index {}: {}", i, e),
            }
        }

        Ok(media_sizes)
    }

    /// Localize a media name
    pub fn localize_media(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        flags: u32,
        size: &MediaSize,
    ) -> Result<String> {
        let mut cups_size = bindings::cups_size_s {
            media: [0; 128],
            width: size.width,
            length: size.length,
            bottom: size.bottom,
            left: size.left,
            right: size.right,
            top: size.top,
        };

        // Copy the media name into the cups_size_t structure
        let name_bytes = size.name.as_bytes();
        let max_len = 127.min(name_bytes.len());
        for i in 0..max_len {
            cups_size.media[i] = name_bytes[i] as i8;
        }
        cups_size.media[max_len] = 0;

        let result = unsafe {
            bindings::cupsLocalizeDestMedia(http, dest, self.dinfo, flags, &mut cups_size)
        };

        if result.is_null() {
            Err(Error::UnsupportedFeature(
                "Media localization not supported".to_string(),
            ))
        } else {
            let localized = unsafe { CStr::from_ptr(result) }
                .to_string_lossy()
                .into_owned();
            Ok(localized)
        }
    }

    /// Localize an option name
    pub fn localize_option(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        option: &str,
    ) -> Result<String> {
        let option_c = CString::new(option)?;

        let result =
            unsafe { bindings::cupsLocalizeDestOption(http, dest, self.dinfo, option_c.as_ptr()) };

        if result.is_null() {
            Err(Error::UnsupportedFeature(format!(
                "Option localization not supported for '{}'",
                option
            )))
        } else {
            let localized = unsafe { CStr::from_ptr(result) }
                .to_string_lossy()
                .into_owned();
            Ok(localized)
        }
    }

    /// Localize an option value
    pub fn localize_value(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        option: &str,
        value: &str,
    ) -> Result<String> {
        let option_c = CString::new(option)?;
        let value_c = CString::new(value)?;

        let result = unsafe {
            bindings::cupsLocalizeDestValue(
                http,
                dest,
                self.dinfo,
                option_c.as_ptr(),
                value_c.as_ptr(),
            )
        };

        if result.is_null() {
            Err(Error::UnsupportedFeature(format!(
                "Value localization not supported for '{}'='{}'",
                option, value
            )))
        } else {
            let localized = unsafe { CStr::from_ptr(result) }
                .to_string_lossy()
                .into_owned();
            Ok(localized)
        }
    }

    /// Get ready (loaded) media
    /// 
    /// Returns the media sizes that are currently loaded/ready in the printer.
    /// This is different from supported media - ready media are the ones actually
    /// available for immediate use.
    pub fn get_ready_media(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
    ) -> Result<Vec<MediaSize>> {
        let option_c = match CString::new("media") {
            Ok(s) => s,
            Err(_) => return Ok(Vec::new()),
        };

        let ready_attr = unsafe {
            bindings::cupsFindDestReady(http, dest, self.dinfo, option_c.as_ptr())
        };

        if ready_attr.is_null() {
            return Ok(Vec::new());
        }

        let mut ready_media = Vec::new();
        let count = unsafe { bindings::ippGetCount(ready_attr) };
        
        for i in 0..count {
            unsafe {
                let media_name_ptr = bindings::ippGetString(ready_attr, i, ptr::null_mut());
                if !media_name_ptr.is_null() {
                    let media_name = CStr::from_ptr(media_name_ptr).to_string_lossy();
                    
                    // Try to get the full media size info for this ready media
                    match self.get_media_by_name(http, dest, &media_name, 0) {
                        Ok(size) => ready_media.push(size),
                        Err(_) => {
                            // If we can't get full info, create a basic MediaSize
                            ready_media.push(MediaSize {
                                name: media_name.into_owned(),
                                width: 0,
                                length: 0,
                                left: 0,
                                bottom: 0,
                                right: 0,
                                top: 0,
                            });
                        }
                    }
                }
            }
        }

        Ok(ready_media)
    }

    /// Get ready (loaded) finishings
    /// 
    /// Returns the finishing processes that are currently ready/available.
    /// For example, if a printer has staple and punch finishers but is out of staples,
    /// this will only return punch options.
    pub fn get_ready_finishings(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
    ) -> Result<Vec<i32>> {
        let option_c = match CString::new("finishings") {
            Ok(s) => s,
            Err(_) => return Ok(Vec::new()),
        };

        let ready_attr = unsafe {
            bindings::cupsFindDestReady(http, dest, self.dinfo, option_c.as_ptr())
        };

        if ready_attr.is_null() {
            return Ok(Vec::new());
        }

        let mut ready_finishings = Vec::new();
        let count = unsafe { bindings::ippGetCount(ready_attr) };
        
        for i in 0..count {
            let finishing = unsafe { bindings::ippGetInteger(ready_attr, i) };
            ready_finishings.push(finishing);
        }

        Ok(ready_finishings)
    }

    /// Get default value for an option
    /// 
    /// Returns the default value for a given option as a string.
    /// This is the printer's default, not the user's saved preference.
    pub fn get_default_value(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        option: &str,
    ) -> Result<Option<String>> {
        let option_c = CString::new(option)?;

        let default_attr = unsafe {
            bindings::cupsFindDestDefault(http, dest, self.dinfo, option_c.as_ptr())
        };

        if default_attr.is_null() {
            return Ok(None);
        }

        // Try to get as string first
        unsafe {
            let value_ptr = bindings::ippGetString(default_attr, 0, ptr::null_mut());
            if !value_ptr.is_null() {
                let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
                return Ok(Some(value));
            }

            // If not a string, try as integer
            let int_value = bindings::ippGetInteger(default_attr, 0);
            if int_value != 0 {
                return Ok(Some(int_value.to_string()));
            }

            // If not an integer, try as boolean
            let bool_value = bindings::ippGetBoolean(default_attr, 0);
            Ok(Some(if bool_value != 0 { "true".to_string() } else { "false".to_string() }))
        }
    }

    /// Get supported values for an option
    /// 
    /// Returns a list of all values supported for the given option.
    /// The returned values are formatted as strings.
    pub fn get_supported_values(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
        option: &str,
    ) -> Result<Vec<String>> {
        let option_c = CString::new(option)?;

        let supported_attr = unsafe {
            bindings::cupsFindDestSupported(http, dest, self.dinfo, option_c.as_ptr())
        };

        if supported_attr.is_null() {
            return Ok(Vec::new());
        }

        let mut supported_values = Vec::new();
        let count = unsafe { bindings::ippGetCount(supported_attr) };
        
        for i in 0..count {
            unsafe {
                // Try to get as string first
                let value_ptr = bindings::ippGetString(supported_attr, i, ptr::null_mut());
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
                    supported_values.push(value);
                    continue;
                }

                // If not a string, try as integer
                let int_value = bindings::ippGetInteger(supported_attr, i);
                if int_value != 0 || i == 0 { // Include 0 if it's the first value
                    supported_values.push(int_value.to_string());
                    continue;
                }

                // If not an integer, try as boolean
                let bool_value = bindings::ippGetBoolean(supported_attr, i);
                supported_values.push(if bool_value != 0 { "true".to_string() } else { "false".to_string() });
            }
        }

        Ok(supported_values)
    }

    /// Get supported options for job creation
    /// 
    /// Returns a list of all options that can be used when creating jobs
    /// for this destination.
    pub fn get_supported_options(
        &self,
        http: *mut bindings::_http_s,
        dest: *mut bindings::cups_dest_s,
    ) -> Result<Vec<String>> {
        self.get_supported_values(http, dest, "job-creation-attributes")
    }
}

impl Drop for DestinationInfo {
    fn drop(&mut self) {
        if !self.dinfo.is_null() {
            unsafe {
                bindings::cupsFreeDestInfo(self.dinfo);
            }
            self.dinfo = ptr::null_mut();
        }
    }
}
