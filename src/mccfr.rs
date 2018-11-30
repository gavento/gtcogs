use std::collections::HashMap;
use crate::{Game, PlayerObservation, Probability, ActionIndex, ActivePlayer,
            ObservationVec, Categorical, Strategy, HistoryInfo, History};
use rand::Rng;


#[derive(Clone, Debug)]
pub struct OuterMCCFR<G: Game> {
    pub game: G,
    pub iterations: usize,
    pub nodes_touched: usize,
    table: HashMap<ObservationVec<G>, (Vec<f64>, Vec<f64>)>, // (strategy, regret)
}

impl<G: Game> OuterMCCFR<G> {
    pub fn new(game: G) -> Self {
        OuterMCCFR {
            game,
            iterations: 0,
            nodes_touched: 0,
            table: HashMap::new(),
        }
    }

    pub fn sample_rng<OS, R>(&mut self, rng: &mut R, player: usize, other_strats: &[OS], epsilon: f64) 
        where OS: Strategy<G>, R: Rng {
        self.iterations += 1;
        self.sample_rec(rng, player, other_strats, &self.game.new_history(true), 1.0, 1.0, 1.0, epsilon);
    }

    /// returns (utility, p_tail, p_sample_leaf)
    fn sample_rec<OS: Strategy<G>, R: Rng>(&mut self, rng: &mut R, player: usize, other_strats: &[OS],
                                     hinfo: &HistoryInfo<G>,
                                     p_reach_updated: f64, p_reach_others: f64,
                                     p_sample: f64, epsilon: f64) -> (f64, f64, f64) {
        self.nodes_touched += 1;
        match hinfo.active {
            ActivePlayer::Terminal(ref payoffs) => {
                (payoffs[player], 1.0, p_sample)
            },
            ActivePlayer::Chance(ref cat) => {
                let a = cat.sample_rng(rng);
                let nh = self.game.play(hinfo, a);
                self.sample_rec(rng, player, other_strats, &nh, p_reach_updated,
                                         p_reach_others, p_sample, epsilon)
            },
            ActivePlayer::Player(activep, ref actions) => {
                let activep = activep as usize;
                /*
                let (p_sample, a_sample) = if activep == player {
                    let dist = self.policy(&hinfo.active, &hinfo.observations[activep]);
                    let idx = if rng.next_f64() > epsilon {
                        dist.sample_idx_rng(rng)
                    } else {
                        rng.gen_range(0, actions.len())
                    };
                    (dist.probs()[idx] * (1.0 - epsilon) + epsilon / dist.probs().len(),
                    dist.items()[idx])
                } else {
                    let strat_idx = if activep > player { activep - 1 } else { activep };
                    other_strats[strat_idx]
                        .policy(&hinfo.active, &hinfo.observations[activep])
                        .sample_pair_rng(rng)
                };
                let newinfo = self.game.play(hinfo, a_sample);
                if activep == player {

                } else {

                }*/
                unimplemented!()
            },
        }
    }
}


impl<G: Game> Strategy<G> for OuterMCCFR<G> {
    fn policy(&self, active: &ActivePlayer, obs: &ObservationVec<G>) -> Categorical<ActionIndex> {
        if let ActivePlayer::Player(p, ref actions) = active {
            match self.table.get(obs) {
                None => Categorical::uniform(&actions as &[_]),
                Some(ref d) => Categorical::new_normalized(&d.0 as &[_], &actions as &[_]),
            }
        } else {
            panic!("Policy requested for non-player state {:?}, observed {:?}", active, obs)
        }
    }
}