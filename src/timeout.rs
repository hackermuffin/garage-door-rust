use tokio::time::sleep;

use crate::state::State;
use crate::Mutex;

pub async fn main(state: &Mutex<State>) {
    let missing_loop_timeout = state.lock().await.consts().missing_loop_interval;
    loop {
        {
            state.lock().await.check_timeout();
        }
        sleep(missing_loop_timeout).await;
    }
}
