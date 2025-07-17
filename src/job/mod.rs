mod status;
mod lifecycle;
mod management;
mod options;

pub use status::{JobStatus, JobInfo};
pub use management::{get_jobs, get_active_jobs, get_completed_jobs, get_job_info, cancel_job};
pub use options::{PrintOptions, ColorMode, PrintQuality, DuplexMode, Orientation};

use crate::bindings;
use crate::destination::Destination;
use crate::error::{Error, Result};
use crate::error_helpers::{cups_error_to_our_error, validate_document_format, check_document_size};
use std::ffi::CString;
use std::ptr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const FORMAT_PDF: &str = "application/pdf";
pub const FORMAT_POSTSCRIPT: &str = "application/postscript";
pub const FORMAT_TEXT: &str = "text/plain";
pub const FORMAT_JPEG: &str = "image/jpeg";

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

    pub fn submit_file<P: AsRef<Path>>(&self, file_path: P, format: &str) -> Result<()> {
        let path = file_path.as_ref();
        
        if !path.exists() {
            return Err(Error::DocumentSubmissionFailed(
                format!("File not found: {}", path.display())
            ));
        }

        validate_document_format(format, &self.dest_name)?;

        let metadata = path.metadata().map_err(|e| {
            Error::DocumentSubmissionFailed(format!("Cannot access file metadata: {}", e))
        })?;

        check_document_size(metadata.len() as usize, None)?;

        let mut file = File::open(path).map_err(|e| {
            Error::DocumentSubmissionFailed(format!("Failed to open file: {}", e))
        })?;

        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(|e| {
            Error::DocumentSubmissionFailed(format!("Failed to read file: {}", e))
        })?;

        self.submit_data(&content, format, path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document"))
    }

    pub fn submit_data(&self, data: &[u8], format: &str, doc_name: &str) -> Result<()> {
        validate_document_format(format, &self.dest_name)?;
        check_document_size(data.len(), None)?;

        let dest = crate::get_destination(&self.dest_name)?;
        
        if !dest.is_accepting_jobs() {
            return Err(Error::PrinterNotAccepting(
                self.dest_name.clone(),
                "Printer is currently not accepting jobs".to_string()
            ));
        }

        let dest_info = dest.get_detailed_info(ptr::null_mut())?;
        let dest_ptr = dest.as_ptr();

        if dest_ptr.is_null() {
            return Err(Error::NullPointer);
        }

        let doc_name_c = CString::new(doc_name)?;
        let format_c = CString::new(format)?;

        let status = unsafe {
            bindings::cupsStartDestDocument(
                ptr::null_mut(),
                dest_ptr,
                dest_info.as_ptr(),
                self.id,
                doc_name_c.as_ptr(),
                format_c.as_ptr(),
                0,
                ptr::null_mut(),
                1,
            )
        };

        if status != bindings::http_status_e_HTTP_STATUS_CONTINUE as bindings::http_status_t {
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

            return Err(cups_error_to_our_error("document start", Some(&self.dest_name)));
        }

        let mut bytes_written = 0;
        let mut remaining = data.len();
        
        while remaining > 0 {
            let chunk_size = remaining.min(8192);
            let chunk = &data[bytes_written..bytes_written + chunk_size];
            
            let result = unsafe {
                bindings::cupsWriteRequestData(
                    ptr::null_mut(),
                    chunk.as_ptr() as *const i8,
                    chunk_size,
                )
            };

            if result != bindings::http_status_e_HTTP_STATUS_CONTINUE as bindings::http_status_t {
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

                return Err(Error::DocumentSubmissionFailed(
                    format!("Failed to write data at byte {} (network error or timeout)", bytes_written)
                ));
            }

            bytes_written += chunk_size;
            remaining -= chunk_size;
        }

        let finish_status = unsafe {
            bindings::cupsFinishDestDocument(
                ptr::null_mut(),
                dest_ptr,
                dest_info.as_ptr(),
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

        if finish_status == bindings::ipp_status_e_IPP_STATUS_OK as bindings::ipp_status_t {
            Ok(())
        } else {
            Err(cups_error_to_our_error("document finish", Some(&self.dest_name)))
        }
    }
}

pub fn create_job(dest: &Destination, title: &str) -> Result<Job> {
    if !dest.is_accepting_jobs() {
        return Err(Error::PrinterNotAccepting(
            dest.name.clone(),
            "Printer is not accepting new jobs".to_string()
        ));
    }

    let title_c = CString::new(title)?;
    let dest_info = dest.get_detailed_info(ptr::null_mut())?;
    let dest_ptr = dest.as_ptr();
    
    if dest_ptr.is_null() {
        return Err(Error::NullPointer);
    }
    
    let mut job_id: i32 = 0;
    
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
        Err(cups_error_to_our_error("job creation", Some(&dest.name)))
    }
}

pub fn create_job_with_options(dest: &Destination, title: &str, options: &PrintOptions) -> Result<Job> {
    if !dest.is_accepting_jobs() {
        return Err(Error::PrinterNotAccepting(
            dest.name.clone(),
            "Printer is not accepting new jobs".to_string()
        ));
    }

    let title_c = CString::new(title)?;
    let dest_info = dest.get_detailed_info(ptr::null_mut())?;
    let dest_ptr = dest.as_ptr();
    
    if dest_ptr.is_null() {
        return Err(Error::NullPointer);
    }

    let cups_options = options.as_cups_options();
    let mut cups_options_ptr: *mut bindings::cups_option_s = ptr::null_mut();
    let mut num_options = 0;

    for (name, value) in &cups_options {
        let name_c = CString::new(*name)?;
        let value_c = CString::new(*value)?;

        unsafe {
            num_options = bindings::cupsAddOption(
                name_c.as_ptr(),
                value_c.as_ptr(),
                num_options,
                &mut cups_options_ptr,
            );
        }
    }
    
    let mut job_id: i32 = 0;
    
    let status = unsafe {
        bindings::cupsCreateDestJob(
            ptr::null_mut(),
            dest_ptr,
            dest_info.as_ptr(),
            &mut job_id,
            title_c.as_ptr(),
            num_options,
            cups_options_ptr,
        )
    };

    unsafe {
        if !cups_options_ptr.is_null() {
            bindings::cupsFreeOptions(num_options, cups_options_ptr);
        }

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
        Err(cups_error_to_our_error("job creation with options", Some(&dest.name)))
    }
}