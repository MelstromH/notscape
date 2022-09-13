use std::{collections::{HashMap}, thread, time::Duration, ops::Add, sync::Arc};

use tide::{Response, Body, Request};
use dotenv;
use async_std::{prelude::*, sync::RwLock};
use tide_websockets::{Message, WebSocket, WebSocketConnection, async_tungstenite};
use tokio::{runtime::Runtime};

#[tokio::main]
async fn main() -> Result<(), tide::Error> {
    dotenv::dotenv().ok();
    tide::log::start();

    let state = State::new();
    let overseer = Overseer::new(state.clone());
    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        overseer.start_game().await.unwrap();
    });

   

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
            {
                let clients = state.clients.read().await;
                client.id = clients.keys().count().add(1).to_string();
            }
            
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
    clients: Arc<RwLock<HashMap<String,Client>>>,
}

impl State {
    fn new() -> Self {
        Self {
            clients: Default::default(),
        }
    }

    async fn add_player_to_game(&self, client_id: String, client: Client) -> Result<String, std::io::Error> {
        let mut clients = self.clients.write().await;
        clients.insert(client_id, client);
        println!("{}", clients.keys().count());
        Ok("Client Inserted".to_string())
    }

    async fn remove_player(&self, client_id: String) -> Result<String, std::io::Error> {
        let mut clients = self.clients.write().await;
        clients.remove(&client_id);
        println!("REMOVED {}", client_id);
        Ok("Client Inserted".to_string())
    }

    async fn send_ticks(&self) ->  Result<String, tide_websockets::Error> {
        let clients = self.clients.read().await;
        println!("{}", clients.keys().count());
        for (id, client) in clients.iter() {
            if let Err(err) = client.wsc.send_string(id.to_string()).await {
                match err {
                    tide_websockets::Error::ConnectionClosed => {
                        return Ok(id.to_string())
                    },
                    _ => return Err(err)
                }
            }
        }
        Ok("ticks sent".to_string())
        
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

    async fn start_game(&self) -> async_tungstenite::tungstenite::Result<()> {
        let state = self.state.clone();
    
        loop {
            match state.send_ticks().await {
                Ok(r) => if r != "ticks sent".to_string() {
                    state.remove_player(r).await?;
                }
                Err(e) => return Err(e)
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}