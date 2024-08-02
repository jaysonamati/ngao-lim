mod state;
mod errors;
mod kafka;
mod routing;
mod queries;

// use axum::{routing::{get, post}, Router};
use routing::init_router;
use shuttle_service::{DeploymentMetadata, SecretStore};
use sqlx::PgPool;
use state::AppState;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] db: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_runtime::Metadata] metadata: DeploymentMetadata,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&db).await.unwrap();

    let con = match metadata.env {
        shuttle_service::Environment::Local => kafka::create_kafka_consumer(&secrets),
        shuttle_service::Environment::Deployment => kafka::create_kafka_consumer_upstash(&secrets),
    };

    tokio::spawn(async move {
        kafka::kafka_consumer_task(con, db).await;
    });

    let state = AppState::new(&secrets, &metadata);

    let router = init_router(state);

    Ok(router.into())
}
