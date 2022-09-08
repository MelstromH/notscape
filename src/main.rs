use tide::{Response, Body};
use dotenv;
use async_std::{prelude::*};
use tide_websockets::{Message, WebSocket};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    tide::log::start();

    let mut app = tide::new();
    app.at("/")
    .with(WebSocket::new(|_request, mut stream| async move {
        while let Some(Ok(Message::Text(input))) = stream.next().await {
            
            let output = format!("YO! {}", input);
            stream
                .send_string(output)
                .await?;
        }

        Ok(())
    }))
    .get(|_| async move {
        let mut res = Response::new(201);
        res.set_body(Body::from_file("home/index.html").await?);
        Ok(res)
    });


    app.at("/script.js")
    .get(|_| async move {
        let mut res = Response::new(201);
        res.set_body(Body::from_file("home/script.js").await?);
        Ok(res)
    });
    
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
