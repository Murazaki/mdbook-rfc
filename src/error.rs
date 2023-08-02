
use std::{error::Error, fmt};
use error_stack::{report, Result};    

#[derive(Debug)]
pub enum MdBookRFCError {
    Other
}

impl fmt::Display for MdBookRFCError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Error processing in mdbook-rfc: Could not handle the request.")
    }
}

impl Error for MdBookRFCError {}
