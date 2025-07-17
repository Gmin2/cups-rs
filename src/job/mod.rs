use crate::bindings;
use crate::destination::Destination;
use crate::error::{Error, Result};
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

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
            return Err(Error::DocumentSubmissionFailed(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let mut file = File::open(path)
            .map_err(|e| Error::DocumentSubmissionFailed(format!("Failed to open file: {}", e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| Error::DocumentSubmissionFailed(format!("Failed to read file: {}", e)))?;

        self.submit_data(
            &content,
            format,
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("document"),
        )
    }

    pub fn submit_data(&self, data: &[u8], format: &str, doc_name: &str) -> Result<()> {
        let dest = crate::get_destination(&self.dest_name)?;
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

            return Err(Error::DocumentSubmissionFailed(
                "Failed to start document".to_string(),
            ));
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

                return Err(Error::DocumentSubmissionFailed(format!(
                    "Failed to write data at byte {}",
                    bytes_written
                )));
            }

            bytes_written += chunk_size;
            remaining -= chunk_size;
        }

        let finish_status = unsafe {
            bindings::cupsFinishDestDocument(ptr::null_mut(), dest_ptr, dest_info.as_ptr())
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
            Err(Error::DocumentSubmissionFailed(
                "Failed to finish document".to_string(),
            ))
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
