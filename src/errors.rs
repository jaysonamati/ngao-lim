use axum::{http::StatusCode, response::{IntoResponse, Response}};

// Representing all the error messages we can get
// using an enum that uses the thiserror crate to easily derive error messages:
// TODO: Ideally this should be in a separate file
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("RDKafka error: {0}")]
    RDKafka(#[from] rdkafka::error::RDKafkaError),
    #[error("Kafka error: {0}")]
    Kafka(rdkafka::error::KafkaError),
    #[error("De/Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Oneshot message was cancelled")]
    CancelledMessage(#[from] futures::channel::oneshot::Canceled)
}

/// Note that while three of our types use the #[from] attribute macro to quickly derive the From<T> implementation, 
/// converting a KafkaError into our enum variant is a little bit more tricky.
/// The methods that return this error will normally return the error as a tuple
/// containing both the error and the record where the error occurred. We can thus implement it like this:
impl<'a>
    From<(
        rdkafka::error::KafkaError,
        rdkafka::producer::FutureRecord<'a, str, std::vec::Vec<u8>>,
    )> for ApiError
{
    fn from(
        e: (
            rdkafka::error::KafkaError,
            rdkafka::producer::FutureRecord<'a, str, std::vec::Vec<u8>>,
        ),
    ) -> Self {
        Self::Kafka(e.0)
    }
}

impl From<(rdkafka::error::KafkaError, rdkafka::message::OwnedMessage)> for ApiError {
    fn from(e: (rdkafka::error::KafkaError, rdkafka::message::OwnedMessage)) -> Self {
        Self::Kafka(e.0)
    }
}

/// To use this error type with our Axum service, we need to implement the IntoResponse
/// trait. This trait specifically represents a type that can be turned into a HTTP response.
/// We can do so by pattern matching the enum like so:
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::RDKafka(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Kafka(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::SerdeJson(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ApiError::CancelledMessage(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };
        (status,body).into_response()
    }
}
