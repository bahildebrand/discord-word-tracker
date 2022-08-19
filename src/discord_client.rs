use crate::counter_db::CounterDb;

use regex::Regex;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{Args, CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::prelude::*;
use serenity::Client;
use tracing::{debug, error, info};

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum DiscordClientError {
    InvalidArgs,
}

impl fmt::Display for DiscordClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidArgs => write!(f, "Invalid number of args"),
        }
    }
}

impl Error for DiscordClientError {}

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

struct YoutubeParserContainer;

impl TypeMapKey for YoutubeParserContainer {
    type Value = YoutubeParser;
}

#[command]
#[description = "See the top shills for a category"]
#[usage("~leaderboard <category>")]
#[example("~leaderboard ign")]
async fn leaderboard(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() != 1 {
        return Err(DiscordClientError::InvalidArgs.into());
    }

    let data = ctx.data.read().await;
    let counter_db = data.get::<DBClient>().unwrap();

    let category = args.current().expect("Argument should always be present");
    let category_key = format!("USER#{}", category);
    let mut results = counter_db.prefix_get_key(&category_key);

    results.sort_by(|(_, a), (_, b)| b.cmp(&a));
    let results: Vec<(String, u64)> = results
        .into_iter()
        .map(|(key, value)| {
            let key_string = std::str::from_utf8(&key).unwrap();
            (
                key_string.split("#").into_iter().collect::<Vec<_>>()[2].to_string(),
                u64::from_be_bytes((*value).try_into().unwrap()),
            )
        })
        .collect();

    let mut response_string = format!("{} leaderboard:", category);
    for res in results {
        let row = format!("\n{} - {}", res.0, res.1);
        response_string.push_str(&row);
    }

    msg.reply(&ctx, response_string).await.unwrap();

    Ok(())
}

#[group]
#[commands(leaderboard)]
struct General;

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
            .group(&GENERAL_GROUP)
            .normal_message(normal_message_hook);

        // TODO: Actual error handling
        let discord_token = std::env::var("DISCORD_TOKEN").unwrap();
        let client = Client::builder(&discord_token, GatewayIntents::all())
            .event_handler(Handler)
            .framework(framework)
            .await
            .unwrap();

        let youtube_token = std::env::var("YOUTUBE_TOKEN").unwrap();
        let youtube_parser = YoutubeParser::new(youtube_token);

        let counter_db_client = Arc::new(counter_db_client);
        {
            let mut data = client.data.write().await;

            data.insert::<DBClient>(counter_db_client.clone());

            // TODO: Set this dynamically
            let categories = HashSet::from(["ign".to_string()]);
            data.insert::<Categories>(categories);

            data.insert::<YoutubeParserContainer>(youtube_parser);
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

    let yt_parser = data.get::<YoutubeParserContainer>().unwrap();

    let lower_msg = if msg.content.contains("youtube.com") {
        parse_yt_link_channel(&msg.content, yt_parser)
            .await
            .unwrap_or_else(|| msg.content.to_lowercase())
    } else {
        msg.content.to_lowercase()
    };

    for category in categories {
        let regex_string = format!(r"\b({})\b", category);
        let re = Regex::new(&regex_string).unwrap();

        if re.is_match(&lower_msg) {
            let key = format!("USER#{}#{}", category, msg.author.name);
            counter_db.inc_key(&key, 1);
            debug!("Incremented {} to {}", key, counter_db.get_key(&key));
        }
    }
}

async fn parse_yt_link_channel(link: &str, yt_parser: &YoutubeParser) -> Option<String> {
    let args = link.split(|c| c == '?' || c == '&');
    let mut video_id = String::from("");

    for a in args {
        let a_string = String::from(a);
        let stripped_string = a_string.strip_prefix("v=");

        match stripped_string {
            Some(id) => {
                video_id = String::from(id);
                break;
            }
            None => continue,
        }
    }

    yt_parser.get_channel_name(video_id).await
}

pub struct YoutubeParser {
    youtube_api_key: String,
}

impl YoutubeParser {
    pub fn new(youtube_api_key: String) -> YoutubeParser {
        YoutubeParser { youtube_api_key }
    }

    pub async fn get_channel_name(&self, video_id: String) -> Option<String> {
        let video_url = format!(
            "https://www.googleapis.com/youtube/v3/\
                videos?part=snippet&id={}&key={}",
            video_id, self.youtube_api_key
        );

        let res = reqwest::get(&video_url).await;
        match res {
            Err(e) => {
                error!("Failed to get video by id: {}", e);
                None
            }
            Ok(resp) => {
                let json_val = json::parse(&resp.text().await.unwrap_or_default()[..]).unwrap();
                let channel = json_val["items"][0]["snippet"]["channelTitle"].to_string();
                Some(channel.to_lowercase())
            }
        }
    }
}
