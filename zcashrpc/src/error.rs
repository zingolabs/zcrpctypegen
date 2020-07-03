use crate::envelope::ResponseEnvelopeError;
use crate::json::JsonError;

/// An `Error<R>` represents errors encountered in making an RPC
/// request. The parameter `R` is for the response type which is specific
/// to each method, and only appears in the `ResponseEnvelopeError` case.
#[derive(derive_more::From, Debug)]
pub enum Error<R> {
    /// A `ResponseEnvelopeError<R>` represents either application-level
    /// error from `zcashd` or a failure to adhere to expected JSON RPC
    /// protocol semantics.
    Response(ResponseEnvelopeError<R>),

    /// A `Reqwest::Error` comes directly from the `reqwest` HTTP client
    /// dependency, and these errors represent HTTP-level errors.
    Reqwest(reqwest::Error),

    /// A `json::Error` represents a JSON deserialization error, which
    /// includes "structural" mismatches between what this client expects
    /// versus what the server replied with. For example, if this library
    /// expects an integer JSON value but finds a string, the error is
    /// captured with this case.
    Json(JsonError),
}
