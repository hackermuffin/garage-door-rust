use tokio::time::{sleep, Duration};

use crate::state::State;
use crate::Mutex;

pub async fn main(state: &Mutex<State>) {
    loop {
        {
            state.lock().await.check_timeout();
        }
        sleep(Duration::from_secs(10)).await;
    }
}
