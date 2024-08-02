use rdkafka::{config::RDKafkaLogLevel, consumer::{CommitMode, Consumer, StreamConsumer}, producer::FutureProducer, ClientConfig, Message};
use serde::{Deserialize, Serialize};
use shuttle_service::SecretStore;

use crate::queries;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMessage {
    name: String,
    message: String,
}

impl<'a> CustomMessage {
    pub fn name(&'a self) -> &'a str {
        &self.name
    }

    pub fn message(&'a self) -> &'a str {
        &self.message
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KafkaMessage {
    action: Action,
    message_id: i32,
    data: Option<CustomMessage>
}

impl<'a> KafkaMessage {
    pub fn data(&'a self) -> &'a CustomMessage {
        self.data.as_ref().unwrap()
    }

    pub fn message_id(&'a self) -> &'a i32 {
        &self.message_id
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Action {
    Create,
    Update,
    Delete
}

pub fn create_kafka_producer_upstash(secrets: &SecretStore) -> FutureProducer {
    let url = secrets.get("KAFKA_URL").unwrap();
    let user = secrets.get("KAFKA_SASL_USER").unwrap();
    let pw = secrets.get("KAFKA_SASL_PASS").unwrap();

    ClientConfig::new()
        .set("bootstrap.servers", url)
        .set("sasl.mechanism", "SCRAM-SHA-256")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.username", user)
        .set("sasl.password", pw)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}

pub fn create_kafka_consumer_upstash(secrets: &SecretStore) -> StreamConsumer {
    let url = secrets.get("KAFKA_URL").unwrap();
    let user = secrets.get("KAFKA_SASL_USER").unwrap();
    let pw = secrets.get("KAFKA_SASL_PASS").unwrap();

    ClientConfig::new()
        .set("bootstrap.servers", url)
        .set("sasl.mechanism", "SCRAM-SHA-256")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.username", user)
        .set("sasl.password", pw)
        .set("group.id", "hello")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed")
}

/// To get started with `rdkafka`, we need to create a publisher and a consumer. We can do
/// this with these two functions:
pub fn create_kafka_producer(secrets: &SecretStore) -> FutureProducer {
    let url = secrets.get("KAFKA_URL").unwrap();

    let log_level: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", url)
        .set("message.timeout.ms", "5000")
        .set("allow.auto.create.topics", "true")
        .create()
        .expect("Producer creation error");

    log_level
}

pub fn create_kafka_consumer(secrets: &SecretStore) -> StreamConsumer {
    let url = secrets.get("KAFKA_URL").unwrap();

    ClientConfig::new()
        .set("group.id", "ngao-lim")
        .set("bootstrap.servers", url)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        // only store offset from the consumer
        .set("enable.auto.offset.store", "false")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed")
}

/*
You may note that above, we’ve enabled auto commit while only enabling storing offset from the consumer. The reason for this is that it allows us to rely on the underlying Kafka logic to commit regularly while only allowing the consumer to commit a message after it’s been fully processed. This enables us to prevent any loss of messages! This is also called At Least Once delivery.

Note that the producer has permissions to automatically create topics. In production, you may want to remove this and create topics manually. Allowing a producer to create topics freely may result in some unexpected behaviour!

Additionally, some hosted Kafka services will require SASL or SSL authentication. You can find more about the dependencies in the rdkafka-rust repo here. Note that if you are unable to install dependencies, rdkafka also has feature flags for vendored versions of the required dependencies.
 */

/// Next, the important part: receiving our messages! As a basic example, we will spawn a Tokio task to handle this.
/// Here is a short example of how we can use a StreamConsumer to subscribe to a channel,
/// then loop while waiting for the message stream to receive a message:

#[tracing::instrument(skip_all)]
pub async fn kafka_consumer_task(con: StreamConsumer, db: sqlx::PgPool) {
    con.subscribe(&["messages"]).expect("Failed to subscribe to topics");

    tracing::info!("Starting the consumer loop...");

    loop {
        match con.recv().await {
            Err(e) => tracing::warn!("Kafka error: {}", e),
            Ok(m) => {
                let Some(payload) = m.payload() else {
                    tracing::error!("Could not find a payload :(");
                    continue;
                };

                // here we use `from_slice()` as we initially send it as &[u8]
                let message: KafkaMessage = match serde_json::from_slice(payload) {
                    Ok(res) => res,
                    Err(e) => {
                        // If there is a deserialization error print an error
                        // and go to the next loop iteration
                        tracing::error!("Deserialization error: {e}");
                        continue;
                    },
                };

                // Print out our payload
                tracing::info!("Got payload: {message:?}");

                // Match action with db operations
                match message.action {
                    Action::Create => queries::create_message(message, &db).await,
                    Action::Update => queries::update_message(message, &db).await,
                    Action::Delete => queries::delete_message(message, &db).await,
                }

                // let _ = con
                //     .store_offset_from_message(&m)
                //     .inspect_err(|e| tracing::warn!("Error while storing offset: {}", e));
                con.commit_message(&m, CommitMode::Async).unwrap();

            }
        }
    }
}