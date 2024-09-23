use tokio::join;
use tokio::sync::Mutex;

mod discord;
mod state;
mod timeout;
mod web;

static STATE: Mutex<state::State> = Mutex::const_new(state::initial_state());

#[tokio::main]
async fn main() {
    let _ = join!(discord::main(), web::main(), timeout::main());
}
