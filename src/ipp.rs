//! Low-level IPP (Internet Printing Protocol) request and response handling
//!
//! This module provides type-safe wrappers around CUPS IPP functions for building
//! and sending custom IPP requests. It's useful for advanced use cases that aren't
//! covered by the higher-level destination and job APIs.
//!
//! # Examples
//!
//! ## Creating and Sending an IPP Request
//!
//! ```no_run
//! use cups_rs::{IppRequest, IppOperation, IppTag, IppValueTag, ConnectionFlags, get_default_destination};
//!
//! let printer = get_default_destination().expect("No default printer");
//! let connection = printer.connect(ConnectionFlags::Scheduler, Some(5000), None)
//!     .expect("Failed to connect");
//!
//! let mut request = IppRequest::new(IppOperation::GetPrinterAttributes)
//!     .expect("Failed to create request");
//!
//! request.add_string(IppTag::Operation, IppValueTag::Uri,
//!                   "printer-uri", "ipp://localhost/printers/default")
//!     .expect("Failed to add attribute");
//!
//! let response = request.send(&connection, connection.resource_path())
//!     .expect("Failed to send request");
//!
//! if response.is_successful() {
//!     println!("Request successful!");
//! }
//! ```

use crate::bindings;
use crate::connection::HttpConnection;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr;

/// IPP attribute group tags
///
/// These tags define which group an IPP attribute belongs to in an IPP message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IppTag {
    Zero,
    Operation,
    Job,
    Printer,
    Subscription,
    EventNotification,
    Document,
    UnsupportedGroup,
}

impl From<IppTag> for bindings::ipp_tag_t {
    fn from(tag: IppTag) -> bindings::ipp_tag_t {
        match tag {
            IppTag::Zero => bindings::ipp_tag_e_IPP_TAG_ZERO,
            IppTag::Operation => bindings::ipp_tag_e_IPP_TAG_OPERATION,
            IppTag::Job => bindings::ipp_tag_e_IPP_TAG_JOB,
            IppTag::Printer => bindings::ipp_tag_e_IPP_TAG_PRINTER,
            IppTag::Subscription => bindings::ipp_tag_e_IPP_TAG_SUBSCRIPTION,
            IppTag::EventNotification => bindings::ipp_tag_e_IPP_TAG_EVENT_NOTIFICATION,
            IppTag::Document => bindings::ipp_tag_e_IPP_TAG_DOCUMENT,
            IppTag::UnsupportedGroup => bindings::ipp_tag_e_IPP_TAG_UNSUPPORTED_GROUP,
        }
    }
}

/// IPP value tags
///
/// These tags define the type of value an IPP attribute contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IppValueTag {
    Integer,
    Boolean,
    Enum,
    String,
    Text,
    Name,
    Keyword,
    Uri,
    Charset,
    Language,
    MimeType,
}

impl From<IppValueTag> for bindings::ipp_tag_t {
    fn from(tag: IppValueTag) -> bindings::ipp_tag_t {
        match tag {
            IppValueTag::Integer => bindings::ipp_tag_e_IPP_TAG_INTEGER,
            IppValueTag::Boolean => bindings::ipp_tag_e_IPP_TAG_BOOLEAN,
            IppValueTag::Enum => bindings::ipp_tag_e_IPP_TAG_ENUM,
            IppValueTag::String => bindings::ipp_tag_e_IPP_TAG_STRING,
            IppValueTag::Text => bindings::ipp_tag_e_IPP_TAG_TEXT,
            IppValueTag::Name => bindings::ipp_tag_e_IPP_TAG_NAME,
            IppValueTag::Keyword => bindings::ipp_tag_e_IPP_TAG_KEYWORD,
            IppValueTag::Uri => bindings::ipp_tag_e_IPP_TAG_URI,
            IppValueTag::Charset => bindings::ipp_tag_e_IPP_TAG_CHARSET,
            IppValueTag::Language => bindings::ipp_tag_e_IPP_TAG_LANGUAGE,
            IppValueTag::MimeType => bindings::ipp_tag_e_IPP_TAG_MIMETYPE,
        }
    }
}

/// IPP operation codes
///
/// These codes identify the operation being performed in an IPP request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IppOperation {
    PrintJob,
    ValidateJob,
    CreateJob,
    SendDocument,
    CancelJob,
    GetJobAttributes,
    GetJobs,
    GetPrinterAttributes,
    PausePrinter,
    ResumePrinter,
}

impl From<IppOperation> for bindings::ipp_op_t {
    fn from(op: IppOperation) -> bindings::ipp_op_t {
        match op {
            IppOperation::PrintJob => bindings::ipp_op_e_IPP_OP_PRINT_JOB,
            IppOperation::ValidateJob => bindings::ipp_op_e_IPP_OP_VALIDATE_JOB,
            IppOperation::CreateJob => bindings::ipp_op_e_IPP_OP_CREATE_JOB,
            IppOperation::SendDocument => bindings::ipp_op_e_IPP_OP_SEND_DOCUMENT,
            IppOperation::CancelJob => bindings::ipp_op_e_IPP_OP_CANCEL_JOB,
            IppOperation::GetJobAttributes => bindings::ipp_op_e_IPP_OP_GET_JOB_ATTRIBUTES,
            IppOperation::GetJobs => bindings::ipp_op_e_IPP_OP_GET_JOBS,
            IppOperation::GetPrinterAttributes => bindings::ipp_op_e_IPP_OP_GET_PRINTER_ATTRIBUTES,
            IppOperation::PausePrinter => bindings::ipp_op_e_IPP_OP_PAUSE_PRINTER,
            IppOperation::ResumePrinter => bindings::ipp_op_e_IPP_OP_RESUME_PRINTER,
        }
    }
}

/// IPP status codes
///
/// These codes indicate the result of an IPP operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IppStatus {
    Ok,
    OkIgnoredOrSubstituted,
    OkConflicting,
    ErrorBadRequest,
    ErrorForbidden,
    ErrorNotAuthenticated,
    ErrorNotAuthorized,
    ErrorNotPossible,
    ErrorTimeout,
    ErrorNotFound,
    ErrorGone,
    ErrorRequestEntity,
    ErrorRequestValue,
    ErrorDocumentFormatNotSupported,
    ErrorConflicting,
    ErrorPrinterIsDeactivated,
    ErrorTooManyJobs,
    ErrorInternalError,
}

impl IppStatus {
    pub fn from_code(code: bindings::ipp_status_t) -> Self {
        match code {
            bindings::ipp_status_e_IPP_STATUS_OK => IppStatus::Ok,
            bindings::ipp_status_e_IPP_STATUS_OK_IGNORED_OR_SUBSTITUTED => {
                IppStatus::OkIgnoredOrSubstituted
            }
            bindings::ipp_status_e_IPP_STATUS_OK_CONFLICTING => {
                IppStatus::OkConflicting
            }
            bindings::ipp_status_e_IPP_STATUS_ERROR_BAD_REQUEST => IppStatus::ErrorBadRequest,
            bindings::ipp_status_e_IPP_STATUS_ERROR_FORBIDDEN => IppStatus::ErrorForbidden,
            bindings::ipp_status_e_IPP_STATUS_ERROR_NOT_AUTHENTICATED => {
                IppStatus::ErrorNotAuthenticated
            }
            bindings::ipp_status_e_IPP_STATUS_ERROR_NOT_AUTHORIZED => {
                IppStatus::ErrorNotAuthorized
            }
            bindings::ipp_status_e_IPP_STATUS_ERROR_NOT_POSSIBLE => IppStatus::ErrorNotPossible,
            bindings::ipp_status_e_IPP_STATUS_ERROR_TIMEOUT => IppStatus::ErrorTimeout,
            bindings::ipp_status_e_IPP_STATUS_ERROR_NOT_FOUND => IppStatus::ErrorNotFound,
            bindings::ipp_status_e_IPP_STATUS_ERROR_GONE => IppStatus::ErrorGone,
            bindings::ipp_status_e_IPP_STATUS_ERROR_REQUEST_ENTITY => {
                IppStatus::ErrorRequestEntity
            }
            bindings::ipp_status_e_IPP_STATUS_ERROR_REQUEST_VALUE => IppStatus::ErrorRequestValue,
            bindings::ipp_status_e_IPP_STATUS_ERROR_DOCUMENT_FORMAT_NOT_SUPPORTED => {
                IppStatus::ErrorDocumentFormatNotSupported
            }
            bindings::ipp_status_e_IPP_STATUS_ERROR_CONFLICTING => IppStatus::ErrorConflicting,
            bindings::ipp_status_e_IPP_STATUS_ERROR_PRINTER_IS_DEACTIVATED => {
                IppStatus::ErrorPrinterIsDeactivated
            }
            bindings::ipp_status_e_IPP_STATUS_ERROR_TOO_MANY_JOBS => IppStatus::ErrorTooManyJobs,
            bindings::ipp_status_e_IPP_STATUS_ERROR_INTERNAL => IppStatus::ErrorInternalError,
            _ => IppStatus::ErrorInternalError,
        }
    }

    pub fn is_successful(&self) -> bool {
        matches!(
            self,
            IppStatus::Ok | IppStatus::OkIgnoredOrSubstituted | IppStatus::OkConflicting
        )
    }
}

/// An IPP request message
///
/// Represents an IPP request that can be customized with attributes and sent to a CUPS server.
/// The request is automatically freed when dropped.
///
/// # Examples
///
/// ```no_run
/// use cups_rs::{IppRequest, IppOperation, IppTag, IppValueTag};
///
/// let mut request = IppRequest::new(IppOperation::GetPrinterAttributes)
///     .expect("Failed to create request");
///
/// request.add_string(IppTag::Operation, IppValueTag::Keyword,
///                   "requested-attributes", "printer-state")
///     .expect("Failed to add attribute");
/// ```
pub struct IppRequest {
    ipp: *mut bindings::_ipp_s,
    _phantom: PhantomData<bindings::_ipp_s>,
}

impl IppRequest {
    /// Create a new IPP request
    pub fn new(operation: IppOperation) -> Result<Self> {
        let ipp = unsafe { bindings::ippNewRequest(operation.into()) };

        if ipp.is_null() {
            return Err(Error::UnsupportedFeature(
                "Failed to create IPP request".to_string(),
            ));
        }

        Ok(IppRequest {
            ipp,
            _phantom: PhantomData,
        })
    }

    /// Get the raw pointer to the ipp_t structure
    pub fn as_ptr(&self) -> *mut bindings::_ipp_s {
        self.ipp
    }

    /// Add a string attribute
    pub fn add_string(
        &mut self,
        group: IppTag,
        value_tag: IppValueTag,
        name: &str,
        value: &str,
    ) -> Result<()> {
        let name_c = CString::new(name)?;
        let value_c = CString::new(value)?;

        let attr = unsafe {
            bindings::ippAddString(
                self.ipp,
                group.into(),
                value_tag.into(),
                name_c.as_ptr(),
                ptr::null(),
                value_c.as_ptr(),
            )
        };

        if attr.is_null() {
            Err(Error::UnsupportedFeature(format!(
                "Failed to add string attribute '{}'",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Add an integer attribute
    pub fn add_integer(
        &mut self,
        group: IppTag,
        value_tag: IppValueTag,
        name: &str,
        value: i32,
    ) -> Result<()> {
        let name_c = CString::new(name)?;

        let attr = unsafe {
            bindings::ippAddInteger(
                self.ipp,
                group.into(),
                value_tag.into(),
                name_c.as_ptr(),
                value,
            )
        };

        if attr.is_null() {
            Err(Error::UnsupportedFeature(format!(
                "Failed to add integer attribute '{}'",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Add a boolean attribute
    pub fn add_boolean(&mut self, group: IppTag, name: &str, value: bool) -> Result<()> {
        let name_c = CString::new(name)?;

        let attr = unsafe {
            bindings::ippAddBoolean(self.ipp, group.into(), name_c.as_ptr(), value as ::std::os::raw::c_char)
        };

        if attr.is_null() {
            Err(Error::UnsupportedFeature(format!(
                "Failed to add boolean attribute '{}'",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Add multiple string attributes
    pub fn add_strings(
        &mut self,
        group: IppTag,
        value_tag: IppValueTag,
        name: &str,
        values: &[&str],
    ) -> Result<()> {
        let name_c = CString::new(name)?;
        let values_c: Vec<CString> = values
            .iter()
            .map(|v| CString::new(*v).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;

        let values_ptrs: Vec<*const ::std::os::raw::c_char> = values_c.iter().map(|s| s.as_ptr()).collect();

        let attr = unsafe {
            bindings::ippAddStrings(
                self.ipp,
                group.into(),
                value_tag.into(),
                name_c.as_ptr(),
                values.len() as i32,
                ptr::null(),
                values_ptrs.as_ptr(),
            )
        };

        if attr.is_null() {
            Err(Error::UnsupportedFeature(format!(
                "Failed to add string array attribute '{}'",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Send this request and receive a response
    pub fn send(&self, connection: &HttpConnection, resource: &str) -> Result<IppResponse> {
        let resource_c = CString::new(resource)?;

        // Note: cupsDoRequest frees the request, so we need to create a copy
        // Create a new request with the same operation code as the original
        let operation = unsafe { bindings::ippGetOperation(self.ipp) };
        let request_copy = unsafe { bindings::ippNewRequest(operation) };
        if request_copy.is_null() {
            return Err(Error::UnsupportedFeature(
                "Failed to copy IPP request".to_string(),
            ));
        }

        unsafe {
            // Copy all attributes from the original request to the new one
            bindings::ippCopyAttributes(request_copy, self.ipp, 0, None, ptr::null_mut());
        }

        let response = unsafe {
            bindings::cupsDoRequest(connection.as_ptr(), request_copy, resource_c.as_ptr())
        };

        if response.is_null() {
            Err(Error::ServerError(
                "No response received from server".to_string(),
            ))
        } else {
            Ok(IppResponse {
                ipp: response,
                _phantom: PhantomData,
            })
        }
    }
}

impl Drop for IppRequest {
    fn drop(&mut self) {
        if !self.ipp.is_null() {
            unsafe {
                bindings::ippDelete(self.ipp);
            }
            self.ipp = ptr::null_mut();
        }
    }
}

/// An IPP response message
///
/// Represents the response from an IPP request. Contains status code and attributes
/// that can be queried. The response is automatically freed when dropped.
///
/// # Examples
///
/// ```no_run
/// # use cups_rs::{IppRequest, IppOperation, IppTag, ConnectionFlags, get_default_destination};
/// # let printer = get_default_destination().unwrap();
/// # let connection = printer.connect(ConnectionFlags::Scheduler, Some(5000), None).unwrap();
/// # let request = IppRequest::new(IppOperation::GetPrinterAttributes).unwrap();
/// let response = request.send(&connection, connection.resource_path()).unwrap();
///
/// if response.is_successful() {
///     if let Some(attr) = response.find_attribute("printer-state", Some(IppTag::Printer)) {
///         println!("Printer state: {:?}", attr.get_integer(0));
///     }
/// }
/// ```
pub struct IppResponse {
    ipp: *mut bindings::_ipp_s,
    _phantom: PhantomData<bindings::_ipp_s>,
}

impl IppResponse {
    /// Get the raw pointer to the ipp_t structure
    pub fn as_ptr(&self) -> *mut bindings::_ipp_s {
        self.ipp
    }

    /// Get the status code from the response
    pub fn status(&self) -> IppStatus {
        let status_code = unsafe { bindings::ippGetStatusCode(self.ipp) };
        IppStatus::from_code(status_code)
    }

    /// Check if the response indicates success
    pub fn is_successful(&self) -> bool {
        self.status().is_successful()
    }

    /// Find an attribute by name
    pub fn find_attribute(&self, name: &str, group: Option<IppTag>) -> Option<IppAttribute> {
        let name_c = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let group_tag = group.map(|g| g.into()).unwrap_or(bindings::ipp_tag_e_IPP_TAG_ZERO);

        let attr = unsafe {
            bindings::ippFindAttribute(self.ipp, name_c.as_ptr(), group_tag)
        };

        if attr.is_null() {
            None
        } else {
            Some(IppAttribute { attr })
        }
    }

    /// Get all attributes in the response
    pub fn attributes(&self) -> Vec<IppAttribute> {
        let mut attributes = Vec::new();
        let mut attr = unsafe { bindings::ippFirstAttribute(self.ipp) };

        while !attr.is_null() {
            attributes.push(IppAttribute { attr });
            attr = unsafe { bindings::ippNextAttribute(self.ipp) };
        }

        attributes
    }
}

impl Drop for IppResponse {
    fn drop(&mut self) {
        if !self.ipp.is_null() {
            unsafe {
                bindings::ippDelete(self.ipp);
            }
            self.ipp = ptr::null_mut();
        }
    }
}

/// An IPP attribute
///
/// Represents a single attribute from an IPP response. Attributes can contain
/// one or more values of various types (string, integer, boolean, etc.).
#[derive(Clone, Copy)]
pub struct IppAttribute {
    attr: *mut bindings::_ipp_attribute_s,
}

impl IppAttribute {
    /// Get the attribute name
    pub fn name(&self) -> Option<String> {
        unsafe {
            let name_ptr = bindings::ippGetName(self.attr);
            if name_ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(name_ptr).to_string_lossy().into_owned())
            }
        }
    }

    /// Get the number of values
    pub fn count(&self) -> usize {
        unsafe { bindings::ippGetCount(self.attr) as usize }
    }

    /// Get a string value
    pub fn get_string(&self, index: usize) -> Option<String> {
        unsafe {
            let value_ptr = bindings::ippGetString(self.attr, index as i32, ptr::null_mut());
            if value_ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(value_ptr).to_string_lossy().into_owned())
            }
        }
    }

    /// Get an integer value
    pub fn get_integer(&self, index: usize) -> i32 {
        unsafe { bindings::ippGetInteger(self.attr, index as i32) }
    }

    /// Get a boolean value
    pub fn get_boolean(&self, index: usize) -> bool {
        unsafe { bindings::ippGetBoolean(self.attr, index as i32) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipp_request_creation() {
        let request = IppRequest::new(IppOperation::GetPrinterAttributes);
        assert!(request.is_ok());
    }

    #[test]
    fn test_ipp_add_string() {
        let mut request = IppRequest::new(IppOperation::GetPrinterAttributes).unwrap();
        let result = request.add_string(
            IppTag::Operation,
            IppValueTag::Uri,
            "printer-uri",
            "ipp://localhost/printers/test",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_ipp_add_integer() {
        let mut request = IppRequest::new(IppOperation::GetJobs).unwrap();
        let result = request.add_integer(IppTag::Operation, IppValueTag::Integer, "limit", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ipp_add_boolean() {
        let mut request = IppRequest::new(IppOperation::GetJobs).unwrap();
        let result = request.add_boolean(IppTag::Operation, "my-jobs", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ipp_status() {
        assert!(IppStatus::Ok.is_successful());
        assert!(IppStatus::OkIgnoredOrSubstituted.is_successful());
        assert!(!IppStatus::ErrorBadRequest.is_successful());
        assert!(!IppStatus::ErrorNotFound.is_successful());
    }
}
