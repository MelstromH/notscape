use std::{collections::{HashMap, hash_map::{Entry}}, io::{Error, ErrorKind}};

use tide::{Response, Body, Request};
use dotenv;
use async_std::{prelude::*};
use tide_websockets::{Message, WebSocket, WebSocketConnection};
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), tide::Error> {
    dotenv::dotenv().ok();
    tide::log::start();

    let inter = time::interval(Duration::from_millis(1000));
    let state = State::new();
    let overseer = Overseer::new(state.clone());

    tokio::spawn(async move {
        overseer.start_game(inter).await;
    }).await.unwrap();

    let mut app = tide::with_state(state);



    app.at("/")
    .get(|_| async move {
        let mut res = Response::new(201);
        res.set_body(Body::from_file("home/index.html").await?);
        Ok(res)
    });

    app.at("/ws")
        .get(WebSocket::new(|request: Request<State>, mut stream| async move {

            let state = request.state().clone();
            let mut client = Client::new(stream.clone());
            client.id = request.query().unwrap_or_default();

            state.add_player_to_game(client.id.to_owned(), client).await?;

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
    clients: HashMap<String, Client>,
}

impl State {
    fn new() -> Self {
        Self {
            clients: Default::default(),
        }
    }

    async fn add_player_to_game(&self, client_id: String, client: Client) -> Result<String, std::io::Error> {
        let mut clients = self.clients.clone();
        match clients.entry(client_id.to_owned()){
            Entry::Vacant(_) =>
            {
                clients.insert(client_id.to_owned(), client);
                
                Ok(String::from("new client added"))
            },
            Entry::Occupied(_) =>
            {
                return Err(Error::new(ErrorKind::NotConnected, format!("client {} already connected", client_id)));
            }
        }
    }

    async fn send_ticks(&self) ->  Result<(), tide::Error> {
        for (id, client) in &self.clients {
            client.wsc.send_string(id.to_string()).await?;
        }
        Ok(())
    }
    
}

#[derive(Clone)]
struct Client {
    id: String, // connection Id
    wsc : WebSocketConnection,
}

impl Client{
    fn new(stream : WebSocketConnection) -> Self {
        Self {
            id: Default::default(),
            wsc: stream,
        }
    }
}

#[derive(Clone)]
struct Overseer {
    state: State,
}

impl Overseer {
    fn new(state: State) -> Self {
        Self {
            state: state,
        }
    }

    async fn start_game(&self, mut interval: time::Interval) -> Result<(), tide::Error>  {
        loop {
            self.state.send_ticks().await?;
            interval.tick().await;
        }
    }
}