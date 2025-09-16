use crate::bindings;
use crate::destination::{DestCallback, Destination};
use crate::error::{Error, Result};
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

/// Connection flags for cupsConnectDest
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFlags {
    /// Connect to CUPS scheduler
    Scheduler = 0,
    /// Connect directly to device/printer
    Device = 1,
}

impl From<ConnectionFlags> for u32 {
    fn from(flags: ConnectionFlags) -> u32 {
        match flags {
            ConnectionFlags::Scheduler => 0,
            ConnectionFlags::Device => 1,
        }
    }
}

/// Represents an HTTP connection to a CUPS server or printer
pub struct HttpConnection {
    http: *mut bindings::_http_s,
    resource: String,
    _phantom: PhantomData<bindings::_http_s>,
}

impl HttpConnection {
    /// Create a new HttpConnection from a raw http_t pointer
    pub(crate) unsafe fn from_raw(http: *mut bindings::_http_s, resource: String) -> Result<Self> {
        if http.is_null() {
            return Err(Error::ConnectionFailed(
                "Failed to establish connection".to_string(),
            ));
        }

        Ok(HttpConnection {
            http,
            resource,
            _phantom: PhantomData,
        })
    }

    /// Get the raw pointer to the http_t structure
    pub fn as_ptr(&self) -> *mut bindings::_http_s {
        self.http
    }

    /// Get the resource path for this connection
    pub fn resource_path(&self) -> &str {
        &self.resource
    }

    /// Close the HTTP connection
    pub fn close(&mut self) {
        if !self.http.is_null() {
            unsafe {
                bindings::httpClose(self.http);
            }
            self.http = ptr::null_mut();
        }
    }

    /// Check if the connection is still valid
    pub fn is_connected(&self) -> bool {
        !self.http.is_null()
    }
}

impl Drop for HttpConnection {
    fn drop(&mut self) {
        self.close();
    }
}

impl Destination {
    /// Connect to this destination
    /// 
    /// Opens a direct connection to the destination, which can be used for
    /// sending IPP requests directly to the printer or CUPS scheduler.
    /// 
    /// # Arguments
    /// * `flags` - Whether to connect to scheduler or device directly
    /// * `timeout_ms` - Connection timeout in milliseconds, None for indefinite
    /// * `cancel` - Optional cancellation flag
    /// 
    /// # Returns
    /// * `Ok((HttpConnection, String))` - Connection and resource path
    /// * `Err(Error)` - Connection failed
    pub fn connect(
        &self,
        flags: ConnectionFlags,
        timeout_ms: Option<i32>,
        cancel: Option<&AtomicBool>,
    ) -> Result<HttpConnection> {
        // Create a raw cups_dest_t for this destination
        let dest_ptr = self.as_ptr();
        if dest_ptr.is_null() {
            return Err(Error::NullPointer);
        }

        let timeout = timeout_ms.unwrap_or(-1);
        let mut cancel_int: c_int = 0;
        let cancel_ptr = if cancel.is_some() {
            &mut cancel_int as *mut c_int
        } else {
            ptr::null_mut()
        };

        // Allocate resource buffer
        const RESOURCE_SIZE: usize = 1024;
        let mut resource_buf: Vec<u8> = vec![0; RESOURCE_SIZE];

        let http_conn = unsafe {
            bindings::cupsConnectDest(
                dest_ptr,
                flags.into(),
                timeout,
                cancel_ptr,
                resource_buf.as_mut_ptr() as *mut i8,
                RESOURCE_SIZE,
                None, // No callback for now
                ptr::null_mut(), // No user data
            )
        };

        // Check for cancellation
        if let Some(cancel_flag) = cancel {
            if cancel_int != 0 {
                cancel_flag.store(true, Ordering::SeqCst);
            }
        }

        if http_conn.is_null() {
            return Err(Error::ConnectionFailed(format!(
                "Failed to connect to destination '{}'",
                self.name
            )));
        }

        // Convert resource buffer to string
        let resource_len = resource_buf.iter().position(|&x| x == 0).unwrap_or(0);
        let resource = String::from_utf8_lossy(&resource_buf[..resource_len]).into_owned();

        unsafe { HttpConnection::from_raw(http_conn, resource) }
    }

    /// Connect to this destination with a callback
    /// 
    /// Opens a connection with a callback function that can monitor the
    /// connection process and potentially cancel it.
    /// 
    /// # Arguments
    /// * `flags` - Whether to connect to scheduler or device directly
    /// * `timeout_ms` - Connection timeout in milliseconds, None for indefinite
    /// * `cancel` - Optional cancellation flag
    /// * `callback` - Callback function for connection monitoring
    /// * `user_data` - User data passed to callback
    /// 
    /// # Returns
    /// * `Ok(HttpConnection)` - Established connection
    /// * `Err(Error)` - Connection failed or was cancelled
    pub fn connect_with_callback<T>(
        &self,
        flags: ConnectionFlags,
        timeout_ms: Option<i32>,
        cancel: Option<&AtomicBool>,
        callback: &mut DestCallback<T>,
        user_data: &mut T,
    ) -> Result<HttpConnection> {
        // Create a raw cups_dest_t for this destination
        let dest_ptr = self.as_ptr();
        if dest_ptr.is_null() {
            return Err(Error::NullPointer);
        }

        let timeout = timeout_ms.unwrap_or(-1);
        let mut cancel_int: c_int = 0;
        let cancel_ptr = if cancel.is_some() {
            &mut cancel_int as *mut c_int
        } else {
            ptr::null_mut()
        };

        // Create callback context
        let mut context = ConnectContext {
            callback,
            user_data,
        };

        // Allocate resource buffer
        const RESOURCE_SIZE: usize = 1024;
        let mut resource_buf: Vec<u8> = vec![0; RESOURCE_SIZE];

        let http_conn = unsafe {
            bindings::cupsConnectDest(
                dest_ptr,
                flags.into(),
                timeout,
                cancel_ptr,
                resource_buf.as_mut_ptr() as *mut i8,
                RESOURCE_SIZE,
                Some(connect_dest_callback::<T>),
                &mut context as *mut _ as *mut c_void,
            )
        };

        // Check for cancellation
        if let Some(cancel_flag) = cancel {
            if cancel_int != 0 {
                cancel_flag.store(true, Ordering::SeqCst);
            }
        }

        if http_conn.is_null() {
            return Err(Error::ConnectionFailed(format!(
                "Failed to connect to destination '{}' or connection was cancelled",
                self.name
            )));
        }

        // Convert resource buffer to string
        let resource_len = resource_buf.iter().position(|&x| x == 0).unwrap_or(0);
        let resource = String::from_utf8_lossy(&resource_buf[..resource_len]).into_owned();

        unsafe { HttpConnection::from_raw(http_conn, resource) }
    }
}

// Context structure for the connection callback
struct ConnectContext<'a, T> {
    callback: &'a mut DestCallback<T>,
    user_data: &'a mut T,
}

// C-compatible callback function for connection monitoring
unsafe extern "C" fn connect_dest_callback<T>(
    user_data: *mut c_void,
    flags: u32,
    dest_ptr: *mut bindings::cups_dest_s,
) -> c_int {
    // Reconstruct our context
    let context = unsafe { &mut *(user_data as *mut ConnectContext<T>) };

    // Convert the raw destination to our Rust type
    unsafe {
        match Destination::from_raw(dest_ptr) {
            Ok(dest) => {
                // Call the user's callback
                if (context.callback)(flags, &dest, context.user_data) {
                    1 // Continue connection
                } else {
                    0 // Cancel connection
                }
            }
            Err(_) => {
                // Error parsing destination, but continue anyway
                1
            }
        }
    }
}

/// Connect to a destination
/// 
/// This is a convenience function that creates a connection to a destination.
/// 
/// # Arguments
/// * `dest` - Destination to connect to
/// * `flags` - Connection flags
/// * `timeout_ms` - Connection timeout in milliseconds, None for indefinite
/// * `cancel` - Optional cancellation flag
/// 
/// # Returns
/// * `Ok(HttpConnection)` - Established connection
/// * `Err(Error)` - Connection failed
pub fn connect_to_destination(
    dest: &Destination,
    flags: ConnectionFlags,
    timeout_ms: Option<i32>,
    cancel: Option<&AtomicBool>,
) -> Result<HttpConnection> {
    dest.connect(flags, timeout_ms, cancel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::destination::get_all_destinations;

    #[test]
    fn test_connection_flags() {
        assert_eq!(u32::from(ConnectionFlags::Scheduler), 0);
        assert_eq!(u32::from(ConnectionFlags::Device), 1);
    }

    #[test]
    fn test_connect_to_scheduler() {
        // This test requires a CUPS server to be running
        if let Ok(destinations) = get_all_destinations() {
            if let Some(dest) = destinations.first() {
                // Try to connect with a short timeout
                match dest.connect(ConnectionFlags::Scheduler, Some(1000), None) {
                    Ok(conn) => {
                        assert!(conn.is_connected());
                        assert!(!conn.resource_path().is_empty());
                        println!("Connected to '{}' with resource path: '{}'", 
                                dest.name, conn.resource_path());
                    }
                    Err(e) => {
                        // Connection might fail in test environment, that's OK
                        println!("Connection failed (expected in test): {}", e);
                    }
                }
            }
        }
    }
}