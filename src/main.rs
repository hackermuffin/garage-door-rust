use log::{debug, error, info};
use tokio::join;
use tokio::sync::Mutex;

mod discord;
mod state;
mod timeout;
mod web;

#[tokio::main]
async fn main() {
    logger_init();
    info!("Starting program...");

    let state: &'static Mutex<state::State> = leak(Mutex::new(state::State::new()));

    match serde_json::to_string_pretty(&*state.lock().await) {
        Ok(json) => debug!("State read in:\n{}", json),
        Err(e) => {
            error!("Unable to print parsed state: {}", e)
        }
    }

    debug!("Starting subprocesses...");
    let _ = join!(discord::main(state), web::main(state), timeout::main(state));
}

fn logger_init() {
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    debug!("Logger initialised succfully.")
}

fn leak<T>(x: T) -> &'static mut T {
    Box::leak(Box::new(x))
}
