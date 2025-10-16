use crate::utils::KAFKA_BROKER_CONNECTION_STRING;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use serde::Serialize;
use std::{sync::Arc, time::Duration};

pub struct KafkaProduceAdapter {
    pub future_producer: Arc<FutureProducer>,
}

impl Default for KafkaProduceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl KafkaProduceAdapter {
    pub fn new() -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", KAFKA_BROKER_CONNECTION_STRING.as_str())
            .set("message.timeout.ms", "5000") //Timeout 5s
            .create()
            .expect("Failed to create producer");

        Self {
            future_producer: Arc::new(producer),
        }
    }

    pub async fn producer<T>(&self, topic: &str, key: &str, payload: &T)
    where
        T: Serialize,
    {
        let serialized_payload =
            serde_json::to_string(payload).map_err(|e| format!("Serialization error: {}", e));

        match serialized_payload {
            Ok(data) => {
                let result = self
                    .future_producer
                    .send(
                        FutureRecord::to(topic).key(key).payload(&data),
                        Duration::from_secs(0),
                    )
                    .await;

                match result {
                    Ok(delivery) => println!("Message delivered: {:?}", delivery),
                    Err((e, _)) => eprintln!("Failed to deliver message: {:?}", e),
                }
            }
            Err(_) => {
                println!("error roi fix di.")
            }
        };
    }
}
