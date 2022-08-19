mod counter_db;
mod discord_client;

use crate::discord_client::DiscordClient;

use counter_db::CounterDb;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("DB_PATH").unwrap();
    let counter_db_client = CounterDb::new(db_path);
    let mut client = DiscordClient::new(counter_db_client).await;

    client.start().await;
}
