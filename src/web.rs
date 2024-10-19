use log::{error, trace};
use tide::{Request, Response};

use crate::state::State;
use crate::Mutex;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 3000;

async fn serve(state: &Mutex<State>, mut _req: Request<()>) -> tide::Result {
    trace!("Get request recieved...");
    let json = serde_json::to_string_pretty(&*state.lock().await);
    match json {
        Ok(json) => {
            trace!("Responding with {json}");
            Ok(format!("{}\n", json).into())
        }
        Err(_) => {
            error!("Failed to generate json of internal state!");
            Ok(Response::new(500))
        }
    }
}

async fn update(state: &Mutex<State>, mut req: Request<()>) -> tide::Result {
    trace!("Post request recieved");
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
    trace!("Starting web server...");
    let mut app = tide::new();
    app.at("/").get(|x| serve(state, x));
    app.at("/").post(|x| update(state, x));
    app.listen(format!("{}:{}", ADDR, PORT))
        .await
        .expect("Web server crashed!")
}
