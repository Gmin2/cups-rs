use crate::bindings;
use crate::destination::Destination;
use crate::error::{Error, Result};
use std::ffi::CString;
use std::ptr;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: i32,
    pub dest_name: String,
    pub title: String,
}

impl Job {
    pub fn new(id: i32, dest_name: String, title: String) -> Self {
        Job {
            id,
            dest_name,
            title,
        }
    }
}

pub fn create_job(dest: &Destination, title: &str) -> Result<Job> {
    let title_c = CString::new(title)?;
    
    let dest_info = dest.get_detailed_info(ptr::null_mut())?;
    
    let dest_ptr = dest.as_ptr();
    if dest_ptr.is_null() {
        return Err(Error::NullPointer);
    }
    
    let mut job_id: i32 = 0;
    
    // create job using cups api
    let status = unsafe {
        bindings::cupsCreateDestJob(
            ptr::null_mut(),
            dest_ptr,
            dest_info.as_ptr(),
            &mut job_id,
            title_c.as_ptr(),
            0,
            ptr::null_mut(),
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
        Ok(Job::new(job_id, dest.name.clone(), title.to_string()))
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
        
        Err(Error::JobCreationFailed(format!(
            "CUPS job creation failed with status {}: {}",
            status, error_msg
        )))
    }
}