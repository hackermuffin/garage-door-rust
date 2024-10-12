use tokio::join;
use tokio::sync::Mutex;

mod discord;
mod state;
mod timeout;
mod web;

#[tokio::main]
async fn main() {
    let state: &'static Mutex<state::State> = leak(Mutex::new(state::State::new()));
    let _ = join!(discord::main(state), web::main(state), timeout::main(state));
}

fn leak<T>(x: T) -> &'static mut T {
    Box::leak(Box::new(x))
}
