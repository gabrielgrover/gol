use crate::{board::Board, cell::Cell, utility::generate_id};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time,
};

pub struct GameOfLife {
    sim_tx: Sender<GOLMessage>,
}

#[derive(Debug)]
enum GOLMessage {
    Run,
    Pause,
    Resume,
    Subscribe {
        sender: Sender<SubscribeMessage>,
        id: String,
    },
}

#[derive(Debug)]
pub enum SubscribeMessage {
    OnChange {
        cells: Arc<HashSet<Cell>>,
        generation_index: i32,
    },
    OnPause {
        cells: Arc<HashSet<Cell>>,
        generation_index: i32,
    },
    OnResume {
        cells: Arc<HashSet<Cell>>,
        generation_index: i32,
    },
    Done,
    None,
}

impl GameOfLife {
    pub fn new(max_rows: i32, max_cols: i32, live_cells: HashSet<Cell>) -> Self {
        let (sim_tx, sim_rx) = channel::<GOLMessage>(100);

        let board = birth_universe(max_rows, max_cols, live_cells);

        tokio::spawn(run_gol_simulation(sim_rx, sim_tx.clone(), board));

        Self { sim_tx }
    }

    pub async fn run_sim(&self) -> Result<(), String> {
        self.sim_tx
            .send(GOLMessage::Run)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn pause(&self) -> Result<(), String> {
        self.sim_tx
            .send(GOLMessage::Pause)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn resume(&self) -> Result<(), String> {
        self.sim_tx
            .send(GOLMessage::Resume)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn subscribe(&self, subscription_tx: Sender<SubscribeMessage>) -> String {
        let subscriber_id = generate_id();

        let message = GOLMessage::Subscribe {
            sender: subscription_tx,
            id: subscriber_id.clone(),
        };

        self.sim_tx
            .send(message)
            .await
            .expect("failed to send subscribe message");

        subscriber_id
    }
}

fn birth_universe(rows: i32, cols: i32, live_cells: HashSet<Cell>) -> Board {
    let board = Board::new(rows, cols);

    board.run_cells_transform(|cells| {
        let mut new_cells = HashSet::new();

        for cell in cells.iter() {
            if live_cells.contains(cell) {
                new_cells.insert(Cell::birth(cell));
            } else {
                new_cells.insert(Cell::kill(cell));
            }
        }

        new_cells
    })
}

async fn run_gol_simulation(
    mut receiver: Receiver<GOLMessage>,
    sender: Sender<GOLMessage>,
    board: Board,
) {
    let mut running = true;
    let mut subscribers = HashMap::<String, Sender<SubscribeMessage>>::new();
    let mut cells = Arc::new(board.take_cells());
    let mut generation_index: i32 = 0;

    while let Some(msg) = receiver.recv().await {
        match msg {
            GOLMessage::Run => {
                if !running {
                    continue;
                }

                cells = Arc::new(cells_transform(&cells));

                generation_index += 1;

                for (_id, sub) in subscribers.iter() {
                    sub.send(SubscribeMessage::OnChange {
                        cells: Arc::clone(&cells),
                        generation_index,
                    })
                    .await
                    .expect("Failed to send subscription message.");
                }

                time::sleep(Duration::from_millis(200)).await;

                sender
                    .send(GOLMessage::Run)
                    .await
                    .expect("Failed to send GOLMessage::Run message");
            }

            GOLMessage::Resume => {
                running = true;

                for (_id, sub) in subscribers.iter() {
                    sub.send(SubscribeMessage::OnResume {
                        cells: Arc::clone(&cells),
                        generation_index,
                    })
                    .await
                    .expect("Failed to send OnResume message");
                }

                sender
                    .send(GOLMessage::Run)
                    .await
                    .expect("Failed to send GOLMessage::Run message");
            }

            GOLMessage::Pause => {
                running = false;

                for (_id, sub) in subscribers.iter() {
                    sub.send(SubscribeMessage::OnPause {
                        cells: Arc::clone(&cells),
                        generation_index,
                    })
                    .await
                    .expect("Failed to send OnPause message.");
                }
            }

            GOLMessage::Subscribe { sender, id } => {
                subscribers.insert(id, sender);
            }
        }
    }
}

fn cells_transform(previous_generation: &HashSet<Cell>) -> HashSet<Cell> {
    let mut new_generation = HashSet::new();

    for cell in previous_generation.iter() {
        let cell_tuple = Cell::to_tuple(cell);
        let cell_is_alive = Cell::is_alive(cell);

        let living_neighbors: Vec<_> = get_neighbors(cell_tuple)
            .iter()
            .filter_map(|(r, c)| {
                previous_generation
                    .get(&Cell::new(*r, *c))
                    .and_then(|neighbor_cell| {
                        if Cell::is_alive(neighbor_cell) {
                            Some(neighbor_cell)
                        } else {
                            None
                        }
                    })
            })
            .collect();

        let living_neighbor_count = living_neighbors.len();

        if cell_is_alive && (living_neighbor_count == 2 || living_neighbor_count == 3) {
            new_generation.insert(Cell::birth(cell));
        } else if !cell_is_alive && living_neighbor_count == 3 {
            new_generation.insert(Cell::birth(cell));
        } else {
            new_generation.insert(Cell::kill(cell));
        }
    }

    new_generation
}

fn get_neighbors(c: (i32, i32)) -> Vec<(i32, i32)> {
    let n = (1, 0);
    let s = (-1, 0);
    let w = (0, -1);
    let e = (0, 1);

    let ne = (1, 1);
    let nw = (1, -1);
    let se = (-1, 1);
    let sw = (-1, -1);

    let moves = vec![n, s, w, e, ne, nw, se, sw];

    let (r, c) = c;

    moves.iter().map(|(m0, m1)| (m0 + r, m1 + c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_live_cells;

    #[tokio::test]
    async fn should_be_able_to_subscribe_and_get_correct_data() {
        let live_cells = generate_live_cells();
        let game = GameOfLife::new(100, 100, live_cells.clone());

        let (tx, mut rx) = channel::<SubscribeMessage>(8);

        let _ = game.subscribe(tx.clone()).await;

        game.run_sim().await.expect("Failed to start GOL sim");

        time::sleep(Duration::from_millis(200)).await;

        let msg = rx.try_recv().expect("Failed to get message");

        if let SubscribeMessage::OnChange {
            cells,
            generation_index,
        } = msg
        {
            let initial_conditions = birth_universe(100, 100, live_cells).take_cells();
            let expected_experiment_result =
                (0..generation_index).fold(initial_conditions, |acc, _cs| cells_transform(&acc));

            for received_cell in cells.iter() {
                let expected_cell = expected_experiment_result
                    .get(received_cell)
                    .expect("Cell should have twin");

                assert_eq!(
                    Cell::is_alive(received_cell),
                    Cell::is_alive(&expected_cell)
                );
            }
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn should_receive_on_pause_messaage() {
        let game = GameOfLife::new(100, 100, generate_live_cells());
        let (tx, mut rx) = channel::<SubscribeMessage>(8);
        let _ = game.subscribe(tx.clone()).await;

        game.run_sim().await.expect("Failed to start GOL sim");

        time::sleep(Duration::from_millis(200)).await;

        game.pause().await.expect("Failed to pause sim");

        while let Some(msg) = rx.recv().await {
            match msg {
                SubscribeMessage::OnPause {
                    cells: _,
                    generation_index: _,
                } => {
                    break;
                }

                _ => {
                    continue;
                }
            }
        }

        assert!(true);
    }

    #[tokio::test]
    async fn should_resume_after_pause() {
        let live_cells = generate_live_cells();
        let game = GameOfLife::new(100, 100, live_cells.clone());

        let (tx, mut rx) = channel::<SubscribeMessage>(8);

        let _ = game.subscribe(tx.clone()).await;

        game.run_sim().await.expect("Failed to start GOL sim");

        time::sleep(Duration::from_millis(200)).await;

        game.pause().await.expect("Failed to pause sim");

        let t_0 = async move {
            let mut cells: Arc<HashSet<Cell>> = Arc::new(HashSet::new());
            let mut idx: i32 = 0;

            while let Some(received_msg) = rx.recv().await {
                match received_msg {
                    SubscribeMessage::OnPause {
                        cells: cs,
                        generation_index: i,
                    } => {
                        cells = cs;
                        idx = i;
                        break;
                    }

                    _ => {
                        continue;
                    }
                }
            }

            println!("{:?}", idx);

            (cells, idx, rx)
        }
        .await;

        game.resume().await.expect("Failed to resume GOL sim");

        let (cells_t0, generation_index_t0, mut rx_0) = t_0;

        let (on_resume_msg, mut rx_1) = async move {
            let mut msg = SubscribeMessage::None;

            while let Some(received_msg) = rx_0.recv().await {
                match received_msg {
                    SubscribeMessage::OnResume {
                        cells: _,
                        generation_index: _,
                    } => {
                        msg = received_msg;

                        break;
                    }

                    _ => {
                        continue;
                    }
                }
            }

            (msg, rx_0)
        }
        .await;

        match on_resume_msg {
            SubscribeMessage::OnResume {
                cells,
                generation_index,
            } => {
                assert_eq!(generation_index, generation_index_t0);

                for received_cell in cells.iter() {
                    let expected_cell = (&cells_t0)
                        .get(received_cell)
                        .expect("Failed to retrieve cell");

                    assert_eq!(Cell::is_alive(received_cell), Cell::is_alive(expected_cell));
                }
            }
            _ => assert!(false),
        }

        let on_change_msg = async move {
            let mut msg = SubscribeMessage::None;

            while let Some(received_msg) = rx_1.recv().await {
                match received_msg {
                    SubscribeMessage::OnChange {
                        cells: _,
                        generation_index: _,
                    } => {
                        msg = received_msg;

                        break;
                    }

                    _ => {
                        continue;
                    }
                }
            }

            msg
        }
        .await;

        match on_change_msg {
            SubscribeMessage::OnChange {
                cells,
                generation_index,
            } => {
                assert_eq!(generation_index, generation_index_t0 + 1);

                let expected_experiment_result = cells_transform(&cells_t0);

                for received_cell in cells.iter() {
                    let expected_cell = expected_experiment_result
                        .get(received_cell)
                        .expect("Cell should have twin");

                    assert_eq!(
                        Cell::is_alive(received_cell),
                        Cell::is_alive(&expected_cell)
                    );
                }
            }

            _ => assert!(false),
        }
    }
}
