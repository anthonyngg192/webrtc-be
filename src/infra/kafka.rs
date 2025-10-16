use std::sync::Arc;

use crate::utils::KAFKA_BROKER_CONNECTION_STRING;
use log::{info, warn};
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    ClientConfig, Message,
};

#[async_trait]
pub trait TopicHandler: Send + Sync {
    async fn handle(&self, payload: &str);
}

pub struct BaseKafkaConsumer {
    pub topic: String,
    pub handler: Arc<dyn TopicHandler + 'static>,
}

impl BaseKafkaConsumer {
    pub fn new(topic: String, handler: Arc<dyn TopicHandler + 'static>) -> Self {
        Self { topic, handler }
    }

    pub fn start_kafka_consumer(&self) {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &self.topic)
            .set("bootstrap.servers", KAFKA_BROKER_CONNECTION_STRING.as_str())
            .set("enable.partition.eof", "false")
            // .set("session.timeout.ms", "6000") // Update incase some messages need more time to handle.
            .set("enable.auto.commit", "false")
            .create()
            .expect("Failed to create Kafka consumer");

        consumer
            .subscribe(&[&self.topic])
            .expect("Failed to subscribe to topics");

        let handler = Arc::clone(&self.handler);

        actix::spawn(async move {
            loop {
                match consumer.recv().await {
                    Err(e) => warn!("Kafka error: {}", e),
                    Ok(m) => {
                        let payload = match m.payload_view::<str>() {
                            None => "",
                            Some(Ok(s)) => s,
                            Some(Err(e)) => {
                                warn!("Error while deserializing message payload: {:?}", e);
                                ""
                            }
                        };

                        let _ = handler.handle(payload).await;
                        info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                              m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                        consumer.commit_message(&m, CommitMode::Async).unwrap();
                    }
                };
            }
        });
    }
}
