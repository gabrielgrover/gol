use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use game_of_life_lib::{generate_live_cells, Cell, GameOfLife, SubscribeMessage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let game = Arc::new(GameOfLife::new(500, 500, generate_live_cells()));

    let app = Router::new()
        .route(
            "/",
            get({
                let _shared_game = Arc::clone(&game);
                move || async { "Hello world!" }
            }),
        )
        .route("/ws", get(|ws| handle_socket(ws)));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
enum GOLSocketMessage {
    GetCells,
    SendCells { cells: Vec<Cell> },
    Subscribe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientSubsriptionMessage {
    Subscribe,
    MessageFromServer {
        cells: Vec<Cell>,
        generation_index: i32,
    },
    StartSim,
    None,
}

async fn handle_socket(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket))
}

async fn websocket(mut ws: WebSocket) {
    let game = GameOfLife::new(500, 500, generate_live_cells());
    let (gol_sub_tx, mut gol_sub_rx) = mpsc::channel::<SubscribeMessage>(100);

    game.run_sim()
        .await
        .expect("Failed to start Game of Life simulation");

    loop {
        let msg = tokio::select! {
            Some(msg) = gol_sub_rx.recv() => {
                //println!("message from gol: {:?}", msg);
                parse_gol_subscription_message(msg)
            },
            Some(Ok(msg)) = ws.recv() =>
                parse_subscription_message_from_client(msg)
            ,
            else => { break },
        };

        match msg {
            ClientSubsriptionMessage::Subscribe => {
                //println!("subscribing to gol");

                let _ = game.subscribe(gol_sub_tx.clone()).await;
            }

            ClientSubsriptionMessage::MessageFromServer {
                cells,
                generation_index,
            } => {
                for batched_cells in cells.chunks(100) {
                    let client_sub_message = ClientSubsriptionMessage::MessageFromServer {
                        cells: batched_cells.to_vec(),
                        generation_index,
                    };
                    let json_str = serde_json::to_string(&client_sub_message)
                        .expect("Failed to stringify message client message");
                    let message_to_client = Message::Text(json_str);

                    ws.send(message_to_client)
                        .await
                        .expect("Failed to send message to client");
                }
            }

            ClientSubsriptionMessage::StartSim => {}

            _ => {}
        }
    }
}

fn parse_subscription_message_from_client(message: Message) -> ClientSubsriptionMessage {
    match message {
        Message::Text(client_sent_msg) => {
            let parsed_client_message =
                serde_json::from_str::<ClientSubsriptionMessage>(&client_sent_msg)
                    .expect("Failed to parse client message");

            parsed_client_message
        }

        _ => ClientSubsriptionMessage::None,
    }
}

fn parse_gol_subscription_message(message: SubscribeMessage) -> ClientSubsriptionMessage {
    match message {
        SubscribeMessage::OnChange {
            cells,
            generation_index,
        } => ClientSubsriptionMessage::MessageFromServer {
            cells: cells
                .iter()
                .map(|c| {
                    if Cell::is_alive(c) {
                        Cell::birth(c)
                    } else {
                        Cell::kill(c)
                    }
                })
                .collect(),
            generation_index,
        },

        _ => ClientSubsriptionMessage::None,
    }
}
