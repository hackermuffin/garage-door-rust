use chrono::{prelude::*, TimeDelta};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::{env, fmt};
use tokio::time::Duration;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize)]
pub enum DoorPosition {
    Open,
    Closed,
    Missing,
}

impl fmt::Display for DoorPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Open => "open",
                Self::Closed => "closed",
                Self::Missing => "missing",
            }
        )
    }
}

#[serde_as]
#[derive(Serialize)]
pub struct State {
    consts: Consts,
    status: DoorPosition,
    last_update: Option<DateTime<Local>>,
    open_time: Option<DateTime<Local>>,
    pings_sent: u32,
    #[serde_as(as = "DisplayFromStr")]
    current_ping_interval: TimeDelta,
}

impl State {
    pub fn new() -> State {
        let consts = Consts::new();
        State {
            current_ping_interval: consts.starting_ping_interval,
            consts,
            status: DoorPosition::Missing,
            last_update: None,
            open_time: None,
            pings_sent: 0,
        }
    }

    pub fn consts(&self) -> &Consts {
        &self.consts
    }

    pub fn status(&self) -> &DoorPosition {
        &self.status
    }

    pub fn open(&mut self) {
        self.status = DoorPosition::Open;
        self.last_update = Some(Local::now());
        if self.open_time.is_none() {
            self.open_time = Some(Local::now())
        }
    }
    pub fn close(&mut self) {
        self.status = DoorPosition::Closed;
        self.last_update = Some(Local::now());
        self.open_time = None;
        self.pings_sent = 0;
        self.current_ping_interval = self.consts.starting_ping_interval;
    }
    pub fn missing(&mut self) {
        self.status = DoorPosition::Missing;
    }
    pub fn check_timeout(&mut self) {
        if self.status != DoorPosition::Missing {
            match self.last_update {
                None => {}
                Some(last_update) => {
                    let diff = Local::now() - last_update;
                    if diff > self.consts.missing_timeout {
                        self.missing();
                    }
                }
            }
        }
    }
    pub fn check_send_ping(&self) -> Option<String> {
        match self.status {
            DoorPosition::Missing => {
                if self.last_update.is_some() {
                    let diff = Local::now() - self.last_update.unwrap();
                    if diff > self.current_ping_interval {
                        return Some(format!(
                            "@everyone Door is missing for {}",
                            self.current_ping_interval
                        ));
                    }
                }
                None
            }
            DoorPosition::Open => {
                if self.open_time.is_some() {
                    let diff = Local::now() - self.open_time.unwrap();
                    if diff > self.current_ping_interval {
                        return Some(format!(
                            "@everyone Door is open for {}",
                            self.current_ping_interval
                        ));
                    }
                }
                None
            }
            DoorPosition::Closed => None,
        }
    }
    pub fn ping_sent(&mut self) {
        self.pings_sent += 1;

        self.current_ping_interval = self.current_ping_interval * 2;

        // TODO: should confirm time diff is greater than current_ping_interval
    }
}

// Store const data that should be setup once then read only accessible through state
#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct Consts {
    // Discord bot token
    #[serde(skip_serializing)]
    pub discord_token: String,

    // Wait interval between discord updates
    pub discord_ping_loop_interval: Duration,
    pub discord_presence_loop_interval: Duration,
    pub discord_log_loop_interval: Duration,

    // Discord channels to use
    pub discord_ping_channel: String,
    pub discord_log_channel: String,

    // Missing times
    pub missing_loop_interval: Duration,
    #[serde_as(as = "DisplayFromStr")]
    pub missing_timeout: TimeDelta,

    // Other timing consts
    #[serde_as(as = "DisplayFromStr")]
    pub starting_ping_interval: TimeDelta,
}

impl Consts {
    // Handling initialisation of the constant data. Will panic if invalid config in env
    pub fn new() -> Consts {
        fn get_env(name: &str) -> Option<String> {
            env::var(name).ok()
        }

        fn get_duration(env_var: &str, default: Duration) -> Duration {
            let env = get_env(env_var);
            match env {
                Some(str) => {
                    let val = str
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("Invalid duration {}", str));
                    Duration::from_secs(val)
                }
                None => default,
            }
        }

        fn get_timedelta(env_var: &str, default: TimeDelta) -> TimeDelta {
            let env = get_env(env_var);
            match env {
                Some(str) => {
                    let val = str
                        .parse::<i64>()
                        .unwrap_or_else(|_| panic!("Invalid duration {}", str));
                    TimeDelta::seconds(val)
                }
                None => default,
            }
        }

        Consts {
            discord_token: get_env("DISCORD_TOKEN")
                .expect("Could not retrive anv var DISCORD_TOKEN"),
            discord_ping_loop_interval: get_duration(
                "DISCORD_PING_LOOP_INTERVAL",
                Duration::from_secs(1),
            ),
            discord_presence_loop_interval: get_duration(
                "DISCORD_PRESENCE_LOOP_INTERVAL",
                Duration::from_secs(1),
            ),
            discord_log_loop_interval: get_duration(
                "DISCORD_LOG_LOOP_INTERVAL",
                Duration::from_secs(1),
            ),
            discord_ping_channel: get_env("DISCORD_PING_CHANNEL").unwrap_or("pings".to_string()),
            discord_log_channel: get_env("DISCORD_LOG_CHANNEL").unwrap_or("log".to_string()),
            missing_loop_interval: get_duration("MISSING_LOOP_INTERVAL", Duration::from_secs(1)),
            missing_timeout: get_timedelta("MISSING_TIMEOUT", TimeDelta::minutes(1)),
            starting_ping_interval: get_timedelta("STARTING_PING_INTERVAL", TimeDelta::seconds(10)),
        }
    }
}
