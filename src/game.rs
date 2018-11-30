use std::fmt::Debug;
use std::hash::Hash;
use std::borrow::Cow;

use crate::{ActionIndex, Categorical, Utility};

pub trait Game: Debug + Clone {
    type State: Clone + Debug;
    type Observation: Clone + Debug + Hash + PartialEq + Eq;
    type Action: Clone + Debug + Hash + PartialEq + Eq;

    fn players(&self) -> usize { 2 }
    fn initial(&self) -> HistoryInfo<Self>;
    fn play(&self, history: &HistoryInfo<Self>, action_index: ActionIndex) -> HistoryInfo<Self>;
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum PlayerObservation<O: Clone + Hash + Debug + PartialEq + Eq> {
    OwnAction(ActionIndex),
    Observation(O),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActivePlayer<G: Game> {
    Player(u32, Vec<G::Action>),
    Chance(Categorical<G::Action>),
    Terminal(Vec<Utility>),
}

#[derive(Clone, Debug)]
pub struct HistoryInfo<G: Game> {
    pub history_indices: Vec<ActionIndex>,
    pub history: Vec<G::Action>,
    pub active: ActivePlayer<G>,
    pub observations: Vec<Vec<PlayerObservation<G::Observation>>>,
    pub state: G::State,
}

impl<G: Game> HistoryInfo<G> {
    pub fn new(game: &G, state: G::State, active: ActivePlayer<G>) -> Self {
        HistoryInfo {
            history_indices: Vec::new(),
            history: Vec::new(),
            observations: vec!{vec!{}; (game.players() + 1) as usize},
            state,
            active,
        }
    }

    pub fn advance(&self, action_index: ActionIndex, new_state: G::State, new_active: ActivePlayer<G>,
                new_observations: Vec<Option<G::Observation>>) -> Self {
        let mut player = None;
        let action: G::Action = match self.active {
            ActivePlayer::Terminal(_) => panic!("play in terminal game state {:?}", self),
            ActivePlayer::Player(p, ref actions) => {
                player = Some(p);
                actions[action_index as usize].clone()
                },
            ActivePlayer::Chance(ref dist) => dist.items()[action_index as usize].clone(),
        };
        let obs = self.observations.iter()
                      .zip(new_observations)
                      .map(|(ovec, o)|
                            if let Some(oin) = o {
                                extended_vec(ovec, PlayerObservation::Observation(oin))
                                // TODO: Own observation if active player
                            } else {
                                ovec.clone()
                            })
                      .collect();
        HistoryInfo {
            history_indices: extended_vec(&self.history_indices, action_index),
            history: extended_vec(&self.history, action),
            observations: obs,
            state: new_state,
            active: new_active,
        }
    }
}

#[inline]
fn extended_vec<T: Clone>(v: &Vec<T>, val: T) -> Vec<T> {
    let mut v2 = Vec::with_capacity(v.len() + 1);
    v2.clone_from(v);
    v2.push(val);
    v2
}