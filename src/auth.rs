use crate::bindings;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;

/// Password callback function type
/// 
/// This callback is called when CUPS needs authentication credentials.
/// 
/// # Parameters
/// - `prompt`: The authentication prompt string
/// - `http_connection`: Optional HTTP connection (None for CUPS_HTTP_DEFAULT)
/// - `method`: HTTP method ("GET", "POST", "PUT", etc.)
/// - `resource`: The resource path being accessed
/// 
/// # Returns
/// - `Some(String)`: The password to use for authentication
/// - `None`: Cancel authentication
pub type PasswordCallback = dyn Fn(&str, Option<&str>, &str, &str) -> Option<String> + Send + Sync;

/// Client certificate callback function type
/// 
/// This callback is called when CUPS needs a client certificate for authentication.
/// 
/// # Parameters
/// - `server_name`: The server name requiring the certificate
/// 
/// # Returns
/// - `Some(Vec<u8>)`: The certificate data in DER format
/// - `None`: No certificate available
pub type ClientCertCallback = dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync;

/// Server certificate validation callback function type
/// 
/// This callback is called to validate server certificates.
/// 
/// # Parameters
/// - `server_name`: The server name
/// - `certificate`: The server certificate data in DER format
/// 
/// # Returns
/// - `true`: Accept the certificate
/// - `false`: Reject the certificate
pub type ServerCertCallback = dyn Fn(&str, &[u8]) -> bool + Send + Sync;

// Thread-local storage for authentication callbacks
thread_local! {
    static PASSWORD_CALLBACK: std::cell::RefCell<Option<Arc<PasswordCallback>>> = 
        const { std::cell::RefCell::new(None) };
    static CLIENT_CERT_CALLBACK: std::cell::RefCell<Option<Arc<ClientCertCallback>>> = 
        const { std::cell::RefCell::new(None) };
    static SERVER_CERT_CALLBACK: std::cell::RefCell<Option<Arc<ServerCertCallback>>> = 
        const { std::cell::RefCell::new(None) };
}

/// Set a password callback for GUI applications
/// 
/// This function sets a password callback that will be called whenever
/// CUPS needs authentication credentials. The callback should prompt
/// the user for a password and return it.
/// 
/// Pass `None` to restore the default console-based authentication.
/// 
/// # Arguments
/// - `callback`: The password callback function, or None to restore default
/// 
/// # Example
/// ```rust
/// use cups_rs::auth::set_password_callback;
/// 
/// let result = set_password_callback(Some(Box::new(|prompt, _http, _method, _resource| {
///     println!("Authentication required: {}", prompt);
///     // In a real GUI app, show a password dialog here
///     Some("user_password".to_string())
/// })));
/// assert!(result.is_ok());
/// ```
pub fn set_password_callback(callback: Option<Box<PasswordCallback>>) -> Result<()> {
    let has_callback = callback.is_some();
    
    PASSWORD_CALLBACK.with(|cb| {
        *cb.borrow_mut() = callback.map(|c| Arc::from(c));
    });

    // Set the C callback function
    unsafe {
        if has_callback {
            bindings::cupsSetPasswordCB2(Some(password_callback_wrapper), ptr::null_mut());
        } else {
            bindings::cupsSetPasswordCB2(None, ptr::null_mut());
        }
    }

    Ok(())
}

/// Set a client certificate callback for SSL/TLS authentication
/// 
/// This function sets a callback that will be called when CUPS needs
/// a client certificate for SSL/TLS authentication.
/// 
/// Pass `None` to remove the current callback.
/// 
/// # Arguments
/// - `callback`: The client certificate callback function, or None to remove
/// 
/// # Example
/// ```rust
/// use cups_rs::auth::set_client_cert_callback;
/// 
/// let result = set_client_cert_callback(Some(Box::new(|server_name| {
///     println!("Certificate required for: {}", server_name);
///     // In a real app, load certificate from file or keystore
///     Some(vec![1, 2, 3]) // Mock certificate data
/// })));
/// assert!(result.is_ok());
/// ```
pub fn set_client_cert_callback(callback: Option<Box<ClientCertCallback>>) -> Result<()> {
    CLIENT_CERT_CALLBACK.with(|cb| {
        *cb.borrow_mut() = callback.map(|c| Arc::from(c));
    });

    // Note: cupsSetClientCertCB might not be available in all CUPS versions
    // This is a placeholder for when the binding is available
    
    Ok(())
}

/// Set a server certificate validation callback
/// 
/// This function sets a callback that will be called to validate
/// server certificates during SSL/TLS connections.
/// 
/// Pass `None` to use default certificate validation.
/// 
/// # Arguments
/// - `callback`: The server certificate validation callback, or None for default
/// 
/// # Example
/// ```rust
/// use cups_rs::auth::set_server_cert_callback;
/// 
/// let result = set_server_cert_callback(Some(Box::new(|server_name, cert_data| {
///     println!("Validating certificate for: {}", server_name);
///     println!("Certificate size: {} bytes", cert_data.len());
///     // In a real app, validate the certificate properly
///     true // Accept all certificates (NOT recommended for production)
/// })));
/// assert!(result.is_ok());
/// ```
pub fn set_server_cert_callback(callback: Option<Box<ServerCertCallback>>) -> Result<()> {
    SERVER_CERT_CALLBACK.with(|cb| {
        *cb.borrow_mut() = callback.map(|c| Arc::from(c));
    });

    // Note: cupsSetServerCertCB might not be available in all CUPS versions
    // This is a placeholder for when the binding is available
    
    Ok(())
}

/// Get a password using the current password callback
/// 
/// This function calls the current password callback to get a password
/// for authentication. It's typically used internally by CUPS.
/// 
/// # Arguments
/// - `prompt`: The authentication prompt
/// - `http`: Optional HTTP connection
/// - `method`: HTTP method being used
/// - `resource`: The resource being accessed
/// 
/// # Returns
/// - `Some(String)`: The password provided by the callback
/// - `None`: No password callback set or user cancelled
pub fn get_password(
    prompt: &str,
    http: Option<&str>,
    method: &str,
    resource: &str,
) -> Option<String> {
    PASSWORD_CALLBACK.with(|cb| {
        let callback_ref = cb.borrow();
        if let Some(callback) = callback_ref.as_ref() {
            callback(prompt, http, method, resource)
        } else {
            None
        }
    })
}

/// Get a client certificate using the current callback
/// 
/// This function calls the current client certificate callback to get
/// a certificate for SSL/TLS authentication.
/// 
/// # Arguments
/// - `server_name`: The server name requiring the certificate
/// 
/// # Returns
/// - `Some(Vec<u8>)`: The certificate data in DER format
/// - `None`: No certificate callback set or no certificate available
pub fn get_client_certificate(server_name: &str) -> Option<Vec<u8>> {
    CLIENT_CERT_CALLBACK.with(|cb| {
        let callback_ref = cb.borrow();
        if let Some(callback) = callback_ref.as_ref() {
            callback(server_name)
        } else {
            None
        }
    })
}

/// Validate a server certificate using the current callback
/// 
/// This function calls the current server certificate validation callback
/// to validate a server certificate.
/// 
/// # Arguments
/// - `server_name`: The server name
/// - `certificate`: The certificate data in DER format
/// 
/// # Returns
/// - `true`: Certificate is valid/accepted
/// - `false`: Certificate is invalid/rejected or no callback set
pub fn validate_server_certificate(server_name: &str, certificate: &[u8]) -> bool {
    SERVER_CERT_CALLBACK.with(|cb| {
        let callback_ref = cb.borrow();
        if let Some(callback) = callback_ref.as_ref() {
            callback(server_name, certificate)
        } else {
            false // Default to reject if no callback
        }
    })
}

/// Perform authentication for an HTTP request
/// 
/// This function handles authentication for a specific HTTP request.
/// It will call the password callback if needed and set up the
/// appropriate authentication headers.
/// 
/// # Arguments
/// - `http_connection`: HTTP connection (use None for CUPS_HTTP_DEFAULT)
/// - `method`: HTTP method ("GET", "POST", "PUT", etc.)
/// - `resource`: The resource path
/// 
/// # Returns
/// - `Ok(())`: Authentication successful or not required
/// - `Err(Error)`: Authentication failed
pub fn do_authentication(
    _http_connection: Option<&str>,
    method: &str,
    resource: &str,
) -> Result<()> {
    let method_c = CString::new(method)?;
    let resource_c = CString::new(resource)?;

    let result = unsafe {
        bindings::cupsDoAuthentication(
            ptr::null_mut(), // Use CUPS_HTTP_DEFAULT for now
            method_c.as_ptr(),
            resource_c.as_ptr(),
        )
    };

    if result != 0 {
        Ok(())
    } else {
        Err(Error::AuthenticationFailed(format!(
            "Authentication failed for {} {}", method, resource
        )))
    }
}

/// Internal C callback wrapper for password callbacks
extern "C" fn password_callback_wrapper(
    prompt: *const c_char,
    _http: *mut bindings::_http_s,
    method: *const c_char,
    resource: *const c_char,
    _user_data: *mut std::os::raw::c_void,
) -> *const c_char {
    // Safety: We ensure these pointers are valid C strings from CUPS
    let prompt_str = if prompt.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(prompt).to_str().unwrap_or("") }
    };

    let method_str = if method.is_null() {
        "GET"
    } else {
        unsafe { CStr::from_ptr(method).to_str().unwrap_or("GET") }
    };

    let resource_str = if resource.is_null() {
        "/"
    } else {
        unsafe { CStr::from_ptr(resource).to_str().unwrap_or("/") }
    };

    // Get password from Rust callback
    let password = PASSWORD_CALLBACK.with(|cb| {
        let callback_ref = cb.borrow();
        if let Some(callback) = callback_ref.as_ref() {
            callback(prompt_str, None, method_str, resource_str)
        } else {
            None
        }
    });

    match password {
        Some(pwd) => {
            // Convert to C string and return
            // Note: This creates a memory leak, but CUPS expects the string to remain valid
            // until the next authentication call. This is how the CUPS API works.
            let c_string = CString::new(pwd).unwrap_or_else(|_| CString::new("").unwrap());
            let ptr = c_string.into_raw();
            ptr
        }
        None => ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_password_callback() {
        let result = set_password_callback(Some(Box::new(|_prompt, _http, _method, _resource| {
            Some("test_password".to_string())
        })));
        assert!(result.is_ok());

        // Test getting password
        let password = get_password("Enter password:", None, "GET", "/");
        assert_eq!(password, Some("test_password".to_string()));

        // Test removing callback
        let result = set_password_callback(None);
        assert!(result.is_ok());

        let password = get_password("Enter password:", None, "GET", "/");
        assert_eq!(password, None);
    }

    #[test]
    fn test_certificate_callbacks() {
        // Test client certificate callback
        let cert_data = vec![1, 2, 3, 4, 5];
        let cert_data_clone = cert_data.clone();
        
        let result = set_client_cert_callback(Some(Box::new(move |server_name| {
            if server_name == "test.example.com" {
                Some(cert_data_clone.clone())
            } else {
                None
            }
        })));
        assert!(result.is_ok());

        let certificate = get_client_certificate("test.example.com");
        assert_eq!(certificate, Some(cert_data));

        let no_certificate = get_client_certificate("other.example.com");
        assert_eq!(no_certificate, None);

        // Test server certificate validation callback
        let result = set_server_cert_callback(Some(Box::new(|server_name, cert_data| {
            server_name == "trusted.example.com" && !cert_data.is_empty()
        })));
        assert!(result.is_ok());

        let valid = validate_server_certificate("trusted.example.com", &[1, 2, 3]);
        assert!(valid);

        let invalid = validate_server_certificate("untrusted.example.com", &[1, 2, 3]);
        assert!(!invalid);

        let empty_cert = validate_server_certificate("trusted.example.com", &[]);
        assert!(!empty_cert);

        // Test removing callbacks
        let result = set_client_cert_callback(None);
        assert!(result.is_ok());
        
        let no_cert = get_client_certificate("test.example.com");
        assert_eq!(no_cert, None);

        let result = set_server_cert_callback(None);
        assert!(result.is_ok());
        
        let no_validation = validate_server_certificate("trusted.example.com", &[1, 2, 3]);
        assert!(!no_validation);
    }
}