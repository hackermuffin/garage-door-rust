use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use http_types::Request;
use http_types::{
    Method::{Get, Post},
    Response, StatusCode,
};
use log::{debug, error, info, trace, warn};

use crate::state::State;
use crate::Mutex;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 3000;

pub async fn main(state: &'static Mutex<State>) -> http_types::Result<()> {
    // Open up a TCP connection and create a URL.
    let listener = TcpListener::bind((ADDR, PORT)).await?;
    let addr = format!("http://{}", listener.local_addr()?);

    info!("Web server listening on {}", addr);

    // For each incoming TCP connection, spawn a task and call `accept`.
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(async {
            if let Err(err) = accept(state, stream).await {
                error!("Web server error: {}", err);
            }
        });
    }
    Ok(())
}

// Take a TCP stream, and convert it into sequential HTTP request / response pairs.
async fn accept(state: &'static Mutex<State>, stream: TcpStream) -> http_types::Result<()> {
    trace!("Acception web connection from {}", stream.peer_addr()?);
    let opts = async_h1::ServerOptions::default().with_default_host("localhost");
    async_h1::accept_with_opts(
        stream.clone(),
        |req| async move {
            match req.method() {
                Get => serve(state).await,
                Post => update(state, req).await,
                _ => Response::new(StatusCode::MethodNotAllowed),
            }
        },
        opts,
    )
    .await?;
    Ok(())
}

async fn serve(state: &'static Mutex<State>) -> Response {
    debug!("Get request recieved...");
    let json = serde_json::to_string_pretty(&*state.lock().await);
    match json {
        Ok(json) => {
            debug!("Responding with {json}");
            json.into()
        }
        Err(e) => {
            error!("Error generating json state: {e}");
            Response::new(500)
        }
    }
}

async fn update(state: &'static Mutex<State>, mut req: Request) -> Response {
    trace!("Post request recieved");
    let data = req.body_bytes().await.unwrap()[0];
    match data as char {
        '1' => {
            trace!("Updating state to closed");
            state.lock().await.close();
            "Status updated to closed\n".into()
        }
        '0' => {
            trace!("Updating state to open");
            state.lock().await.open();
            "Status updated to open\n".into()
        }
        _ => {
            // Invalid update
            warn!("Invalid data recieved: {data}, updating to missing.");
            state.lock().await.missing();
            Response::new(418)
        }
    }
}
