use tide::{Request, Response};

use crate::state::State;
use crate::Mutex;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 3000;

async fn serve(state: &Mutex<State>, mut _req: Request<()>) -> tide::Result {
    let json = serde_json::to_string_pretty(&*state.lock().await);
    match json {
        Ok(json) => Ok(format!("{}\n", json).into()),
        Err(_) => Ok(Response::new(500)),
    }
}

async fn update(state: &Mutex<State>, mut req: Request<()>) -> tide::Result {
    let data = req.body_bytes().await.unwrap()[0];
    match data as char {
        '1' => {
            state.lock().await.close();
            Ok("Status updated to closed\n".into())
        }
        '0' => {
            state.lock().await.open();
            Ok("Status updated to open\n".into())
        }
        _ => {
            // Invalid update
            state.lock().await.missing();

            let resp = Response::new(418);
            Ok(resp)
        }
    }
}

pub async fn main(state: &'static Mutex<State>) {
    let mut app = tide::new();
    app.at("/").get(|x| serve(state, x));
    app.at("/").post(|x| update(state, x));
    app.listen(format!("{}:{}", ADDR, PORT))
        .await
        .expect("error")
}
