/// The error type for all Suggest component operations. These errors are
/// exposed to your application, which should handle them as needed.
use error_support::{ErrorHandling, GetErrorHandling};

/// A list of errors that are internal to the component. This is the error
/// type for private and crate-internal methods, and is never returned to the
/// application.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SearchApiError {
    #[error("Other error: {reason}")]
    Other { reason: String },
}

// Define how our internal errors are handled and converted to external errors
// See `support/error/README.md` for how this works, especially the warning about PII.
impl GetErrorHandling for Error {
    type ExternalError = SearchApiError;

    fn get_error_handling(&self) -> ErrorHandling<Self::ExternalError> {
        match self {
            _ => ErrorHandling::convert(SearchApiError::Other {
                reason: self.to_string(),
            })
            .report_error("search-unexpected"),
        }
    }
}
