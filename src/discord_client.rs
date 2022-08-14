use crate::counter_db::CounterDb;

use regex::Regex;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::prelude::*;
use serenity::Client;
use tracing::{debug, info};

use std::collections::HashSet;
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

struct Categories;

impl TypeMapKey for Categories {
    type Value = HashSet<String>;
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

            // TODO: Set this dynamically
            let categories = HashSet::from(["ign".to_string()]);
            data.insert::<Categories>(categories);
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
async fn normal_message_hook(ctx: &Context, msg: &Message) {
    info!(
        "Message received: {:?} from {:?}",
        msg.content, msg.author.name
    );

    let data = ctx.data.read().await;
    let counter_db = data.get::<DBClient>().unwrap();

    let categories = data.get::<Categories>().unwrap();
    let lower_msg = msg.content.to_lowercase();

    for category in categories {
        let regex_string = format!(r"\b({})\b", category);
        let re = Regex::new(&regex_string).unwrap();

        if re.is_match(&lower_msg) {
            let key = format!("USER#{}#{}", msg.author.name, category);
            counter_db.inc_key(&key, 1);
            debug!("Incremented {} to {}", key, counter_db.get_key(&key));
        }
    }
}
