use tokio::time::{sleep, Duration};

use crate::STATE;

pub async fn main() {
    loop {
        {
            STATE.lock().await.check_timeout();
        }
        sleep(Duration::from_secs(10)).await;
    }
}
