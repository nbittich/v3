#![feature(thread_id_value)]
extern crate core;

use bb8::{Pool, RunError, PooledConnection};
use bb8_redis::redis::aio::PubSub;
use bb8_redis::redis::{
    AsyncCommands, ConnectionAddr, ConnectionInfo, RedisConnectionInfo, RedisError, RedisResult, Msg, Connection,
};
use bb8_redis::RedisConnectionManager;
use futures_util::StreamExt;
use std::env::var;
use std::time::Duration;

const REDIS_HOST: &str = "REDIS_HOST";
const REDIS_USERNAME: &str = "REDIS_USERNAME";
const REDIS_PASSWORD: &str = "REDIS_PASSWORD";
const REDIS_PORT: &str = "REDIS_PORT";

pub struct Subscription {
    name: String,
    conn: PubSub,
}
impl Subscription {
    fn recv<F, T>( self, f: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn(Msg) -> T + Send + 'static,
        T: Send,
    {
        tokio::spawn(async move {
            let mut sub = self.conn;
             let mut pubsub_stream = sub.on_message();
            while let Some(msg) = pubsub_stream.next().await {
                f(msg);
            }
        })
    }
}
pub struct RedisService {
    pool: Pool<RedisConnectionManager>,
}
impl RedisService {
    pub async fn subscribe(&self, chan: &str) -> Result<Subscription, RedisError> {
        let conn = self.pool.dedicated_connection().await?;
        let mut pub_sub = conn.into_pubsub();
        pub_sub.subscribe(chan).await?;
        Ok(Subscription {
            name: String::from(chan),
            conn: pub_sub,
        })
    }

    async fn pick_conn_from_pool(&self) -> RedisResult<PooledConnection<'_, RedisConnectionManager>> {
        let conn = self.pool.get().await.map_err(|e| match e {
            RunError::User(e) => e,
            RunError::TimedOut => RedisError::from(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "redis timed out",
            )),
        });
        conn
    }

    pub async fn publish(&self, chan: &str, msg: String) -> RedisResult<()> {
        let mut conn = self.pick_conn_from_pool().await?;
        //tokio::time::sleep(Duration::from_secs(10)).await;
        conn.publish(chan, &msg).await
    }
    pub async fn create_pool() -> Self {
        let redis_host = var(REDIS_HOST).unwrap_or_else(|_| String::from("localhost"));
        let redis_port = var(REDIS_PORT).unwrap_or_else(|_| String::from("6379"));
        let redis_port = redis_port.parse::<u16>().unwrap();
        let connection_info = ConnectionInfo {
            addr: ConnectionAddr::Tcp(redis_host, redis_port),
            redis: RedisConnectionInfo {
                db: 0,
                username: var(REDIS_USERNAME).ok(),
                password: var(REDIS_PASSWORD).ok(),
            },
        };
        let manager = bb8_redis::RedisConnectionManager::new(connection_info).unwrap();
        let pool = bb8::Pool::builder()
            .max_size(50)
            .connection_timeout(Duration::from_secs(3600))
            .build(manager)
            .await
            .unwrap();

        RedisService { pool }
    }
}

#[cfg(test)]
mod tests {
    use bb8_redis::redis::Msg;

    use std::sync::Arc;

    use std::time::{Duration, SystemTime};

    use crate::{RedisService, Subscription};

   

    fn send(
        redis_service: &Arc<RedisService>,
        chan: &'static str,
        msg: String,
    ) -> tokio::task::JoinHandle<()> {
        let clone = Arc::clone(redis_service);
        tokio::spawn(async move {
            let res = clone.publish(chan, msg).await;
            if let Some(_) = res.err() {
                println!("woops, some packet lost, shit happens");
            }
        })
    }

    #[tokio::test]
    async fn it_works() {
        let overall_time = SystemTime::now();
        let redis_service = Arc::new(RedisService::create_pool().await);

        let subscription = redis_service.subscribe("heyya").await.unwrap();
        let subscription2 = redis_service.subscribe("heyya").await.unwrap();

        let i = 2;
        let mut futs = Vec::with_capacity(1000001);
        for a in 0..i {
            futs.push(send(&redis_service, "heyya", format!("ola amigo{a}")));
        }

        let process_msg = |msg: Msg| {
            let pubsub_msg: String = msg.get_payload().unwrap();
            println!("{}", pubsub_msg);
        };
        futs.push(subscription.recv(process_msg));
        futs.push(subscription2.recv(process_msg));

        let _ = futures_util::future::join_all(futs).await;

        println!(
            "Overall time spent: {:?}",
            SystemTime::now().duration_since(overall_time)
        );
    }

    #[tokio::test]
    async fn some_async_test() {
        let overall_time = SystemTime::now();
        let mut futs = Vec::new();
        for i in 1..12 {
            futs.push(tokio::spawn(async move {
                let mut count = 1;
                let overall_time = SystemTime::now();

                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    count += 1;
                    if count == 5 {
                        break;
                    }
                }
                println!(
                    "time in {i}: {:?}",
                    SystemTime::now().duration_since(overall_time)
                );
            }));
        }
        futures_util::future::join_all(futs).await;

        println!("{:?}", SystemTime::now().duration_since(overall_time));
    }
}
