use chrono::prelude::*;
use serde::Serialize;
use std::{env, fmt, ops::Mul};

#[derive(PartialEq, Eq, Copy, Clone, Serialize)]
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

#[derive(Serialize)]
pub struct State {
    pub discord_init: bool,
    consts: Consts,
    status: DoorPosition,
    last_update: Option<DateTime<Local>>,
    open_time: Option<DateTime<Local>>,
    pings_sent: u32,
    current_ping_interval: Duration,
}

impl State {
    pub fn new() -> State {
        let consts = Consts::new();
        State {
            discord_init: false,
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
                    if diff > self.consts.missing_timeout.into() {
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
                    if diff > self.current_ping_interval.into() {
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
                    if diff > self.current_ping_interval.into() {
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
#[derive(Clone, Serialize)]
pub struct Consts {
    // Discord bot token
    pub discord_token: Secret,

    // Wait interval between discord updates
    pub discord_ping_loop_interval: Duration,
    pub discord_presence_loop_interval: Duration,
    pub discord_log_loop_interval: Duration,

    // Discord channels to use
    pub discord_ping_channel: String,
    pub discord_log_channel: String,

    // Missing times
    pub missing_loop_interval: Duration,
    pub missing_timeout: Duration,

    // Other timing consts
    pub starting_ping_interval: Duration,
}

impl Consts {
    // Handling initialisation of the constant data. Will panic if invalid config in env
    pub fn new() -> Consts {
        fn get_env(name: &str) -> Option<String> {
            env::var(name).ok()
        }

        fn get_duration(env_var: &str, default: u32) -> Duration {
            let env = get_env(env_var);
            match env {
                Some(str) => {
                    let val = str
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid duration {}", str));
                    Duration(val)
                }
                None => Duration(default),
            }
        }

        fn get_timedelta(env_var: &str, default: u32) -> Duration {
            let env = get_env(env_var);
            match env {
                Some(str) => {
                    let val = str
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid duration {}", str));
                    Duration(val)
                }
                None => Duration(default),
            }
        }

        Consts {
            discord_token: Secret(
                get_env("DISCORD_TOKEN").expect("Could not retrive anv var DISCORD_TOKEN"),
            ),
            discord_ping_loop_interval: get_duration("DISCORD_PING_LOOP_INTERVAL", 1),
            discord_presence_loop_interval: get_duration("DISCORD_PRESENCE_LOOP_INTERVAL", 1),
            discord_log_loop_interval: get_duration("DISCORD_LOG_LOOP_INTERVAL", 1),
            discord_ping_channel: get_env("DISCORD_PING_CHANNEL").unwrap_or("pings".to_string()),
            discord_log_channel: get_env("DISCORD_LOG_CHANNEL").unwrap_or("log".to_string()),
            missing_loop_interval: get_duration("MISSING_LOOP_INTERVAL", 1),
            missing_timeout: get_timedelta("MISSING_TIMEOUT", 60),
            starting_ping_interval: get_timedelta("STARTING_PING_INTERVAL", 10),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Duration(u32);

impl From<Duration> for tokio::time::Duration {
    fn from(Duration(secs): Duration) -> tokio::time::Duration {
        tokio::time::Duration::from_secs(secs as u64)
    }
}

impl From<Duration> for chrono::TimeDelta {
    fn from(Duration(secs): Duration) -> chrono::TimeDelta {
        chrono::TimeDelta::seconds(secs as i64)
    }
}

impl<T> Mul<T> for Duration
where
    u32: Mul<T, Output = u32>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        let Duration(secs) = self;
        Self(secs * rhs)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timedelta: chrono::TimeDelta = (*self).into();
        write!(f, "{timedelta}")
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let timedelta: chrono::TimeDelta = (*self).into();
        serializer.serialize_str(&timedelta.to_string())
    }
}

#[derive(Clone)]
pub struct Secret(pub String);

impl Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("<redacted>")
    }
}
