use chrono::prelude::*;
use chrono::TimeDelta;

//const BASE_PING_INTERVAL: TimeDelta = TimeDelta::minutes(10);
const BASE_PING_INTERVAL: TimeDelta = TimeDelta::seconds(10);
const MISSING_TIMEOUT: TimeDelta = TimeDelta::minutes(1);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DoorPosition {
    Open,
    Closed,
    Missing,
}

#[derive(Debug)]
pub struct State {
    pub status: DoorPosition,
    last_update: Option<DateTime<Local>>,
    open_time: Option<DateTime<Local>>,
    pings_sent: u32,
    current_ping_interval: TimeDelta,
}

pub const fn initial_state() -> State {
    State {
        status: DoorPosition::Missing,
        last_update: None,
        open_time: None,
        pings_sent: 0,
        current_ping_interval: BASE_PING_INTERVAL,
    }
}

impl State {
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
        self.current_ping_interval = BASE_PING_INTERVAL;
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
                    if diff > MISSING_TIMEOUT {
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
                            "Door is missing for {}",
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
                        return Some(format!("Door is open for {}", self.current_ping_interval));
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
