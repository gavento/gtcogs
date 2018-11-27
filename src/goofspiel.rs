use crate::{ActionIndex, ActivePlayer, Game, History, HistoryInfo, Probability, Categorical};
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
            card_set: (1..cards + 1).collect(),
        }
    }
}

impl Game for Goofspiel {
    type StateData = ();
    type Observation = u16;

    fn active_player(&self, history: &History, _state: &Self::StateData) -> ActivePlayer {
        let len = history.len();
        if len >= self.cards * 3 {
            let mut p0 = 0f64;
            let mut p1 = 0f64;
            for h in history.chunks(3) {
                if let [c, c0, c1] = h {
                    if c0 > c1 {
                        p0 += *c as f64;
                    }
                    if c1 > c0 {
                        p1 += *c as f64;
                    }
                }
            }
            let w0 = (p0 - p1).signum();
            ActivePlayer::Terminal(match self.scoring {
                Scoring::Absolute => vec![p0, p1],
                Scoring::WinLoss => vec![w0, -w0],
                Scoring::ZeroSum => vec![p0 - p1, p1 - p0],
            })
        } else {
            let modulo = len % 3;
            let mut cards = self.card_set.clone();
            for i in (modulo..len).step_by(3) {
                cards.remove(history[i] as usize);
            }
            let cards: Vec<_> = cards.iter().map(|c| c as ActionIndex).collect();
            if modulo == 0 {
                ActivePlayer::Chance(Categorical::uniform(cards))
            } else {
                ActivePlayer::Player((modulo - 1) as u32, cards)
            }
        }
    }

    fn observations(
        &self,
        history: &History,
        _state: &Self::StateData,
    ) -> Vec<Option<Self::Observation>> {
        let len = history.len();
        println!("OBS {:?}", history);
        vec![
            match len % 3 {
                1 => Some(history[len - 1]),
                0 if len >= 3 => Some(
                    ((history[len - 2] as i16 - history[len - 1] as i16).signum() + 1) as u16
                ),
                _ => None,
            };
            3
        ]
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
            let mut hist = g.new_history(true);
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
