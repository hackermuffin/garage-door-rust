use http_types::Result;
use serenity::all::OnlineStatus;
use serenity::async_trait;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

use futures::future;
use tokio::join;
use tokio::time::sleep;

use crate::state::State;
use crate::Mutex;

async fn update_status(state: &Mutex<State>, ctx: Context) {
    let discord_presence_loop_interval = state.lock().await.consts().discord_presence_loop_interval;
    loop {
        // Get current presence
        // TODO

        // Set presence
        let status = *state.lock().await.status();
        let presence = match status {
            crate::state::DoorPosition::Open => "Open",
            crate::state::DoorPosition::Closed => "Closed",
            crate::state::DoorPosition::Missing => "Missing",
        };
        ctx.set_presence(Some(ActivityData::playing(presence)), OnlineStatus::Online);

        sleep(discord_presence_loop_interval).await;
    }
}

async fn check_ping(state: &Mutex<State>, ctx: Context, data_about_bot: Ready) {
    let consts = state.lock().await.consts().clone();
    loop {
        // Check if ping needs to be sent
        let open_state = state.lock().await;
        let ping = open_state.check_send_ping();
        drop(open_state);

        // Send ping
        if let Some(ping) = ping {
            let res =
                send_message(&ctx, &data_about_bot, &consts.discord_ping_channel, &ping).await;
            if let Err(e) = res {
                println!("Error encoundered during ping: {}", e);
            }

            // Update state
            let mut open_state = state.lock().await;
            open_state.ping_sent();
            drop(open_state);
        }

        sleep(consts.discord_ping_loop_interval).await;
    }
}

async fn log(state: &Mutex<State>, ctx: Context, data_about_bot: Ready) {
    let consts = state.lock().await.consts().clone();
    let mut prev_status = *state.lock().await.status();
    loop {
        // Get current status
        let status = *state.lock().await.status();

        if status != prev_status {
            let msg = format!("Status updated to {:?}.", status);
            let res = send_message(&ctx, &data_about_bot, &consts.discord_log_channel, &msg).await;

            if let Err(e) = res {
                println! {"Error logging: {}", e}
            }

            prev_status = status;
        }

        sleep(consts.discord_log_loop_interval).await;
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

struct Handler<'a> {
    state: &'a Mutex<State>,
}

#[async_trait]
impl<'a> EventHandler for Handler<'a> {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        join![
            update_status(self.state, ctx.clone()),
            check_ping(self.state, ctx.clone(), data_about_bot.clone()),
            log(self.state, ctx, data_about_bot)
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

pub async fn main(state: &'static Mutex<State>) {
    // Login with a bot token from the environment
    let token = state.lock().await.consts().discord_token.clone();
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { state })
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
