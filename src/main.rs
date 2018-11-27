#![feature(associated_type_defaults)]
extern crate bit_set;

use std::fmt::Debug;
use std::hash::Hash;

type ActionIndex = u16;
type Payoff = f32;
type Probability = f64;

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
enum PlayerObservation<O: Clone + Hash + Debug + PartialEq + Eq> {
    OwnAction(ActionIndex),
    Observation(O),
}

#[derive(Clone, Debug, PartialEq)]
enum ActivePlayer {
    Player(u32, Vec<ActionIndex>),
    Chance(Vec<Probability>, Vec<ActionIndex>),
    Terminal(Vec<Payoff>),
}

//#[derive(Clone, Debug, Hash, PartialEq, Eq)]
type History = Vec<ActionIndex>;

#[derive(Clone, Debug)]
struct HistoryInfo<G: Game> {
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

trait Game: Debug + Clone {
    type StateData: Clone + Default + Debug;
    type Observation: Clone + Debug + Hash + PartialEq + Eq;

    fn new_history(&self, with_observations: bool) -> HistoryInfo<Self> {
        HistoryInfo::new(self, with_observations)
    }

    fn players(&self) -> u32 { 2 }

    fn action_name(&self, history: &HistoryInfo<Self>, action: ActionIndex) -> String {
        action.to_string()
    }

    fn play(&self, hinfo: &HistoryInfo<Self>, action: ActionIndex) -> HistoryInfo<Self> {
        println!("{:?} {:?}", hinfo, action);
        match hinfo.active {
            ActivePlayer::Player(p, ref actions) =>
                debug_assert!(actions.contains(&action)),
            ActivePlayer::Chance(ref probs, ref actions) => {
                debug_assert!(actions.contains(&action));
                debug_assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-3);
            },
            ActivePlayer::Terminal(_) =>
                panic!("Playing {:?} in terminal node {:?}.", action, hinfo),
        };

        let hist = &hinfo.history;
        
        let new_state = self.update_state(&hist, &hinfo.state, action);
        
        let mut new_hist = hist.clone();  // Todo: more efficient copy (prealloc)?
        new_hist.push(action);
        
        let new_active = self.active_player(&new_hist, &new_state);
        
        let mut new_obs = hinfo.observations.clone();
        if !new_obs.is_empty() {
            let obs = self.observations(&new_hist, &new_state);
            for (i, (ref mut ovec, ref o)) in new_obs.iter_mut().zip(obs.iter()).enumerate() {
                if let ActivePlayer::Player(p, _) = hinfo.active {
                    if p as usize == i {
                        ovec.push(PlayerObservation::OwnAction(action));
                    }
                }
                if let Some(oin) = o {
                    ovec.push(PlayerObservation::Observation(oin.clone()));
                }
            }
        }

        HistoryInfo {
            history: new_hist,
            state: new_state,
            observations: new_obs,
            active: new_active,
        }
    }

    fn update_state(&self, history: &History, old_state: &Self::StateData, action: ActionIndex) -> Self::StateData {
        Default::default()
    }

    fn active_player(&self, history: &History, state: &Self::StateData) -> ActivePlayer;

    fn observations(&self, history: &History, state: &Self::StateData) -> Vec<Option<Self::Observation>>;
}


mod goofspiel {

    use super::{HistoryInfo, Game, ActivePlayer, ActionIndex, History, Probability};
    use bit_set::BitSet;

    #[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
    pub enum Scoring {
        ZeroSum,
        WinLoss,
        Absolute,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Goofspiel {
        cards: usize,
        scoring: Scoring,
        card_set: BitSet,
    }

    impl Goofspiel {
        pub fn new(cards: usize, scoring: Scoring) -> Self {
            Goofspiel {
                cards,
                scoring,
                card_set: (1 .. cards + 1).collect(),
            }
        }
    }

    impl Game for Goofspiel {
        type StateData = ();
        type Observation = u16;

        fn active_player(&self, history: &History, _state: &Self::StateData) -> ActivePlayer {
            let len = history.len();
            if len >= self.cards * 3 {
                let mut p0 = 0f32;
                let mut p1 = 0f32;
                for h in history.chunks(3) {
                    if let [c, c0, c1] = h {
                        if c0 > c1 {
                            p0 += *c as f32;
                        }
                        if c1 > c0 {
                            p1 += *c as f32;
                        }
                    }
                }
                let w0 = (p0 - p1).signum();
                ActivePlayer::Terminal(match self.scoring {
                    Scoring::Absolute => vec!(p0, p1),
                    Scoring::WinLoss => vec!(w0, -w0),
                    Scoring::ZeroSum => vec!(p0 - p1, p1 - p0),
                })
            } else {
                let modulo = len % 3;
                let mut cards = self.card_set.clone();
                for i in (modulo .. len).step_by(3) {
                    cards.remove(history[i] as usize);
                };
                let cards: Vec<_> = cards.iter().map(|c| c as ActionIndex).collect();
                if modulo == 0 {
                    let prob = 1.0 / cards.len() as Probability;
                    ActivePlayer::Chance(vec![prob; cards.len()], cards)
                } else {
                    ActivePlayer::Player((modulo - 1) as u32, cards)
                }
            }
        }

        fn observations(&self, history: &History, _state: &Self::StateData) -> Vec<Option<Self::Observation>> {
            let len = history.len();
            println!("OBS {:?}", history);
            vec![match len % 3 {
                1 => Some(history[len - 1]),
                0 if len >= 3 => Some((
                    (history[len - 2] as i16 - history[len - 1] as i16)
                    .signum() + 1) as u16),
                _ => None,
            }; 3]
        }
    }

    mod test {
        use super::{Goofspiel, Scoring, Game, ActivePlayer};
        use super::super::PlayerObservation::*;

        #[test]
        fn test_example_play() {
            for (p0, p1, scoring) in &[
                    (1.0, 5.0, Scoring::Absolute),
                    (-1.0, 1.0, Scoring::WinLoss),
                    (-4.0, 4.0, Scoring::ZeroSum)] {
                let g = Goofspiel::new(4, *scoring);
                let mut hist = g.new_history(true);
                assert_eq!(hist.active, ActivePlayer::Chance(vec![0.25, 0.25, 0.25, 0.25], vec![1, 2, 3, 4]));
                for a in &[2, 1, 2,  3, 2, 4,  4, 3, 3,  1, 4, 1] {
                    hist = g.play(&hist, *a);
                }
                assert_eq!(hist.active, ActivePlayer::Terminal(vec![*p0, *p1]));
                assert_eq!(hist.observations[0], vec![
                    Observation(2), OwnAction(1), Observation(0),
                    Observation(3), OwnAction(2), Observation(0),
                    Observation(4), OwnAction(3), Observation(1),
                    Observation(1), OwnAction(4), Observation(2)]);
                assert_eq!(hist.observations[1], vec![
                    Observation(2), OwnAction(2), Observation(0),
                    Observation(3), OwnAction(4), Observation(0),
                    Observation(4), OwnAction(3), Observation(1),
                    Observation(1), OwnAction(1), Observation(2)]);
                assert_eq!(hist.observations[2], vec![
                    Observation(2), Observation(0),
                    Observation(3), Observation(0),
                    Observation(4), Observation(1),
                    Observation(1), Observation(2)]);
            }
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

