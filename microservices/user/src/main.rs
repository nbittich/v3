use core::panic;
use domain::{CreateUserCommand, WithJsonProcessor};
use messenger::{Message, Messenger};
use std::sync::Arc;
use store::StoreClient;
use tokio::task::JoinHandle;
use tracing::info;

const APP_NAME: &str = "USER_APP";
const USER_COLLECTION: &str = "user";

#[tokio::main]
async fn main() {
    let messenger = Messenger::new(messenger::messages::USER_EXCHANGE, APP_NAME).await;
    let store = StoreClient::new(APP_NAME.to_string()).await;
    match (messenger, store) {
        (Ok(messenger), Ok(store)) => run(messenger, store).await,
        (Err(msg), _) => panic!("could not create messenger for {APP_NAME}\n: {msg}"),
        (_, Err(msg)) => panic!("could not create store for {APP_NAME}\n: {msg}"),
    }
}
#[tracing::instrument]
async fn run(messenger: Messenger, store_client: StoreClient) {
    info!("Running {}", APP_NAME);
    let messenger = Arc::new(messenger);
    let store_client = Arc::new(store_client);

    let handles = vec![spawn_on_create_user_command(
        Arc::clone(&messenger),
        Arc::clone(&store_client),
    )];

    futures_util::future::join_all(handles).await;
}

#[tracing::instrument]
fn spawn_on_create_user_command(
    messenger: Arc<Messenger>,
    store_client: Arc<StoreClient>,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        let result = messenger::consume_and_ack(
            messenger,
            messenger::messages::CREATE_USER_COMMAND.to_string(),
            on_create_user_command,
            None,
        )
        .await;
        if let Err(msg) = result {
            tracing::error!("Error on create user command task: {}", msg);
        }
    })
}

#[tracing::instrument]
async fn on_create_user_command(msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "received create user command from {} at {}",
        msg.sender(),
        msg.creation_date()
    );
    let payload = CreateUserCommand::from_json_slice(&msg.payload()[..]);
    if let Err(msg) = payload {
        tracing::error!("payload could not be parsed: {}", msg);
    } else {
        let payload = payload.unwrap();
        // todo
    }
    Ok(())
}
