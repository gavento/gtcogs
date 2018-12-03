use crate::{Game, Categorical, Utility, ActionIndex};

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum PlayerObservation<G: Game> {
    OwnAction(G::Action),
    Observation(G::Observation),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActivePlayer<G: Game> {
    Player(u32, Vec<G::Action>),
    Chance(Categorical<G::Action>),
    Terminal(Vec<Utility>),
}

impl<G: Game> ActivePlayer<G> {
    #[inline]
    pub fn actions<'a>(&'a self) -> &'a [G::Action] {
        match self {
            ActivePlayer::Terminal(_) => &[],
            ActivePlayer::Player(_, ref actions) => actions,
            ActivePlayer::Chance(ref dist) => dist.items(),
        }
    }

    #[inline]
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
    pub observations: Vec<Vec<PlayerObservation<G>>>,
    pub state: G::State,
}

impl<G: Game> HistoryInfo<G> {
    pub fn new(game: &G, state: G::State, active: ActivePlayer<G>) -> Self {
        HistoryInfo {
            history_indices: Vec::new(),
            history: Vec::new(),
            observations: vec![vec!{}; (game.players() + 1) as usize],
            state,
            active,
        }
    }

    pub fn observations_since<'a>(&'a self, other: &Self) -> Vec<&'a[PlayerObservation<G>]> {
        self.observations.iter().zip(other.observations.iter()).map(|(so, oo)| {
            &so[oo.len() ..]
        }).collect()
    }
}
