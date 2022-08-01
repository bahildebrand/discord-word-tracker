mod counter_db;
mod discord_client;

use crate::discord_client::DiscordClient;

use counter_db::CounterDb;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let counter_db_client = CounterDb::new();
    let mut client = DiscordClient::new(counter_db_client).await;

    client.start().await;
}
