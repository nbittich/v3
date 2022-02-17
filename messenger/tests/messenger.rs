#[cfg(test)]
mod test {
    use domain::{CreateUserCommand, Metadata};
    use domain::{Address, Profile, User};
    use futures_util::Future;
    use futures_util::StreamExt;
    use messenger::BasicAckOptions;
    use messenger::Message;
    use messenger::Messenger;
    use std::sync::Arc;

    use std::error::Error;
    #[tokio::test]
    async fn hello() {
        let routing_key = "oops";
        let messenger = Arc::new(Messenger::new("ohaio", "wesh").await.unwrap());
        let messenger2 = Arc::new(Messenger::new("User", "wesh").await.unwrap());

        let clone = Arc::clone(&messenger);
        let _publisher_fut = tokio::task::spawn(async move {
            let user = create_user();
            let _ = clone.publish(routing_key, &user).await.unwrap();
        });
        let publisher_fut2 = tokio::task::spawn(async move {
            let _ = messenger2.publish("CreateUserCommand", &CreateUserCommand { domain_metadata: Metadata::default(), nickname:String::from("nordine"), password: String::from("kikoo"), confirm_password: String::from("kikoo"), email: String::from("kikoo@lol.com") }).await.unwrap();
        });
        let clone = Arc::clone(&messenger);

        let _subscriber_fut_1 = tokio::task::spawn(async move {
            async fn f(msg: Message) -> Result<(), Box<dyn std::error::Error>> {
                println!("from fut 1: {}", msg.payload_as_string());
                Ok(())
            }
            consume_and_ack(clone, String::from(routing_key), f, Some(1))
                .await
                .unwrap();
        });
        let messenger = Arc::new(Messenger::new("ohaio", "wesh2").await.unwrap());
        let _subscriber_fut_2 = tokio::task::spawn(async move {
            async fn f(msg: Message) -> Result<(), Box<dyn std::error::Error>> {
                println!("from fut 2: {}", msg.payload_as_string());
                Ok(())
            }
            consume_and_ack(messenger, String::from(routing_key), f, Some(1))
                .await
                .unwrap();
        });
        let _ =
            futures_util::future::join_all(vec![publisher_fut2])
                .await;
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
            let msg = messenger::to_message(&delivery)?;
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
}
