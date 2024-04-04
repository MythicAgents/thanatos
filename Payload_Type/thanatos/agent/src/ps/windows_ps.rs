//! This module is only imported when targeting Windows hosts
use crate::ps::ProcessListingEntry;
use std::error::Error;
use std::result::Result;

/// Get the list of process information
pub fn process_info() -> Result<Vec<ProcessListingEntry>, Box<dyn Error>> {
    Ok(Vec::new())
}
