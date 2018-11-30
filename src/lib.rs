extern crate bit_set;
extern crate rand;

use std::fmt::Debug;
use std::hash::Hash;

mod game;
mod goofspiel;
//mod mccfr;
mod distribution;

pub use self::game::{Game, HistoryInfo, PlayerObservation, ActivePlayer};
pub use self::distribution::Categorical;

pub type ActionIndex = u32;
pub type Utility = f64;

pub trait Strategy<G: Game> {
    #[inline]
    fn policy(&self, active: &ActivePlayer<G>, obs: &Vec<PlayerObservation<G::Observation>>) -> Categorical<ActionIndex>;
}

//pub type History = ;

