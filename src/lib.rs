extern crate bit_set;
extern crate rand;

//mod mccfr;
mod distribution;
mod game;
mod goofspiel;
mod history;
mod strategy;
mod treegame;

pub use self::distribution::Categorical;
pub use self::game::Game;
pub use self::goofspiel::Goofspiel;
pub use self::history::{ActivePlayer, HistoryInfo, PlayerObservation, Observation};
pub use self::strategy::{Strategy, UniformStrategy};
pub use self::treegame::TreeGame;

pub type ActionIndex = u32;
pub type Utility = f64;

