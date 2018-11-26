#![feature(associated_type_defaults)]
extern crate bit_set;

use std::fmt::Debug;
use std::hash::Hash;

type ActionIndex = u16;
type Payoff = f32;
type Probability = f64;

#[derive(Clone, Hash, Debug)]
enum PlayerObservation<O: Clone + Hash + Debug> {
    OwnAction(ActionIndex),
    Observation(O),
}

#[derive(Clone, Debug, PartialEq)]
enum ActivePlayer {
    Player(u32, Vec<ActionIndex>),
    Chance(Vec<(Probability, ActionIndex)>),
    Terminal(Vec<Payoff>),
}

#[derive(Clone, Debug)]
struct History<G: Game> {
    history: Vec<ActionIndex>,
    observations: Vec<Vec<PlayerObservation<G::Observation>>>,
    state: G::StateData,
}

impl<G: Game> History<G> {
    pub fn new_with_state(game: &G, with_observations: bool, state: G::StateData) -> Self {
        let mut observations = Vec::new();
        if with_observations {
            observations.resize((game.players() + 1) as usize, Vec::new())
        }
        History {
            history: Vec::new(),
            observations,
            state,
        }
    }

    pub fn new(game: &G, with_observations: bool) -> Self {
        Self::new_with_state(game, with_observations, G::StateData::default())
    }
}

trait Game: Clone {
    type StateData: Clone + Default + Debug = ();
    type Observation: Clone + Debug + Hash = u16;

    fn new_history(&self, with_observations: bool) -> History<Self> {
        History::new(self, with_observations)
    }

    fn players(&self) -> u32 { 2 }

    fn action_name(&self, history: &History<Self>, action: ActionIndex) -> String {
        action.to_string()
    }

    fn play(&self, history: &History<Self>, action: ActionIndex) -> History<Self> {
        let mut new_hist = history.history.clone();  // Todo: more efficient copy (prealloc)?
        new_hist.push(action);
        let new_state = self.update_state(history, action);
        let mut new_obs = history.observations.clone();
        if !new_obs.is_empty() {
            let obs = self.observations(&history.history, &new_state);
            for (ref mut ovec, ref o) in new_obs.iter_mut().zip(obs.iter()) {
                if let Some(oin) = o {
                    ovec.push(PlayerObservation::Observation(oin.clone()));
                }
            }
        }
        History {
            history: new_hist,
            state: new_state,
            observations: new_obs,
        }
    }

    fn update_state(&self, _history: &History<Self>, _action: ActionIndex) -> Self::StateData {
        Default::default()
    }

    fn active_player(&self, history: &History<Self>) -> ActivePlayer;

    fn observations(&self, new_seq: &Vec<ActionIndex>, new_state: &Self::StateData) -> Vec<Option<Self::Observation>>;
}

mod goofspiel {

    use super::{History, Game, ActivePlayer, ActionIndex};

    #[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
    enum Scoring {
        ZeroSum,
        WinLoss,
        Absolute,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Goofspiel {
        cards: u32,
        scoring: Scoring,
    }

    impl Goofspiel {
        pub fn new(cards: u32, scoring: Scoring) -> Self {
            Goofspiel {cards, scoring}
        }
    }

    impl Game for Goofspiel {
        fn active_player(&self, state: &History<Self>) -> ActivePlayer {
            /*
            let h = &state.history;
            let len = h.len() as u32;
            if len >= self.cards * 3 {
                return ActivePlayer::Terminal();
            }
            v = 
            if len % 3 == 0 {
                return ActivePlayer::Chance(v.iter().map(|a| (1.0 v.len(), a)).collect());
            }
            return ActivePlayer::Player(i % 3 - 1, v);*/
            unimplemented!()
        }

        fn observations(&self, new_seq: &Vec<ActionIndex>, new_state: &Self::StateData) -> Vec<Option<Self::Observation>> {
            unimplemented!()
        }
    }
}



#[derive(Debug)]
struct X<T: Debug> { a: T, }

fn main() {
    let d: &dyn Debug = (&42u32) as &Debug;
    let x1: X<&dyn Debug> = X {a: d};
    let x2: X<i64> = X {a: -42};
    let x3: X<&i64> = X {a: &-42};
    println!("Hello, world {:?} {:?} {:?}!", x1, x2, x3);
}

