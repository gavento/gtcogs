extern crate bit_set;
extern crate rand;

use std::fmt::Debug;
use std::hash::Hash;

mod game;
mod goofspiel;
mod mccfr;
mod distribution;

pub use self::game::Game;
pub use self::distribution::Categorical;

pub type ActionIndex = u16;
pub type Payoff = f64;
pub type Probability = f64;

pub trait Strategy<G: Game> {
    #[inline]
    fn policy(&self, active: &ActivePlayer, obs: &ObservationVec<G>) -> Categorical<ActionIndex>;
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum PlayerObservation<O: Clone + Hash + Debug + PartialEq + Eq> {
    OwnAction(ActionIndex),
    Observation(O),
}

pub type ObservationVec<G: Game> = Vec<PlayerObservation<G::Observation>>;

#[derive(Clone, Debug, PartialEq)]
pub enum ActivePlayer {
    Player(u32, Vec<ActionIndex>),
    Chance(Categorical<ActionIndex>),
    Terminal(Vec<Payoff>),
}

pub type History = Vec<ActionIndex>;

#[derive(Clone, Debug)]
pub struct HistoryInfo<G: Game> {
    pub history: History,
    pub active: ActivePlayer,
    pub observations: Vec<ObservationVec<G>>,
    pub state: G::StateData,
}

impl<G: Game> HistoryInfo<G> {
    pub fn new_with_state(game: &G, with_observations: bool, state: G::StateData) -> Self {
        let history = Vec::new();
        let active = game.active_player(&history, &state);
        HistoryInfo {
            history,
            observations: vec![vec![]; if with_observations { (game.players() + 1) as usize } else { 0 }],
            state,
            active,
        }
    }

    pub fn new(game: &G, with_observations: bool) -> Self {
        Self::new_with_state(game, with_observations, G::StateData::default())
    }
}
