pub mod bindings;
pub mod constants;
pub mod destination;
mod error;
pub mod job;

pub use constants::*;
pub use destination::{
    Destination, DestinationInfo, Destinations, MediaSize, PrinterState, copy_dest,
    enum_destinations, find_destinations, get_all_destinations, get_default_destination,
    get_destination, remove_dest,
};
pub use error::{Error, Result};
pub use job::{*, JobStatus, JobInfo, get_active_jobs, get_completed_jobs};