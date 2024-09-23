use tide::{Request, Response};

use crate::STATE;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 3000;

async fn serve(mut _req: Request<()>) -> tide::Result {
    Ok(format!("{:?}\n", STATE.lock().await).into())
}

async fn update(mut req: Request<()>) -> tide::Result {
    let data = req.body_bytes().await.unwrap()[0];
    match data as char {
        '0' => {
            STATE.lock().await.close();
            Ok("Status updated to closed\n".into())
        }
        '1' => {
            STATE.lock().await.open();
            Ok("Status updated to open\n".into())
        }
        _ => {
            // Invalid update
            STATE.lock().await.missing();

            let resp = Response::new(418);
            Ok(resp)
        }
    }
}

pub async fn main() {
    let mut app = tide::new();
    app.at("/").get(serve);
    app.at("/").post(update);
    app.listen(format!("{}:{}", ADDR, PORT))
        .await
        .expect("error")
}
