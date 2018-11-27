extern crate bit_set;

use std::fmt::Debug;
use std::hash::Hash;

mod game;
mod goofspiel;

pub use self::game::Game;

pub type ActionIndex = u16;
pub type Payoff = f32;
pub type Probability = f64;

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum PlayerObservation<O: Clone + Hash + Debug + PartialEq + Eq> {
    OwnAction(ActionIndex),
    Observation(O),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActivePlayer {
    Player(u32, Vec<ActionIndex>),
    Chance(Vec<Probability>, Vec<ActionIndex>),
    Terminal(Vec<Payoff>),
}

//#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub type History = Vec<ActionIndex>;

#[derive(Clone, Debug)]
pub struct HistoryInfo<G: Game> {
    pub history: History,
    pub active: ActivePlayer,
    pub observations: Vec<Vec<PlayerObservation<G::Observation>>>,
    pub state: G::StateData,
}

impl<G: Game> HistoryInfo<G> {
    pub fn new_with_state(game: &G, with_observations: bool, state: G::StateData) -> Self {
        let mut observations = Vec::new();
        if with_observations {
            observations.resize((game.players() + 1) as usize, Vec::new())
        }
        let history = Vec::new();
        let active = game.active_player(&history, &state);
        HistoryInfo {
            history,
            observations,
            state,
            active,
        }
    }

    pub fn new(game: &G, with_observations: bool) -> Self {
        Self::new_with_state(game, with_observations, G::StateData::default())
    }
}
