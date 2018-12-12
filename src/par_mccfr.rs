use crate::{
    ActionIndex, ActivePlayer, Categorical, Game, HistoryInfo, PlayerObservation, Strategy,
};
use rand::Rng;
use hashbrown::HashMap;
use rayon::prelude::*;

#[derive(Debug)]
pub struct OuterMCCFR<G: Game> {
    pub game: G,
    pub iterations: usize,
//    pub nodes_traversed: usize,
    pub strategies: Vec<RegretStrategy<G>>,
}

impl<G: Game> OuterMCCFR<G> where
        <G as Game>::Observation: std::marker::Send,
        <G as Game>::Action: std::marker::Send,
        G: std::marker::Sync {
    pub fn new(game: G) -> Self {
        OuterMCCFR {
            iterations: 0,
//            nodes_traversed: 0,
            strategies: (0 .. game.players()).map(|_| RegretStrategy::default()).collect(),
            game,
        }
    }

    pub fn compute_par(&mut self, iterations: usize, epsilon: f64, threads: usize) {
        (0..iterations).into_par_iter().for_each(|_| {
            for player in 0..self.game.players() {
//                self.strategies[player].iterations += 1;
                self.sample_rec(&mut rand::thread_rng(), player, self.game.start(), 1.0, 1.0, 1.0, epsilon);
            }
        });
        self.iterations += iterations;
    }

    /// returns (utility, p_tail, p_sample_leaf)
    fn sample_rec<R: Rng>(
        &self,
        rng: &mut R,
        updated_player: usize,
        hinfo: HistoryInfo<G>,
        p_reach_updated: f64,
        p_reach_others: f64,
        p_sample: f64,
        epsilon: f64,
    ) -> (f64, f64, f64) {
//        self.nodes_traversed += 1;
        match hinfo.active {
            ActivePlayer::Terminal(ref payoffs) => (payoffs[updated_player], 1.0, p_sample),
            ActivePlayer::Chance(ref cat) => {
                let a = cat.sample_idx_rng(rng);
                let nh = self.game.play_owned(hinfo, a);
                self.sample_rec(
                    rng,
                    updated_player,
                    nh,
                    p_reach_updated,
                    p_reach_others,
                    p_sample,
                    epsilon,
                )
            }
            ActivePlayer::Player(player, ref actions) => {
                let player = player as usize;
                let n = actions.len();
                let obs: Vec<PlayerObservation<G>> = hinfo.observations[player].clone();
                let eps = if player == updated_player {
                    epsilon
                } else {
                    0.0
                };
                let subhash = self.strategies[player].table.subhash(&obs);
                let regret: Option<_> = subhash.get(&obs).map(|e| &e.1);
                let dist = match regret {
                    Some(r) => regret_matching(r),
                    None => vec![1.0 / n as f64; n],
                };
                drop(subhash);
                //let policy = self.strategies[player].policy(&hinfo.active, &obs); // regret matching!
                let a_sample = if rng.sample::<f64, _>(rand::distributions::Standard) < eps {
                    rng.gen_range(0, n)
                } else {
                    crate::distribution::sample_weighted(&dist, rng)
                };
                let p_dist = dist[a_sample];
                let p_eps = eps / (n as f64) + (1.0 - eps) * p_dist;

                let newinfo = self.game.play_owned(hinfo, a_sample);
                if player == updated_player {
                    let (payoff, p_tail, p_sample_leaf) = self.sample_rec(
                        rng,
                        updated_player,
                        newinfo,
                        p_reach_updated * p_dist,
                        p_reach_others,
                        p_sample * p_eps,
                        epsilon,
                    );
                    let mut dr = vec![0.0; n];
                    let u = payoff * p_reach_others / p_sample_leaf;
                    for ai in 0..n {
                        if ai == a_sample {
                            dr[ai] = u * (p_tail - p_tail * p_dist);
                        } else {
                            dr[ai] = -u * p_tail * p_dist;
                        }
                    }
                    self.strategies[player].update(obs.clone(), Some(&dr), None);
                    (payoff, p_tail * p_dist, p_sample_leaf)
                } else {
                    let (payoff, p_tail, p_sample_leaf) = self.sample_rec(
                        rng,
                        updated_player,
                        newinfo,
                        p_reach_updated,
                        p_reach_others * p_dist,
                        p_sample * p_eps,
                        epsilon,
                    );
                    let mut ds = dist;
                    ds.iter_mut().for_each(|v| {
                        *v *= p_reach_updated / p_sample_leaf;
                    });
                    self.strategies[player].update(obs.clone(), None, Some(&ds));
                    (payoff, p_tail * p_dist, p_sample_leaf)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RegretStrategy<G: Game> {
//    pub updates: usize,
//    pub iterations: usize,
    table: ParHashMap<Vec<PlayerObservation<G>>, (Vec<f64>, Vec<f64>)>, // (strategy, regret)
    phantom: std::marker::PhantomData<G>,
}

impl<G: Game> Default for RegretStrategy<G> {
    fn default() -> Self {
        RegretStrategy {
//            iterations: 0,
//            updates: 0,
            table: ParHashMap::new(128),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G: Game> RegretStrategy<G> {
    pub fn update(
        &self,
        obs: Vec<PlayerObservation<G>>,
        d_reg: Option<&[f64]>,
        d_strat: Option<&[f64]>,
    ) {
//        self.updates += 1;
        let len = d_reg
            .or(d_strat)
            .expect("Pass at least one of d_reg, d_strat to update")
            .len();
        let mut subhash = self.table.subhash(&obs);
        let entry = subhash.entry(obs);
        let val = entry.or_insert_with(|| (vec![0.0; len], vec![0.0; len]));
        if let Some(d) = d_strat {
            if len != d.len() {
                panic!("Passed d_reg and d_strat must have same length.")
            }
            for (ve, de) in val.0.iter_mut().zip(d) {
                *ve += de;
            }
        }
        if let Some(d) = d_reg {
            for (ve, de) in val.1.iter_mut().zip(d) {
                *ve += de;
            }
        }
    }
}

impl<G: Game> Strategy<G> for RegretStrategy<G> {
    fn policy(
        &self,
        active: &ActivePlayer<G>,
        obs: &Vec<PlayerObservation<G>>,
    ) -> Categorical<ActionIndex> {
        if let ActivePlayer::Player(_p, ref actions) = active {
            match self.table.subhash(obs).get(obs) {
                None => Categorical::uniform((0..actions.len() as ActionIndex).collect::<Vec<_>>()),
                Some(ref d) => {
                    let vs = (0..actions.len() as ActionIndex).collect::<Vec<_>>();
                    let ps = &(*d).0 as &[_];
                    if ps.iter().sum::<f64>() < 1e-6 {
                        Categorical::uniform(vs)
                    } else {
                        Categorical::new_normalized(ps, vs)
                    }
                }
            }
        } else {
            panic!(
                "strategy requested for non-player state {:?}, observed {:?}",
                active, obs
            )
        }
    }
}

fn regret_matching(reg: &[f64]) -> Vec<f64> {
    let regp = reg.iter().map(|&v| if v >= 0.0 { v } else { 0.0 });
    let s = regp.clone().sum::<f64>();
    let l = reg.len();
    if s > 0.0 {
        regp.map(|v| v / s).collect()
    } else {
        vec![1.0 / l as f64; l]
    }
}

use std::sync::{Mutex, MutexGuard};
use std::hash::Hash;
use std::borrow::Borrow;


#[derive(Debug)]
struct ParHashMap<K: Hash + Eq, V> {
    hashes: Vec<Mutex<HashMap<K, V>>>,
}

impl<K: Hash + Eq, V> ParHashMap<K, V> {
    pub fn new(slots: usize) -> Self {
        ParHashMap { hashes: (0 .. slots).map(|_| Mutex::new(HashMap::new())).collect() }
    }

    fn subhash<'a>(&'a self, key: &K) -> MutexGuard<'a, HashMap<K, V>> {
        let h = fxhash::hash64(key) as usize;
        self.hashes[h % self.hashes.len()].lock().expect("failed locking ParHash")
    }
}


#[cfg(test)]
mod test {
    use crate::{goofspiel, Game, Goofspiel, Strategy};
    use super::OuterMCCFR;
    use rand::{rngs::SmallRng, SeedableRng};

    #[test]
    fn test_goof3_par_mccfr() {
        let g = Goofspiel::new(3, goofspiel::Scoring::ZeroSum);
        let mut mc = OuterMCCFR::new(g.clone());
        let mut rng = SmallRng::seed_from_u64(1);
        mc.compute_par(5000, 0.6, 4);
        let s = g.start();
        let s = g.play_owned(s, 1);
        let pol = mc.strategies[0].policy(&s.active, &s.observations[0]);
        assert!(pol.probs()[1] > 0.8);
        let s = g.play_owned(s, 1);
        let pol = mc.strategies[1].policy(&s.active, &s.observations[1]);
        assert!(pol.probs()[1] > 0.8);
    }
}
