use http_types::Result;
use serenity::all::OnlineStatus;
use serenity::async_trait;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use std::env;

use futures::future;
use tokio::join;
use tokio::time::{sleep, Duration};

use crate::STATE;

const DISCORD_PING_LOOP_INTERVAL: Duration = Duration::from_secs(1);
const DISCORD_PRESENCE_LOOP_INTERVAL: Duration = Duration::from_secs(1);
const DISCORD_LOG_LOOP_INTERVAL: Duration = Duration::from_secs(1);

const DISCORD_PING_CHANNEL: &str = "pings";
const DISCORD_LOG_CHANNEL: &str = "log";

async fn update_status(ctx: Context) {
    loop {
        // Get current presence
        // TODO

        // Set presence
        let state = STATE.lock().await;
        let status = state.status;
        drop(state);
        let presence = match status {
            crate::state::DoorPosition::Open => "Open",
            crate::state::DoorPosition::Closed => "Closed",
            crate::state::DoorPosition::Missing => "Missing",
        };
        ctx.set_presence(Some(ActivityData::playing(presence)), OnlineStatus::Online);

        sleep(DISCORD_PRESENCE_LOOP_INTERVAL).await;
    }
}

async fn check_ping(ctx: Context, data_about_bot: Ready) {
    loop {
        // Check if ping needs to be sent
        let state = STATE.lock().await;
        let ping = state.check_send_ping();
        drop(state);

        // Send ping
        if let Some(ping) = ping {
            let res = send_message(&ctx, &data_about_bot, DISCORD_PING_CHANNEL, &ping).await;
            if let Err(e) = res {
                println!("Error encoundered during ping: {}", e);
            }

            // Update state
            let mut state = STATE.lock().await;
            state.ping_sent();
            drop(state);
        }

        sleep(DISCORD_PING_LOOP_INTERVAL).await;
    }
}

async fn log(ctx: Context, data_about_bot: Ready) {
    let mut prev_status = STATE.lock().await.status;
    loop {
        // Get current status
        let status = STATE.lock().await.status;

        if status != prev_status {
            let msg = format!("Status updated to {:?}.", status);
            let res = send_message(&ctx, &data_about_bot, DISCORD_LOG_CHANNEL, &msg).await;

            if let Err(e) = res {
                println! {"Error logging: {}", e}
            }

            prev_status = status;
        }

        sleep(DISCORD_LOG_LOOP_INTERVAL).await;
    }
}

// Sends message to the given channel name in all guilds
async fn send_message(
    ctx: &Context,
    data_about_bot: &Ready,
    channel_name: &str,
    message: &str,
) -> Result<()> {
    //Result<Vec<Result<Message, http_types::Error>>> {
    let guilds: Vec<GuildId> = data_about_bot.guilds.iter().map(|x| x.id).collect();
    let all_channels =
        future::try_join_all(guilds.iter().map(|x| x.channels(ctx.http.clone()))).await?;
    let channels_to_send = all_channels
        .iter()
        .map(|x| {
            x.iter()
                .filter(|&(_, channel)| channel.name() == channel_name)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .concat();
    let sends = channels_to_send
        .iter()
        .map(|(_, channel)| channel.say(&ctx.http, message));

    future::try_join_all(sends).await?;

    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        join![
            update_status(ctx.clone()),
            check_ping(ctx.clone(), data_about_bot.clone()),
            log(ctx, data_about_bot)
        ];
    }
    async fn message(&self, ctx: Context, msg: Message) {
        ctx.set_presence(Some(ActivityData::playing("test")), OnlineStatus::Idle);
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }
}

pub async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
