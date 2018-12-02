extern crate bit_set;
extern crate rand;

use std::fmt::Debug;
use std::hash::Hash;

mod game;
mod goofspiel;
//mod mccfr;
mod distribution;
mod history;

pub use self::distribution::Categorical;
pub use self::game::{Game};
pub use self::history::{ActivePlayer, HistoryInfo, PlayerObservation};

pub type ActionIndex = u32;
pub type Utility = f64;

pub trait Strategy<G: Game> {
    #[inline]
    fn policy(
        &self,
        active: &ActivePlayer<G>,
        obs: &Vec<PlayerObservation<G>>,
    ) -> Categorical<ActionIndex>;
}

//pub type History = ;
