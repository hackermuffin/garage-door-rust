use futures::future;
use http_types::Result;
use log::{debug, error, trace};
use serenity::{
    all::{ActivityData, GuildId, Message, OnlineStatus, Ready},
    async_trait,
    prelude::*,
};
use tokio::{join, time::sleep};

use crate::state::State;
use crate::Mutex;

async fn update_status(state: &Mutex<State>, ctx: Context) {
    debug!("Starting discord status update loop...");
    let discord_presence_loop_interval = state.lock().await.consts().discord_presence_loop_interval;
    loop {
        // TODO retrieve current presence
        // TODO set up discord rich presence

        // Set presence
        let status = *state.lock().await.status();
        trace!("Updating discord status to {status}...");
        let presence = match status {
            crate::state::DoorPosition::Open => "Open",
            crate::state::DoorPosition::Closed => "Closed",
            crate::state::DoorPosition::Missing => "Missing",
        };
        ctx.set_presence(Some(ActivityData::playing(presence)), OnlineStatus::Online);
        trace!("Discord status update complete.");

        sleep(discord_presence_loop_interval.into()).await;
    }
}

async fn check_ping(state: &Mutex<State>, ctx: Context, data_about_bot: Ready) {
    debug!("Starting discord ping loop...");
    let consts = state.lock().await.consts().clone();
    loop {
        // Check if ping needs to be sent
        let open_state = state.lock().await;
        let ping = open_state.check_send_ping();
        drop(open_state);

        match &ping {
            Some(string) => debug!("Discord ping required: {string}."),
            None => trace!("Discord ping not required."),
        }

        // Send ping
        if let Some(ping) = ping {
            let res =
                send_message(&ctx, &data_about_bot, &consts.discord_ping_channel, &ping).await;
            if let Err(e) = res {
                error!("Error encoundered during ping: {}", e);
            }

            // Update state
            debug!("Updating internal state to indicate ping sent.");
            let mut open_state = state.lock().await;
            open_state.ping_sent();
            drop(open_state);
            debug!("Internal state updated.");
        }

        sleep(consts.discord_ping_loop_interval.into()).await;
    }
}

async fn log(state: &Mutex<State>, ctx: Context, data_about_bot: Ready) {
    debug!("Starting discord log loop...");
    let consts = state.lock().await.consts().clone();
    let mut prev_status = *state.lock().await.status();
    loop {
        // Get current status
        let status = *state.lock().await.status();

        if status != prev_status {
            debug!("Status transition from {prev_status} -> {status}");
            let msg = format!("Status updated to {}.", status);
            let res = send_message(&ctx, &data_about_bot, &consts.discord_log_channel, &msg).await;

            match res {
                Ok(()) => debug!("Discord log message succesfully sent"),
                Err(e) => error! {"Failure sending discord message: {e}"},
            }

            prev_status = status;
        }

        sleep(consts.discord_log_loop_interval.into()).await;
    }
}

// Sends message to the given channel name in all guilds
async fn send_message(
    ctx: &Context,
    data_about_bot: &Ready,
    channel_name: &str,
    message: &str,
) -> Result<()> {
    debug!("Sending discord message: {message}");
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
    debug!(
        "Sending to: {}",
        channels_to_send
            .iter()
            .map(|(_, guild_channel)| {
                format!(
                    "{}:{}",
                    guild_channel
                        .guild_id
                        .name(&ctx)
                        .unwrap_or("<unknown>".to_string()),
                    guild_channel.name.clone()
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

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
        debug!("Discord bot ready!");
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
                error!("Error sending message: {why:?}");
            }
        }
    }
}

pub async fn main(state: &'static Mutex<State>) {
    debug!("Starting discord main...");
    // Login with a bot token from the environment
    let token = state.lock().await.consts().discord_token.clone();
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    debug!("Attempting discord login...");
    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { state })
        .await
        .expect("Err creating client");
    debug!("Discord login complete.");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
