#[cfg(test)]
mod test {
    use domain::{Address, Profile, User};
    use messenger::Message;
    use messenger::Messenger;
    use std::sync::Arc;

    #[tokio::test]
    async fn hello() {
        let routing_key = "oops";
        let messenger = Arc::new(Messenger::new("ohaio", "wesh").await.unwrap());

        let clone = Arc::clone(&messenger);
        let publisher_fut = tokio::task::spawn(async move {
            let user = create_user();
            let _ = clone.publish(routing_key, &user).await.unwrap();
        });
        let clone = Arc::clone(&messenger);

        let subscriber_fut_1 = tokio::task::spawn(async move {
            async fn f(msg: Message) -> Result<(), Box<dyn std::error::Error>> {
                println!("from fut 1: {}", msg.payload_as_string());
                Ok(())
            }
            messenger::consume_and_ack(clone, String::from(routing_key), f, Some(1))
                .await
                .unwrap();
        });
        let messenger = Arc::new(Messenger::new("ohaio", "wesh2").await.unwrap());
        let subscriber_fut_2 = tokio::task::spawn(async move {
            async fn f(msg: Message) -> Result<(), Box<dyn std::error::Error>> {
                println!("from fut 2: {}", msg.payload_as_string());
                Ok(())
            }
            messenger::consume_and_ack(messenger, String::from(routing_key), f, Some(1))
                .await
                .unwrap();
        });
        let _ =
            futures_util::future::join_all(vec![publisher_fut, subscriber_fut_1, subscriber_fut_2])
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
}
