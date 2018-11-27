use std::fmt::Debug;
use std::hash::Hash;

use crate::{ActionIndex, ActivePlayer, History, HistoryInfo, PlayerObservation};

pub trait Game: Debug + Clone {
    type StateData: Clone + Default + Debug;
    type Observation: Clone + Debug + Hash + PartialEq + Eq;

    fn new_history(&self, with_observations: bool) -> HistoryInfo<Self> {
        HistoryInfo::new(self, with_observations)
    }

    fn players(&self) -> u32 {
        2
    }

    fn action_name(&self, history: &HistoryInfo<Self>, action: ActionIndex) -> String {
        action.to_string()
    }

    fn play(&self, hinfo: &HistoryInfo<Self>, action: ActionIndex) -> HistoryInfo<Self> {
        println!("{:?} {:?}", hinfo, action);
        match hinfo.active {
            ActivePlayer::Player(p, ref actions) => debug_assert!(actions.contains(&action)),
            ActivePlayer::Chance(ref dist) => {
                debug_assert!(dist.items().contains(&action));
            }
            ActivePlayer::Terminal(_) => {
                panic!("Playing {:?} in terminal node {:?}.", action, hinfo)
            }
        };

        let hist = &hinfo.history;

        let new_state = self.update_state(&hist, &hinfo.state, action);

        let mut new_hist = hist.clone(); // Todo: more efficient copy (prealloc)?
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

    fn update_state(
        &self,
        history: &History,
        old_state: &Self::StateData,
        action: ActionIndex,
    ) -> Self::StateData {
        Default::default()
    }

    fn active_player(&self, history: &History, state: &Self::StateData) -> ActivePlayer;

    fn observations(
        &self,
        history: &History,
        state: &Self::StateData,
    ) -> Vec<Option<Self::Observation>>;
}
