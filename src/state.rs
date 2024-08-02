use rdkafka::producer::FutureProducer;
use shuttle_service::{DeploymentMetadata, Environment, SecretStore};

use crate::kafka;


/// Next, we’ll want to create our AppState 
/// which will hold the FutureProducer created by the create_kafka_producer function:

#[derive(Clone)]
pub struct AppState {
    kafka_producer: FutureProducer,
}


impl AppState {
    pub fn new(secrets: &SecretStore, metadata: &DeploymentMetadata) -> Self {
        let kafka_producer = match metadata.env {
            Environment::Local => kafka::create_kafka_producer(secrets),
            Environment::Deployment => kafka::create_kafka_producer_upstash(secrets),
        };

        Self { kafka_producer }
    }
}

impl<'a> AppState {
    pub fn producer(&'a self) -> &'a FutureProducer {
        &self.kafka_producer
    }
}