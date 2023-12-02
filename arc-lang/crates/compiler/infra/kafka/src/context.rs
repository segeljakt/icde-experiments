#![allow(unused)]

use std::time::Duration;

use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::producer::FutureProducer;
use rdkafka::util::Timeout;

pub struct Context {
    pub consumer: Option<StreamConsumer>,
    pub producer: Option<FutureProducer>,
    pub broker: String,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context").finish()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            consumer: None,
            producer: None,
            broker: "localhost:9092".to_string(),
        }
    }
}

impl Context {
    pub fn new(broker: Option<String>) -> Result<Self> {
        let broker = broker.unwrap_or("localhost:9092".to_string());
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker.clone())
            .create()?;
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", "kafka-cli")
            .set("bootstrap.servers", broker.clone())
            .create()?;
        let metadata =
            consumer.fetch_metadata(None, Timeout::After(Duration::from_millis(1000)))?;
        Ok(Self {
            consumer: Some(consumer),
            producer: Some(producer),
            broker,
        })
    }

    pub fn topics(&self) -> Result<()> {
        if let Some(consumer) = self.consumer.as_ref() {
            let metadata =
                consumer.fetch_metadata(None, Timeout::After(Duration::from_millis(1000)))?;

            eprintln!("Kafka Topics:");
            for topic in metadata.topics() {
                eprintln!(
                    "* {} {} partition(s)",
                    topic.name(),
                    topic.partitions().len()
                );
            }
        } else {
            eprintln!("Not connected to Kafka broker");
        }
        Ok(())
    }
}
