mod discord_client;

use crate::discord_client::DiscordClient;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut client = DiscordClient::new().await;

    client.start().await;
}
