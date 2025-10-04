pub mod auth;
pub mod bindings;
pub mod config;
pub mod connection;
pub mod constants;
pub mod destination;
mod error;
mod error_helpers;
pub mod ipp;
pub mod job;
pub mod options;

pub use constants::*;
pub use connection::{ConnectionFlags, HttpConnection, connect_to_destination};
pub use destination::{
    Destination, DestinationInfo, Destinations, MediaSize, PrinterState, OptionConflict, copy_dest,
    enum_destinations, find_destinations, get_all_destinations, get_default_destination,
    get_destination, remove_dest,
};
pub use error::{Error, ErrorCategory, Result};
pub use job::{
    ColorMode, DuplexMode, JobInfo, JobStatus, Orientation, PrintOptions, PrintQuality,
    get_active_jobs, get_completed_jobs, *,
};
pub use ipp::{
    IppAttribute, IppOperation, IppRequest, IppResponse, IppStatus, IppTag, IppValueTag,
};
pub use options::{
    add_integer_option, add_option, encode_option, encode_options, encode_options_with_group,
    get_integer_option, get_option, parse_options, remove_option,
};
