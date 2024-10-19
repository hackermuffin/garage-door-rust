use log::{debug, trace};
use tokio::time::sleep;

use crate::state::State;
use crate::Mutex;

pub async fn main(state: &Mutex<State>) {
    let missing_loop_timeout = state.lock().await.consts().missing_loop_interval;
    debug!("Starting timeout loop with timeout {missing_loop_timeout:?}...");
    loop {
        trace!("Checking timeout...");
        {
            state.lock().await.check_timeout();
        }
        trace!("Timeout waiting {missing_loop_timeout:?} before sleeping again...");
        sleep(missing_loop_timeout).await;
    }
}
