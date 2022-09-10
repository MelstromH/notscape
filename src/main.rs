use tide::{Response, Body, Request};
use dotenv;
use async_std::{prelude::*, stream::Interval};
use tide_websockets::{Message, WebSocket, WebSocketConnection};
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    tide::log::start();

    

    let mut app = tide::with_state(State::new());
    app.at("/")
    .get(|_| async move {
        let mut res = Response::new(201);
        res.set_body(Body::from_file("home/index.html").await?);
        Ok(res)
    });

    app.at("/ws")
        .get(WebSocket::new(|request: Request<State>, mut stream| async move {

            let inter = time::interval(Duration::from_millis(1000));

            let overseer = Overseer::new(stream.clone());
            overseer.start_game(inter).await?;

            while let Some(Ok(Message::Text(input))) = stream.next().await {
                let output: String = input.chars().rev().collect();

                stream
                    .send_string(format!("{} | {}", &input, &output))
                    .await?;
            }

            Ok(())
        }));


    app.at("/script.js")
    .get(|_| async move {
        let mut res = Response::new(201);
        res.set_body(Body::from_file("home/script.js").await?);
        Ok(res)
    });
    
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

#[derive(Clone)]
struct State {
    message: String,
}

impl State {
    fn new() -> Self {
        Self {
            message: "blah".to_string(),
        }
    }
}

#[derive(Clone)]
struct Client {
    id: Option<String>, // connection Id
    wsc : WebSocketConnection,
    label : String
}

#[derive(Clone)]
struct Overseer {
    connection: WebSocketConnection,
}

impl Overseer {
    fn new(wsc: WebSocketConnection) -> Self {
        Self {
            connection: wsc,
        }
    }

    async fn start_game(&self, mut interval: time::Interval) ->tide::Result<()> {
        loop {
            self.connection.send_string("this is a message".to_string()).await?;
            interval.tick().await;
        }
    }
}
