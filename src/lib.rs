#![feature(test)]

extern crate bit_set;
extern crate rand;
extern crate test;

mod distribution;
mod game;
mod goofspiel;
mod history;
mod mccfr;
mod strategy;
mod treegame;

pub use self::distribution::Categorical;
pub use self::game::Game;
pub use self::goofspiel::Goofspiel;
pub use self::history::{ActivePlayer, HistoryInfo, Observation, PlayerObservation};
pub use self::mccfr::{OuterMCCFR, RegretStrategy};
pub use self::strategy::{Strategy, UniformStrategy};
pub use self::treegame::TreeGame;

pub type ActionIndex = u32;
pub type Utility = f64;
