use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use tracing::info;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} started", ready.user.name);
    }
}

pub struct DiscordClient {
    client: Client,
}

impl DiscordClient {
    pub async fn new() -> Self {
        let framework = StandardFramework::new()
            .configure(|c| c.with_whitespace(true).prefix("~"))
            .normal_message(normal_message_hook);

        // TODO: Actual error handling
        let token = std::env::var("DISCORD_TOKEN").unwrap();
        let client = Client::builder(&token, GatewayIntents::all())
            .event_handler(Handler)
            .framework(framework)
            .await
            .unwrap();

        Self { client }
    }

    pub async fn start(&mut self) {
        // TODO: Actual error handling
        self.client.start().await.unwrap();
    }
}

#[hook]
async fn normal_message_hook(_: &Context, msg: &Message) {
    println!("Message received: {:?}", msg.content);
}
