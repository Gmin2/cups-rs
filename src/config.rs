use crate::bindings;
use crate::error::Result;
use std::ffi::{CStr, CString};
use std::ptr;

/// Encryption modes for CUPS connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    /// Never use encryption
    Never,
    /// Use encryption if requested by the server
    IfRequested,
    /// Require encryption for all connections
    Required,
    /// Always use encryption
    Always,
}

impl From<bindings::http_encryption_e> for EncryptionMode {
    fn from(encryption: bindings::http_encryption_e) -> Self {
        match encryption {
            bindings::http_encryption_e_HTTP_ENCRYPTION_NEVER => EncryptionMode::Never,
            bindings::http_encryption_e_HTTP_ENCRYPTION_IF_REQUESTED => EncryptionMode::IfRequested,
            bindings::http_encryption_e_HTTP_ENCRYPTION_REQUIRED => EncryptionMode::Required,
            bindings::http_encryption_e_HTTP_ENCRYPTION_ALWAYS => EncryptionMode::Always,
            _ => EncryptionMode::IfRequested, // Default fallback
        }
    }
}

impl Into<bindings::http_encryption_e> for EncryptionMode {
    fn into(self) -> bindings::http_encryption_e {
        match self {
            EncryptionMode::Never => bindings::http_encryption_e_HTTP_ENCRYPTION_NEVER,
            EncryptionMode::IfRequested => bindings::http_encryption_e_HTTP_ENCRYPTION_IF_REQUESTED,
            EncryptionMode::Required => bindings::http_encryption_e_HTTP_ENCRYPTION_REQUIRED,
            EncryptionMode::Always => bindings::http_encryption_e_HTTP_ENCRYPTION_ALWAYS,
        }
    }
}

/// Get the current CUPS server hostname/address
/// 
/// Returns the hostname/address of the current server. This can be a
/// fully-qualified hostname, a numeric IPv4 or IPv6 address, or a domain
/// socket pathname.
/// 
/// Note: The current server is tracked separately for each thread.
/// 
/// # Returns
/// - Server hostname/address string
/// - Default server if none has been set
pub fn get_server() -> String {
    unsafe {
        let server_ptr = bindings::cupsServer();
        if server_ptr.is_null() {
            "localhost".to_string() // Default fallback
        } else {
            CStr::from_ptr(server_ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Set the default CUPS server name and port
/// 
/// The server string can be a fully-qualified hostname, a numeric IPv4 or IPv6
/// address, or a domain socket pathname. Hostnames and numeric IP addresses can
/// be optionally followed by a colon and port number to override the default
/// port 631, e.g. "hostname:8631".
/// 
/// Note: The current server is tracked separately for each thread.
/// 
/// # Arguments
/// - `server`: Server name/address, or None to restore default
/// 
/// # Examples
/// ```rust
/// use cups_rs::config::set_server;
/// 
/// // Set specific server
/// let result = set_server(Some("print-server.company.com"));
/// assert!(result.is_ok());
/// 
/// // Set server with custom port
/// let result = set_server(Some("192.168.1.100:8631"));
/// assert!(result.is_ok());
/// 
/// // Restore default server
/// let result = set_server(None);
/// assert!(result.is_ok());
/// ```
pub fn set_server(server: Option<&str>) -> Result<()> {
    match server {
        Some(s) => {
            let server_c = CString::new(s)?;
            unsafe {
                bindings::cupsSetServer(server_c.as_ptr());
            }
        }
        None => unsafe {
            bindings::cupsSetServer(ptr::null());
        },
    }
    Ok(())
}

/// Get the current user name
/// 
/// Returns the current user's name as used by CUPS for authentication
/// and job ownership.
/// 
/// Note: The current user name is tracked separately for each thread.
/// 
/// # Returns
/// - Current user name
/// - System username if none has been set
pub fn get_user() -> String {
    unsafe {
        let user_ptr = bindings::cupsUser();
        if user_ptr.is_null() {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "anonymous".to_string())
        } else {
            CStr::from_ptr(user_ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Set the default user name
/// 
/// Sets the user name to use for authentication and job ownership.
/// 
/// Note: The current user name is tracked separately for each thread.
/// 
/// # Arguments
/// - `user`: Username, or None to restore default
/// 
/// # Examples
/// ```rust
/// use cups_rs::config::set_user;
/// 
/// // Set specific user
/// let result = set_user(Some("john.doe"));
/// assert!(result.is_ok());
/// 
/// // Restore default user
/// let result = set_user(None);
/// assert!(result.is_ok());
/// ```
pub fn set_user(user: Option<&str>) -> Result<()> {
    match user {
        Some(u) => {
            let user_c = CString::new(u)?;
            unsafe {
                bindings::cupsSetUser(user_c.as_ptr());
            }
        }
        None => unsafe {
            bindings::cupsSetUser(ptr::null());
        },
    }
    Ok(())
}

/// Get the current encryption settings
/// 
/// Returns the current encryption preference for CUPS connections.
/// 
/// The default encryption setting comes from the CUPS_ENCRYPTION environment
/// variable, then the ~/.cups/client.conf file, and finally the
/// /etc/cups/client.conf file. If not set, the default is IfRequested.
/// 
/// Note: The current encryption setting is tracked separately for each thread.
/// 
/// # Returns
/// - Current encryption mode
pub fn get_encryption() -> EncryptionMode {
    unsafe {
        let encryption = bindings::cupsEncryption();
        EncryptionMode::from(encryption)
    }
}

/// Set the encryption preference
/// 
/// Sets the encryption preference for CUPS connections.
/// 
/// Note: The current encryption setting is tracked separately for each thread.
/// 
/// # Arguments
/// - `mode`: Encryption mode to use
/// 
/// # Examples
/// ```rust
/// use cups_rs::config::{set_encryption, EncryptionMode};
/// 
/// // Require encryption for all connections
/// set_encryption(EncryptionMode::Required);
/// 
/// // Use encryption only if requested
/// set_encryption(EncryptionMode::IfRequested);
/// ```
pub fn set_encryption(mode: EncryptionMode) {
    unsafe {
        bindings::cupsSetEncryption(mode.into());
    }
}

/// Get the current HTTP User-Agent string
/// 
/// Returns the User-Agent string used in HTTP requests to CUPS servers.
/// 
/// # Returns
/// - Current User-Agent string
/// - Default CUPS User-Agent if none has been set
pub fn get_user_agent() -> String {
    unsafe {
        let agent_ptr = bindings::cupsUserAgent();
        if agent_ptr.is_null() {
            format!("CUPS/2.4 (cups-rs/{})", env!("CARGO_PKG_VERSION"))
        } else {
            CStr::from_ptr(agent_ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Set the default HTTP User-Agent string
/// 
/// Sets the User-Agent string used in HTTP requests to CUPS servers.
/// This is useful for identifying your application in server logs.
/// 
/// # Arguments
/// - `user_agent`: User-Agent string, or None to restore default
/// 
/// # Examples
/// ```rust
/// use cups_rs::config::set_user_agent;
/// 
/// // Set custom User-Agent
/// let result = set_user_agent(Some("MyPrintApp/1.0"));
/// assert!(result.is_ok());
/// 
/// // Restore default User-Agent
/// let result = set_user_agent(None);
/// assert!(result.is_ok());
/// ```
pub fn set_user_agent(user_agent: Option<&str>) -> Result<()> {
    match user_agent {
        Some(agent) => {
            let agent_c = CString::new(agent)?;
            unsafe {
                bindings::cupsSetUserAgent(agent_c.as_ptr());
            }
        }
        None => unsafe {
            bindings::cupsSetUserAgent(ptr::null());
        },
    }
    Ok(())
}

/// Configuration manager for CUPS settings
/// 
/// This struct provides a convenient way to manage CUPS configuration
/// settings with automatic cleanup when dropped.
#[derive(Debug)]
pub struct CupsConfig {
    original_server: Option<String>,
    original_user: Option<String>,
    original_encryption: Option<EncryptionMode>,
    original_user_agent: Option<String>,
}

impl CupsConfig {
    /// Create a new configuration manager
    /// 
    /// This captures the current configuration state so it can be restored
    /// when the CupsConfig is dropped.
    pub fn new() -> Self {
        CupsConfig {
            original_server: Some(get_server()),
            original_user: Some(get_user()),
            original_encryption: Some(get_encryption()),
            original_user_agent: Some(get_user_agent()),
        }
    }

    /// Set the CUPS server
    pub fn with_server(self, server: &str) -> Result<Self> {
        set_server(Some(server))?;
        Ok(self)
    }

    /// Set the user name
    pub fn with_user(self, user: &str) -> Result<Self> {
        set_user(Some(user))?;
        Ok(self)
    }

    /// Set the encryption mode
    pub fn with_encryption(self, mode: EncryptionMode) -> Self {
        set_encryption(mode);
        self
    }

    /// Set the User-Agent string
    pub fn with_user_agent(self, user_agent: &str) -> Result<Self> {
        set_user_agent(Some(user_agent))?;
        Ok(self)
    }

    /// Get current configuration summary
    pub fn current_config(&self) -> ConfigSummary {
        ConfigSummary {
            server: get_server(),
            user: get_user(),
            encryption: get_encryption(),
            user_agent: get_user_agent(),
        }
    }
}

impl Default for CupsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CupsConfig {
    fn drop(&mut self) {
        // Restore original settings
        if let Some(server) = &self.original_server {
            let _ = set_server(Some(server));
        }
        if let Some(user) = &self.original_user {
            let _ = set_user(Some(user));
        }
        if let Some(encryption) = &self.original_encryption {
            set_encryption(*encryption);
        }
        if let Some(user_agent) = &self.original_user_agent {
            let _ = set_user_agent(Some(user_agent));
        }
    }
}

/// Summary of current CUPS configuration
#[derive(Debug, Clone)]
pub struct ConfigSummary {
    pub server: String,
    pub user: String,
    pub encryption: EncryptionMode,
    pub user_agent: String,
}

impl std::fmt::Display for ConfigSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server: {}, User: {}, Encryption: {:?}, User-Agent: {}",
            self.server, self.user, self.encryption, self.user_agent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_mode_conversion() {
        let modes = [
            EncryptionMode::Never,
            EncryptionMode::IfRequested,
            EncryptionMode::Required,
            EncryptionMode::Always,
        ];

        for mode in &modes {
            let cups_mode: bindings::http_encryption_e = (*mode).into();
            let converted_back = EncryptionMode::from(cups_mode);
            assert_eq!(*mode, converted_back);
        }
    }

    #[test]
    fn test_server_configuration() {
        let original_server = get_server();
        
        // Test setting custom server
        let test_server = "test.example.com:8631";
        set_server(Some(test_server)).unwrap();
        
        // CUPS might normalize the server name, so check if it contains our test server
        let current_server = get_server();
        assert!(current_server.contains("test.example.com"), 
                "Expected server to contain 'test.example.com', got: {}", current_server);
        
        // Test restoring default
        set_server(None).unwrap();
        // Note: The exact default may vary by system
        
        // Restore original for cleanliness
        set_server(Some(&original_server)).unwrap();
    }

    #[test]
    fn test_config_manager() {
        let original_server = get_server();
        
        {
            let _config = CupsConfig::new()
                .with_server("managed.example.com").unwrap()
                .with_encryption(EncryptionMode::Required);
            
            assert_eq!(get_server(), "managed.example.com");
            assert_eq!(get_encryption(), EncryptionMode::Required);
        }
        
        // Settings should be restored after config is dropped
        assert_eq!(get_server(), original_server);
    }
}