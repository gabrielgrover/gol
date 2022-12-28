mod board;
mod cell;
mod game_of_life;
mod utility;

pub use cell::{generate_live_cells, Cell};
pub use game_of_life::{GameOfLife, SubscribeMessage};
