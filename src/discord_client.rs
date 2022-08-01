use crate::counter_db::CounterDb;

use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::prelude::*;
use serenity::Client;
use tracing::info;

use std::sync::Arc;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} started", ready.user.name);
    }
}

struct DBClient;

impl TypeMapKey for DBClient {
    type Value = Arc<CounterDb>;
}

pub struct DiscordClient {
    client: Client,
    // FIXME: Use or remove this
    #[allow(unused)]
    counter_db_client: Arc<CounterDb>,
}

impl DiscordClient {
    pub async fn new(counter_db_client: CounterDb) -> Self {
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

        let counter_db_client = Arc::new(counter_db_client);
        {
            let mut data = client.data.write().await;

            data.insert::<DBClient>(counter_db_client.clone());
        }

        Self {
            client,
            counter_db_client,
        }
    }

    pub async fn start(&mut self) {
        // TODO: Actual error handling
        self.client.start().await.unwrap();
    }
}

#[hook]
async fn normal_message_hook(_: &Context, msg: &Message) {
    info!(
        "Message received: {:?} from {:?}",
        msg.content, msg.author.name
    );
}
