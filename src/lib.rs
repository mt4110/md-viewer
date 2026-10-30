//! Synthetic fixtures only. Reports persist the complete input locally.
pub mod engine;
pub mod fixture;
pub mod render;
pub mod report;

pub use fixture::load_suite;
pub use report::{build_report, evaluate_suite};

use std::fmt;
use std::path::PathBuf;

/// Fixed diagnostic codes, never parser excerpts or response data.
#[derive(Debug)]
pub struct LabError {
    pub code: &'static str,
    pub case_id: Option<String>,
    pub incomplete_output: Option<PathBuf>,
}

impl LabError {
    pub(crate) fn new(code: &'static str) -> Self {
        Self {
            code,
            case_id: None,
            incomplete_output: None,
        }
    }

    pub(crate) fn for_case(mut self, id: &str) -> Self {
        self.case_id = Some(id.to_owned());
        self
    }
}

impl fmt::Display for LabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error={}", self.code)?;
        if let Some(id) = &self.case_id {
            write!(f, " case={id}")?;
        }
        if let Some(path) = &self.incomplete_output {
            write!(f, " incomplete_output={path:?}")?;
        }
        Ok(())
    }
}

impl std::error::Error for LabError {}
