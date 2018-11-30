use bit_set::BitSet;
use crate::{ActionIndex, ActivePlayer, Categorical, Game, HistoryInfo, Utility};

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

/// Players are p0 and p1, p2 is chance
#[derive(Clone, Debug)]
pub struct State {
    cards: [BitSet; 3],
    scores: [f64; 2],
}

impl Game for Goofspiel {
    type State = State;
    type Observation = i32;
    type Action = u32;

    fn players(&self) -> usize {
        2
    }

    fn initial_state(&self) -> (Self::State, ActivePlayer<Self>) {
        let state = State {
            cards: [
                self.card_set.clone(),
                self.card_set.clone(),
                self.card_set.clone(),
            ],
            scores: [0.0, 0.0],
        };
        let active = ActivePlayer::Chance(Categorical::uniform(
            self.card_set.iter().map(|x| x as u32).collect::<Vec<_>>(),
        ));
        (state, active)
    }

    fn update_state(
        &self,
        prev_state: &Self::State,
        history: &Vec<Self::Action>,
        prev_active: &ActivePlayer<Self>,
    ) -> (
        Self::State,
        ActivePlayer<Self>,
        Vec<Option<Self::Observation>>,
    ) {
        // p=0, p=1 are players p=2 is chance
        let len = history.len();
        let prev_player = (history.len() + 1) % 3;
        let next_player = (history.len() - 1) % 3;
        let action = *history.last().unwrap();
        let mut obs = None;
        // Play the selected card, update state
        let mut state = prev_state.clone();
        state.cards[prev_player].remove(action as usize);
        // Score update and observation
        if prev_player == 1 {
            let bet = self.values[(history[len - 3] - 1) as usize];
            let winner = ((history[len - 2] as i32) - (action as i32)).signum();
            if winner == 1 {
                state.scores[0] += bet;
            }
            if winner == -1 {
                state.scores[1] += bet;
            }
            obs = Some(winner);
        }
        // Observe public card
        if prev_player == 2 {
            obs = Some(action as i32);
        }
        // Terminal reached or active player
        let active = if len == self.cards * 3 {
            let d = state.scores[0] - state.scores[1];
            ActivePlayer::Terminal(match self.scoring {
                Scoring::Absolute => state.scores.as_ref().into(),
                Scoring::ZeroSum => vec![d, -d],
                Scoring::WinLoss => vec![d.signum(), -d.signum()],
            })
        } else {
            let acts = state.cards[next_player].iter().map(|x| x as u32).collect();
            if next_player == 2 {
                ActivePlayer::Chance(Categorical::uniform(acts))
            } else {
                ActivePlayer::Player(next_player as u32, acts)
            }
        };
        // Return new info triplet
        (state, active, vec![obs; 3])
    }
}

mod test {
    use super::super::PlayerObservation::*;
    use super::{ActivePlayer, Categorical, Game, Goofspiel, Scoring};

    #[test]
    fn test_example_play() {
        for (p0, p1, scoring) in &[
            (1.0, 5.0, Scoring::Absolute),
            (-1.0, 1.0, Scoring::WinLoss),
            (-4.0, 4.0, Scoring::ZeroSum),
        ] {
            let g = Goofspiel::new(4, *scoring);
            let mut hist = g.start();
            assert_eq!(
                hist.active,
                ActivePlayer::Chance(Categorical::uniform(vec![1, 2, 3, 4]))
            );
            for a in &[2, 1, 2, 3, 2, 4, 4, 3, 3, 1, 4, 1] {
                hist = g.play_value(&hist, a);
            }
            assert_eq!(hist.active, ActivePlayer::Terminal(vec![*p0, *p1]));
            assert_eq!(
                hist.observations[0],
                vec![
                    Observation(2),
                    OwnAction(1),
                    Observation(-1),
                    Observation(3),
                    OwnAction(2),
                    Observation(-1),
                    Observation(4),
                    OwnAction(3),
                    Observation(0),
                    Observation(1),
                    OwnAction(4),
                    Observation(1)
                ]
            );
            assert_eq!(
                hist.observations[1],
                vec![
                    Observation(2),
                    OwnAction(2),
                    Observation(-1),
                    Observation(3),
                    OwnAction(4),
                    Observation(-1),
                    Observation(4),
                    OwnAction(3),
                    Observation(0),
                    Observation(1),
                    OwnAction(1),
                    Observation(1)
                ]
            );
            assert_eq!(
                hist.observations[2],
                vec![
                    Observation(2),
                    Observation(-1),
                    Observation(3),
                    Observation(-1),
                    Observation(4),
                    Observation(0),
                    Observation(1),
                    Observation(1)
                ]
            );
        }
    }
}
