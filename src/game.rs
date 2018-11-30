use std::fmt::Debug;
use std::hash::Hash;
use std::borrow::Cow;

use crate::{ActionIndex, Categorical, Utility};

pub trait Game: Debug + Clone {
    type State: Clone + Debug;
    type Observation: Clone + Debug + Hash + PartialEq + Eq;
    type Action: Clone + Debug + Hash + PartialEq + Eq;

    /// TO BE IMPLEMENTED BY PLAYER:
    fn players(&self) -> usize;
    fn initial_state(&self) -> (Self::State, ActivePlayer<Self>);
    /// TODO: Give the prev_active?
    /// TODO: Give prev_history and action separately?
    fn update_state(&self, state: &Self::State, history: &Vec<Self::Action>, prev_active: &ActivePlayer<Self>) -> 
        (Self::State, ActivePlayer<Self>, Vec<Option<Self::Observation>>);

    fn start(&self) -> HistoryInfo<Self> {
        let (state, active) = self.initial_state();
        HistoryInfo::new(self, state, active)
    }

    fn play_value(&self, hist: &HistoryInfo<Self>, action: &Self::Action) -> HistoryInfo<Self> {
        if let ActivePlayer::Terminal(_) = hist.active {
            panic!("playing in terminal state {:?}", hist);
        }
        match hist.active.actions().iter().position(|x| x == action) {
            Some(idx) => self.play(&hist, idx),
            None => panic!("action {:?} not available in state {:?}", action, hist)
        }
    }

    fn play(&self, hist: &HistoryInfo<Self>, action_index: usize) -> HistoryInfo<Self> {
        debug_assert_eq!(hist.observations.len(), self.players() + 1);
        if let ActivePlayer::Terminal(_) = hist.active {
            panic!("playing in terminal state {:?}", hist);
        }
        let prev_player = hist.active.player();
        let action = hist.active.actions()
                         .get(action_index as usize)
                         .expect("action index outside action list").clone();
        let history_indices = extended_vec(&hist.history_indices, action_index as ActionIndex);
        let history = extended_vec(&hist.history, action);
        // Observation extensions by own action
        let mut observations = hist.observations.clone();
        if let Some(p) = prev_player {
            observations[p].push(PlayerObservation::OwnAction(action_index as u32));
        }
        // Game-specific logic
        let (state, active, obs) = self.update_state(&hist.state, &history, &hist.active);
        debug_assert_eq!(obs.len(), self.players() + 1);
        if let ActivePlayer::Player(p, _) = active {
            debug_assert!((p as usize) < self.players());
        }
        // Observation extension by new observed
        for (ovec, option_ob) in observations.iter_mut().zip(obs) {
            if let Some(ob) = option_ob {
                ovec.push(PlayerObservation::Observation(ob))
            }
        }
        HistoryInfo {
            history_indices,
            history,
            observations,
            state,
            active,
        }
    }


    ////////// Alternatively, offer to reuse the state, observations, history, etc.
    // Reuse variant of play()
    fn play_owned(&self, hist: HistoryInfo<Self>, action_index: usize) -> HistoryInfo<Self> {
        unimplemented!()
    }
    // Match on Cow
    fn play_cow(&self, hist: Cow<HistoryInfo<Self>>, action_index: usize) -> HistoryInfo<Self> {
        match hist {
            Cow::Borrowed(r) => self.play(r, action_index),
            Cow::Owned(h) => self.play_owned(h, action_index),
        }
    }

    // The player would optionally implement state reuse with (only this instead of update_state)
    fn update_state_cow(&self, state: Cow<Self::State>, history: &Vec<Self::Action>) -> 
        (Self::State, ActivePlayer<Self>, Vec<Option<Self::Observation>>)
    {
        // If you want to modify an existing state
        let mut state2 = state.into_owned();
        // Otherwise just use state as &State via Deref
        unimplemented!()
    }
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

impl<G: Game> ActivePlayer<G> {
    pub fn actions<'a>(&'a self) -> &'a [G::Action] {
        match self {
            ActivePlayer::Terminal(_) => &[],
            ActivePlayer::Player(p, ref actions) => actions,
            ActivePlayer::Chance(ref dist) => dist.items(),
        }
    }
    pub fn player<'a>(&'a self) -> Option<usize> {
        match self {
            ActivePlayer::Terminal(_) => None,
            ActivePlayer::Player(p, _) => Some(*p as usize),
            ActivePlayer::Chance(_) => None,
        }
    }
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
}

#[inline]
fn extended_vec<T: Clone>(v: &Vec<T>, val: T) -> Vec<T> {
    let mut v2 = Vec::with_capacity(v.len() + 1);
    v2.clone_from(v);
    v2.push(val);
    v2
}