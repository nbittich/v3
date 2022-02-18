use core::panic;
use domain::{
    CreateUserCommand, Id, Metadata, Profile, UserCreatedEvent, WithJsonProcessor, WithMetadata,
};
use futures_util::StreamExt;
use messenger::Messenger;
use std::sync::Arc;
use store::{MongoRepository, Repository, StoreClient};
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const APP_NAME: &str = "user_ms";
const USER_COLLECTION: &str = "user";
const USER_CREATED_DEFAULT_ROLE: &str = "USER";

#[tokio::main]
async fn main() {
    setup_tracing();
    let messenger = Messenger::new(messenger::messages::USER_EXCHANGE, APP_NAME).await;
    let store = StoreClient::new(APP_NAME.to_string()).await;
    match (messenger, store) {
        (Ok(messenger), Ok(store)) => run(messenger, store).await,
        (Err(msg), _) => panic!("could not create messenger for {APP_NAME}\n: {msg}"),
        (_, Err(msg)) => panic!("could not create store for {APP_NAME}\n: {msg}"),
    }
}

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[tracing::instrument(skip_all)]
async fn run(messenger: Messenger, store_client: StoreClient) {
    tracing::info!("Running {}", APP_NAME);
    let messenger = Arc::new(messenger);
    let store_client = Arc::new(store_client);

    let handles = vec![spawn_on_create_user_command(
        Arc::clone(&messenger),
        Arc::clone(&store_client),
    )];

    let _res = futures_util::future::join_all(handles).await;
}
fn spawn_on_create_user_command(
    messenger: Arc<Messenger>,
    store_client: Arc<StoreClient>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async move { on_create_user_command(messenger, store_client).await })
}

#[tracing::instrument(skip_all)]
async fn on_create_user_command(
    messenger: Arc<Messenger>,
    store_client: Arc<StoreClient>,
) -> anyhow::Result<()> {
    let db = store_client.get_db();
    let collection = db.collection::<User>(USER_COLLECTION);
    let repository = MongoRepository::new(collection);
    let mut consumer = messenger
        .subscribe(messenger::messages::CREATE_USER_COMMAND)
        .await?;

    while let Some(msg) = consumer.next().await {
        let (_channel, delivery) = msg?;
        let msg = messenger::to_message(&delivery)?;
        tracing::info!(
            "received create user command from {} at {}",
            msg.sender(),
            msg.creation_date()
        );
        let payload = CreateUserCommand::from_json_slice(&msg.payload()[..]);
        match payload {
            Err(msg) => tracing::error!("payload could not be parsed: {}", msg),
            Ok(payload) => {
                let user = User::from_create_command(payload);
                let _ = repository.insert_one(&user).await?;
                let email = String::from(user.profile.email_address());
                let confirmation = messenger
                    .publish(
                        messenger::messages::USER_CREATED_EVENT,
                        &UserCreatedEvent {
                            domain_metadata: Metadata::new_with_default(&user.id),
                            email,
                            nickname: user.nickname,
                        },
                    )
                    .await?;
                tracing::info!("confirmation: {}", confirmation.is_ack());
            }
        };

        let _ = delivery.ack(messenger::BasicAckOptions::default()).await?;
    }
    Ok(())
}
#[derive(
    Debug, Default, WithJsonProcessor, WithMetadata, domain::Serialize, domain::Deserialize,
)]
struct User {
    #[serde(rename = "_id")]
    id: Id,
    #[serde(rename = "metadata")]
    domain_metadata: Metadata,
    nickname: String,
    password: String,
    profile: Profile,
    roles: Vec<String>,
}

impl User {
    fn from_create_command(command: CreateUserCommand) -> User {
        let user_id = Id::default();
        let metadata = Metadata::new_with_default(&user_id);
        User {
            profile: Profile::new_with_default(&command.email),
            id: user_id,
            domain_metadata: metadata,
            nickname: command.nickname,
            password: command.password,
            roles: vec![USER_CREATED_DEFAULT_ROLE.to_string()],
        }
    }
}
