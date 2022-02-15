use deadpool_lapin::lapin::options::{
    BasicAckOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use deadpool_lapin::lapin::publisher_confirm::Confirmation;
use deadpool_lapin::lapin::types::FieldTable;

use deadpool_lapin::lapin::{
    options::BasicConsumeOptions, options::BasicPublishOptions, BasicProperties, Consumer,
    ExchangeKind,
};
use deadpool_lapin::{Config, CreatePoolError, Pool, Runtime};
use domain::{Id, OffsetDateTime, WithJsonProcessor};
use futures_util::{Future, StreamExt};

use std::borrow::Borrow;
use std::env::var;
use std::error::Error;
use std::sync::Arc;

const AMQP_HOST: &str = "AMQP_HOST";
const AMQP_PORT: &str = "AMQP_PORT";
#[derive(Debug)]
pub struct Messenger {
    pool: Pool,
    exchange: String,
    application_name: String,
}
#[derive(
    PartialOrd, PartialEq, Debug, domain::Serialize, domain::Deserialize, domain::WithJsonProcessor,
)]
pub struct Message {
    id: Id,
    creation_date: OffsetDateTime,
    sender: String,
    payload: Vec<u8>,
}

impl Message {
    pub fn id(&self) -> &Id {
        &self.id
    }
    pub fn creation_date(&self) -> OffsetDateTime {
        self.creation_date
    }
    pub fn sender(&self) -> &str {
        &self.sender
    }
    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }
    pub fn payload_as_string(&self) -> String {
        String::from_utf8_lossy(&self.payload[..]).into_owned()
    }
}

impl Messenger {
    pub async fn new(exchange: &str, application_name: &str) -> Result<Messenger, Box<dyn Error>> {
        let pool = Messenger::create_pool()?;
        let _ = Messenger::declare_exchange(&pool, exchange).await?;
        Ok(Messenger {
            pool,
            exchange: String::from(exchange),
            application_name: String::from(application_name),
        })
    }

    async fn declare_exchange(pool: &Pool, exchange: &str) -> Result<(), Box<dyn Error>> {
        let conn = pool.clone().get().await?;
        let channel = conn.create_channel().await?;
        channel
            .exchange_declare(
                exchange,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        Ok(())
    }

    pub async fn subscribe(&self, routing_key: &str) -> Result<Consumer, Box<dyn Error>> {
        let connection = self.pool.clone().get().await?;
        let channel = connection.create_channel().await?;
        let q = format!("{}_{routing_key}", self.application_name);
        let _ = channel
            .queue_declare(
                &q,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        let _ = channel
            .queue_bind(
                &q,
                &self.exchange,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let consumer = channel
            .basic_consume(
                &q,
                self.application_name.as_str(),
                BasicConsumeOptions {
                    no_local: true,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        Ok(consumer)
    }

    pub async fn publish<'a>(
        &self,
        routing_key: &str,
        payload: &impl WithJsonProcessor<'a>,
    ) -> Result<Confirmation, Box<dyn Error>> {
        let connection = self.pool.clone().get().await?;
        let channel = connection.create_channel().await?;
        let payload = payload.to_json()?;

        let message = Message {
            creation_date: OffsetDateTime::now_utc(),
            id: Id::default(),
            sender: String::from(&self.application_name),
            payload: payload.into_bytes(),
        };
        let message = message.to_json()?;

        let confirmation = channel
            .basic_publish(
                &self.exchange,
                routing_key,
                BasicPublishOptions::default(),
                message.into_bytes(),
                BasicProperties::default(),
            )
            .await?
            .await?;
        Ok(confirmation)
    }

    fn create_pool() -> Result<Pool, CreatePoolError> {
        let amqp_host = var(AMQP_HOST).unwrap_or_else(|_| String::from("127.0.0.1"));
        let amqp_port = var(AMQP_PORT).unwrap_or_else(|_| String::from("5672"));
        let cfg = Config {
            url: Some(format!("amqp://{amqp_host}:{amqp_port}")),
            ..Default::default()
        };
        cfg.create_pool(Some(Runtime::Tokio1))
    }
}

pub async fn consume_and_ack<F, Fut>(
    messenger: Arc<Messenger>,
    routing_key: String,
    on_message: F,
    close_after_message: Option<u32>, // close after x message
) -> Result<(), Box<dyn Error>>
where
    F: Fn(Message) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn Error>>>,
{
    let mut consumer = messenger.subscribe(&routing_key).await?;
    let mut count = 0;
    while let Some(msg) = consumer.next().await {
        let (channel, delivery) = msg.expect("error in consumer");
        let data = &delivery.data;
        let msg = Message::from_json(String::from_utf8_lossy(&data[..]).borrow())?;
        let _ = on_message(msg).await?;
        let _ = delivery.ack(BasicAckOptions::default()).await?;
        count += 1;
        if let Some(nb_message_before_close) = close_after_message {
            if count == nb_message_before_close {
                channel.close(100, "BYE").await?;
                return Ok(());
            }
        }
    }
    Ok(())
}
