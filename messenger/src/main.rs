use std::sync::Arc;

use deadpool_lapin::{Config, Manager, Pool, Runtime};
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    options::BasicConsumeOptions,
    BasicProperties,
};
use deadpool_lapin::lapin::options::{BasicAckOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::default();
    cfg.url = Some("amqp://127.0.0.1:5672/%2f".into());
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    let mut connection = pool.get().await?;
    let channel = connection.create_channel().await?;

    let queue = channel
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
/*    for _ in 1..10 {
        let mut connection = pool.get().await?;
        let channel = connection.create_channel().await?;
        channel.basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            b"hello from deadpool".to_vec(),
            BasicProperties::default(),
        ).await?;
    }*/
    let mut conn = pool.get().await?;
    let chan = conn.create_channel().await?;
   let mut consumer = chan.basic_consume("hello", "", BasicConsumeOptions::default(), FieldTable::default()).await?;
    while let Some(msg) = consumer.next().await {
        let (channel, delivery) = msg.expect("error in consumer");
        let data = &delivery.data;
        println!("{}", String::from_utf8_lossy(&data[..]));
        delivery
            .ack(BasicAckOptions::default())
            .await
            .expect("ack");
    }
    Ok(())
}