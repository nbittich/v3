
use deadpool_lapin::{Config, CreatePoolError, Manager, Pool, Runtime};
use deadpool_lapin::lapin::{options::BasicPublishOptions, options::BasicConsumeOptions, BasicProperties, Consumer};
use deadpool_lapin::lapin::options::{BasicAckOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use futures_util::StreamExt;
use std::env::var;
use std::error::Error;
use deadpool::managed::PoolConfig;
use deadpool_lapin::lapin::publisher_confirm::{Confirmation, PublisherConfirm};
use domain::WithJsonProcessor;

const AMQP_HOST: &str = "AMQP_HOST";
const AMQP_PORT: &str = "AMQP_PORT";

struct Messenger {
    pool: Pool
}

impl Messenger {
    pub fn new() -> Result<Messenger, Box<dyn Error>> {
        let pool = Messenger::create_pool()?;
        Ok(Messenger {
            pool
        })
    }
    pub async fn subscribe(&self, queue: &str) -> Result<Consumer, Box<dyn Error>>{
        let connection = self.pool.clone().get().await?;
        let channel = connection.create_channel().await?;
        let _ = channel
            .queue_declare(
                queue,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let consumer = channel.basic_consume(queue, "", BasicConsumeOptions::default(), FieldTable::default()).await?;
        Ok(consumer)
    }

    pub async fn publish<'a>(&self, queue: &str, payload: &impl WithJsonProcessor<'a> ) -> Result<Confirmation, Box<dyn Error>> {
        let connection = self.pool.clone().get().await?;
        let channel = connection.create_channel().await?;
        let json = payload.to_json()?;
        let confirmation = channel.basic_publish(
            "",
            queue,
            BasicPublishOptions::default(),
            json.into_bytes(),
            BasicProperties::default(),
        ).await?.await?;
        Ok(confirmation)
    }

    fn create_pool() -> Result<Pool, CreatePoolError> {
        let amqp_host = var(AMQP_HOST).unwrap_or_else(|_| String::from("127.0.0.1"));
        let amqp_port = var(AMQP_PORT).unwrap_or_else(|_| String::from("5672"));
        let mut cfg = Config::default();
        cfg.pool = Some(PoolConfig::default());
        cfg.url = Some(format!("amqp://{amqp_host}:{amqp_port}"));
        cfg.create_pool(Some(Runtime::Tokio1))
    }

}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use futures_util::StreamExt;
    use domain::{Address, Profile, User};
    use crate::{BasicAckOptions, Messenger};

    #[tokio::test]
    async fn hello(){
        let queue = "oops";
        let messenger = Arc::new(Messenger::new().unwrap());
        let mut consumer = Arc::clone(&messenger).subscribe(queue).await.unwrap();

        let clone = Arc::clone(&messenger);
        let publisher_fut = tokio::task::spawn(async move {
            let user = create_user();
            let _ = clone.publish(queue,&user).await.unwrap();
        });

        let subscriber_fut = tokio::task::spawn(async move {
            while let Some(msg) = consumer.next().await {
                let (channel, delivery) = msg.expect("error in consumer");
                let data = &delivery.data;
                println!("{}", String::from_utf8_lossy(&data[..]));
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("ack");
                channel.close(100, "BYE").await.unwrap();
            }
        });
        let r = futures_util::future::join(publisher_fut, subscriber_fut).await;
        r.0.unwrap();
        r.1.unwrap();
    }

    fn create_user() -> User {
        let profile = Profile::new(
            Some(Default::default()),
            "nordine",
            "bittich",
            "(0032)0444/999.99.33",
            "nordine@keke.com",
            Address::new("pangaert", "20", "19", "Ganshoren", "Bxl", "Belgium"),
        );
         User::new("nickk", vec!["user", "admin"], "xxxx", profile)
    }
}