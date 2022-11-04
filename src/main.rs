use std::{collections::{HashMap, VecDeque}, thread, time::Duration, ops::Add, sync::Arc, i32};

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
            
            state.add_player_to_game(client.id.clone(), client.clone()).await?;

            //let test_string: String = "test".to_string();

            while let Some(Ok(Message::Text(input))) = stream.next().await {
                //let output: String = input.chars().rev().collect();
                let parts: Vec<&str> = input.split(":").collect();

                match parts[0] {
                    "MOVE" => {
                        state.update_grid(&parts[1].to_string(), client.clone()).await;
                    }

                    _ => println!("ERROR: Invalid input")
                }
                

                

                
                //stream
                //    .send_string(format!("{} | {}", &input, &output))
                //    .await?;
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

fn get_path(goal: i32, start: i32) -> VecDeque<i32>
{
    let mut map = VecDeque::from([(goal, 0)]);
    let mut complete = false;

    let mut test = 0;

    while complete == false {

        print!("({}, {}) - ", test, map.len());
        
        
        let current = map[test];
        let adjacent = get_adjacent(current.0);

        print!("map : {} ::", current.0);     

        // for item in &map {
        //     //print!("{}, {} : ", item.0, item.1);
        // }

        for index in adjacent {
            print!("({}): ", index); 
            if index < 0 {
                print!("oob :");
                continue;
            }

            let mut used = false;

            for cell in &map {
                if cell.0 == index {
                    used = true;
                }
            }

            if used == true {
                print!("used : {}, ", used);
                continue;
            }
            
            map.push_back((index, current.1 + 1));

            if index == start {
                complete = true;
                break;
            }

            print!("used : {}, ", used);  
        }

        print!("complete: {},\n ", complete);  

        if test > 300
        {
            break;
        }
        test = test +1;
    }

    let mut i = 0;
    while i < 256 {
        print!("\n");
        let mut p = 0;
        while p < 16 {
            let mut filled = false;
            for cell in &map {
                if cell.0 == i
                {
                    print!("({})", cell.1);
                    filled = true;
                    break;
                }
            }

            if filled == false {
                print!("(xx)");
            }
            
            p = p + 1;
            i = i + 1;
        }
    }

    let mut path = VecDeque::from([]);

    let mut cell = map.pop_back().unwrap();

    let mut found = false;

    let mut next = (0, 50);

    while found == false {
        //print!(":{}, ", cell.0); 
        let adjacent = get_adjacent(cell.0);
        let adjacent_cardinal = get_adjacent_cardinal(cell.0);

        for candidate in &map {
            //print!("\ncandidate: {}", candidate.0);
            if adjacent.contains(&candidate.0) && candidate.1 < next.1 {
                next = *candidate;
            }
        }

        for better_candidate in &map {
            //print!("\ncandidate: {}", candidate.0);
            if adjacent_cardinal.contains(&better_candidate.0) && better_candidate.1 <= next.1 {
                next = *better_candidate;
            }
        }

        //print!("\nnext: {}", next.0);

        if next.0 == goal {
            found = true;
        }
        

        cell = next;

        path.push_back(cell.0);

        //print!("\n");
    }
    
    
    return path;
}

fn get_adjacent(cell: i32) -> [i32; 8] 
{
    let mut cells = [0, 0, 0, 0, 0, 0, 0, 0];

    //north
    if cell < 16 {
        cells[0] = -1;
    }
    else {
        cells[0] = cell - 16;
    }

    //northeast
    if cell < 16 || (cell + 1) % 16 == 0 {
        cells[1] = -1;
    }
    else {
        cells[1] = cell - 15;
    }

    //east
    if (cell + 1) % 16 == 0 {
        cells[2] = -1;
    }
    else{
        cells[2] = cell + 1;
    }

    //southeast
    if (cell + 1) % 16 == 0 || cell > 239 {
        cells[3] = -1;
    }
    else{
        cells[3] = cell + 17;
    }
    
    //south
    if cell > 239 {
        cells[4] = -1;
    }
    else {
        cells[4] = cell + 16;
    }

    //southwest
    if cell % 16 == 0 || cell > 239{
        cells[5] = -1;
    }
    else{
        cells[5] = cell + 15;
    }

    //west
    if cell % 16 == 0 {
        cells[6] = -1;
    }
    else{
        cells[6] = cell - 1;
    }

    //northwest

    if cell % 16 == 0 || cell < 16{
        cells[7] = -1;
    }
    else{
        cells[7] = cell - 17;
    }


    //return [cell - 16, cell - 15, cell + 1, cell + 17, cell + 16, cell + 15, cell - 1, cell -17];

    return cells;
}

fn get_adjacent_cardinal(cell: i32) -> [i32; 4] 
{
    let mut cells = [0, 0, 0, 0];

    //north
    if cell < 16 {
        cells[0] = -1;
    }
    else {
        cells[0] = cell - 16;
    }

    

    //east
    if (cell + 1) % 16 == 0 {
        cells[1] = -1;
    }
    else{
        cells[1] = cell + 1;
    }

   
    
    //south
    if cell > 239 {
        cells[2] = -1;
    }
    else {
        cells[2] = cell + 16;
    }

    

    //west
    if cell % 16 == 0 {
        cells[3] = -1;
    }
    else{
        cells[3] = cell - 1;
    }

   


    //return [cell - 16, cell - 15, cell + 1, cell + 17, cell + 16, cell + 15, cell - 1, cell -17];

    return cells;
}

#[derive(Clone)]
struct State {
    clients: Arc<RwLock<HashMap<String,Client>>>,
    grid: Arc<RwLock<HashMap<i32, String>>>,
}

impl State {
    fn new() -> Self {
        Self {
            clients: Default::default(),
            //grid: Array2D::filled_with("empty".to_string(), 16, 16),
            grid: Default::default(),
        }
    }

    async fn add_player_to_game(&self, client_id: String, client: Client) -> Result<String, std::io::Error> {
        let mut clients = self.clients.write().await;
        clients.insert(client_id, client);
        print!("\n{}", clients.keys().count());
        Ok("Client Inserted".to_string())
    }

    async fn remove_player(&self, client_id: String) -> Result<String, std::io::Error> {
        let mut clients = self.clients.write().await;
        clients.remove(&client_id);
        println!("REMOVED {}", client_id);
        Ok("Client Inserted".to_string())
    }

    async fn send_ticks(&mut self) ->  Result<String, tide_websockets::Error> {
        let clients = self.clients.read().await;
        println!("{}", clients.keys().count());
        
        for (id, client) in clients.iter() {
            if let Err(err) = client.wsc.send_string(self.prepare_output().await).await {
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

    async fn prepare_output(&self) -> String
    {
        let mut output: String = "".to_string();

        let grid = self.grid.read().await;

        let mut i = 0;
        while i < 256 {
            output.push_str(&grid[&i]);
            output.push_str(",");
            i = i + 1;
        }

        // for (_cell, value) in grid.iter() {
        //         output.push_str(&value);
        //         output.push_str(",");
        // }


        //println!("{}", output);

        return output.to_string();
    }

    async fn update_grid(&self, index: &String, client: Client)
    {
        println!("Input: {}", index);
        let index_num = index.parse::<i32>().unwrap();
        let mut current_pos = 0;

        let mut grid = self.grid.write().await;

        let mut i = 0;
        while i < 256 {
            if grid[&i] == client.id
            {
                current_pos = i;
                grid.entry(i).and_modify(|e| {*e = "0".to_string()});
            }
            i = i + 1;
        }

        

        print!("start: {}\n", current_pos);

        let path = get_path(index_num, current_pos);

        for step in &path
        {
            print!(":{}, ", step);
            grid.entry(*step).and_modify(|e| {*e = "2".to_string()});
        }

        grid.entry(*path.back().unwrap()).and_modify(|e| {*e = client.id});

        print!("\n");
        
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
        let mut state = self.state.clone();

        {
            let mut grid = state.grid.write().await;

            let mut i = 0;
    
            while i < 256 {
                grid.insert(i, "0".to_string());
                i = i + 1;
            }
        }
        
    
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
