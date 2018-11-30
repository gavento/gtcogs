use crate::{ActionIndex, ActivePlayer, Game, HistoryInfo, Categorical, Utility};
use bit_set::BitSet;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Scoring {
    ZeroSum,
    WinLoss,
    Absolute,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Goofspiel {
    cards: usize,
    scoring: Scoring,
    card_set: BitSet,
    values: Vec<Utility>,
}

impl Goofspiel {
    pub fn new(cards: usize, scoring: Scoring) -> Self {
        Goofspiel {
            cards,
            scoring,
            card_set: (1..cards + 1).collect(),
            values: (1..cards + 1).map(|x| x as Utility).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    cards: [BitSet; 3],
    scores: [f64; 2],
}

impl Game for Goofspiel {
    type State = State;
    type Observation =i32;
    type Action = u32;

    fn initial(&self) -> HistoryInfo<Self> {
        let state = State {
            cards: [self.card_set.clone(), self.card_set.clone(), self.card_set.clone()],
            scores: [0.0, 0.0],
        };
        HistoryInfo::new(self, state, ActivePlayer::Chance(Categorical::uniform(self.card_set.iter().map(|x| x as u32).collect::<Vec<_>>())))
    }

    fn play(&self, history: &HistoryInfo<Self>, action_index: ActionIndex) -> HistoryInfo<Self> {
        let mut state = history.state.clone();
        let len = history.history.len();
        let obs: Option<_> = match history.active {
            ActivePlayer::Chance(ref cat) => {
                let action = cat.items()[action_index as usize];
                state.cards[0].remove(action as usize);
                Some(action as i32)
            },
            ActivePlayer::Player(p, ref acts) => {
                let action = acts[action_index as usize];
                state.cards[p as usize].remove(action as usize);
                if p == 2 {
                    let bet = self.values[history.history[len - 2] as usize];
                    let winner = ((action as i32) - (history.history[len - 1] as i32)).signum();
                    if winner == 1 {
                        state.scores[0] += bet;
                    }
                    if winner == -1 {
                        state.scores[1] += bet;
                    }
                    Some(winner)
                } else {
                    None
                }
            },
            ActivePlayer::Terminal(_) => {
                panic!()
            }
        };
        let active = if len + 1 == self.cards * 3 {
            let d = state.scores[0] - state.scores[1];
            ActivePlayer::Terminal(match self.scoring {
                Scoring::Absolute => state.scores.as_ref().into(),
                Scoring::ZeroSum => { vec![d, -d] },
                Scoring::WinLoss => { vec![d.signum(), -d.signum()] }
            })            
        } else {
            let p = (len + 1) % 3;
            let acts = state.cards[p].iter().map(|x| x as u32).collect();
            if p == 0 {
                ActivePlayer::Chance(Categorical::uniform(acts))
            } else {
                ActivePlayer::Player(p as u32, acts)
            }
        };
        history.advance(action_index, state, active, vec![obs; 3])
    }
}

mod test {
    use super::super::PlayerObservation::*;
    use super::{ActivePlayer, Game, Goofspiel, Scoring, Categorical};

    #[test]
    fn test_example_play() {
        for (p0, p1, scoring) in &[
            (1.0, 5.0, Scoring::Absolute),
            (-1.0, 1.0, Scoring::WinLoss),
            (-4.0, 4.0, Scoring::ZeroSum),
        ] {
            let g = Goofspiel::new(4, *scoring);
            let mut hist = g.initial();
            assert_eq!(
                hist.active,
                ActivePlayer::Chance(Categorical::uniform(vec![1, 2, 3, 4]))
            );
            for a in &[2, 1, 2, 3, 2, 4, 4, 3, 3, 1, 4, 1] {
                hist = g.play(&hist, *a);
            }
            assert_eq!(hist.active, ActivePlayer::Terminal(vec![*p0, *p1]));
            assert_eq!(
                hist.observations[0],
                vec![
                    Observation(2),
                    OwnAction(1),
                    Observation(0),
                    Observation(3),
                    OwnAction(2),
                    Observation(0),
                    Observation(4),
                    OwnAction(3),
                    Observation(1),
                    Observation(1),
                    OwnAction(4),
                    Observation(2)
                ]
            );
            assert_eq!(
                hist.observations[1],
                vec![
                    Observation(2),
                    OwnAction(2),
                    Observation(0),
                    Observation(3),
                    OwnAction(4),
                    Observation(0),
                    Observation(4),
                    OwnAction(3),
                    Observation(1),
                    Observation(1),
                    OwnAction(1),
                    Observation(2)
                ]
            );
            assert_eq!(
                hist.observations[2],
                vec![
                    Observation(2),
                    Observation(0),
                    Observation(3),
                    Observation(0),
                    Observation(4),
                    Observation(1),
                    Observation(1),
                    Observation(2)
                ]
            );
        }
    }
}
