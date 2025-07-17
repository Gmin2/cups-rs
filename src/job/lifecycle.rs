use crate::bindings;
use crate::error::{Error, Result};
use super::Job;
use std::ffi::CString;
use std::ptr;

impl Job {
    pub fn close(&self) -> Result<()> {
        let dest = crate::get_destination(&self.dest_name)?;
        let dest_info = dest.get_detailed_info(ptr::null_mut())?;
        let dest_ptr = dest.as_ptr();

        if dest_ptr.is_null() {
            return Err(Error::NullPointer);
        }

        let status = unsafe {
            bindings::cupsCloseDestJob(
                ptr::null_mut(),
                dest_ptr,
                dest_info.as_ptr(),
                self.id,
            )
        };

        unsafe {
            let dest_box = Box::from_raw(dest_ptr);
            if !dest_box.name.is_null() {
                let _ = CString::from_raw(dest_box.name);
            }
            if !dest_box.instance.is_null() {
                let _ = CString::from_raw(dest_box.instance);
            }
            if !dest_box.options.is_null() {
                bindings::cupsFreeOptions(dest_box.num_options, dest_box.options);
            }
        }

        if status == bindings::ipp_status_e_IPP_STATUS_OK as bindings::ipp_status_t {
            Ok(())
        } else {
            let error_msg = unsafe {
                let error_ptr = bindings::cupsLastErrorString();
                if error_ptr.is_null() {
                    "Unknown CUPS error".to_string()
                } else {
                    std::ffi::CStr::from_ptr(error_ptr)
                        .to_string_lossy()
                        .into_owned()
                }
            };
            Err(Error::JobManagementFailed(format!(
                "Failed to close job {}: {}",
                self.id, error_msg
            )))
        }
    }

    pub fn cancel(&self) -> Result<()> {
        let dest = crate::get_destination(&self.dest_name)?;
        let dest_ptr = dest.as_ptr();

        if dest_ptr.is_null() {
            return Err(Error::NullPointer);
        }

        let status = unsafe {
            bindings::cupsCancelDestJob(
                ptr::null_mut(),
                dest_ptr,
                self.id,
            )
        };

        unsafe {
            let dest_box = Box::from_raw(dest_ptr);
            if !dest_box.name.is_null() {
                let _ = CString::from_raw(dest_box.name);
            }
            if !dest_box.instance.is_null() {
                let _ = CString::from_raw(dest_box.instance);
            }
            if !dest_box.options.is_null() {
                bindings::cupsFreeOptions(dest_box.num_options, dest_box.options);
            }
        }

        if status == bindings::ipp_status_e_IPP_STATUS_OK as bindings::ipp_status_t {
            Ok(())
        } else {
            let error_msg = unsafe {
                let error_ptr = bindings::cupsLastErrorString();
                if error_ptr.is_null() {
                    "Unknown CUPS error".to_string()
                } else {
                    std::ffi::CStr::from_ptr(error_ptr)
                        .to_string_lossy()
                        .into_owned()
                }
            };
            Err(Error::JobManagementFailed(format!(
                "Failed to cancel job {}: {}",
                self.id, error_msg
            )))
        }
    }
}