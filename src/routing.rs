use axum::{extract::State, routing::{get, post}, Json, Router};
use rdkafka::producer::FutureRecord;

use crate::{errors::ApiError, kafka::KafkaMessage, state::AppState};

async fn health_check() -> &'static str {
    "OK"
}

pub async fn send_message(
    State(state): State<AppState>,
    Json(message): Json<KafkaMessage>,
) -> Result<&'static str, ApiError> {
    let msg = serde_json::to_vec(&message)?;
    let record: FutureRecord<str, Vec<u8>> = FutureRecord::to("messages").payload(&msg).key("1");

    state.producer().send_result(record)?.await??;

    tracing::info!("Message sent with data: {message:?}");

    Ok("Message sent!")
}

pub fn init_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/send", post(send_message))
        .with_state(state)
}