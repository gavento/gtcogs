use crate::{ActionIndex, ActivePlayer, Categorical, Game, PlayerObservation};

pub trait Strategy<G: Game> {
    #[inline]
    fn policy(
        &self,
        active: &ActivePlayer<G>,
        obs: &Vec<PlayerObservation<G>>,
    ) -> Categorical<ActionIndex>;
}

#[derive(Clone, Debug)]
pub struct UniformStrategy {}

impl<G: Game> Strategy<G> for UniformStrategy {
    fn policy(
        &self,
        active: &ActivePlayer<G>,
        _obs: &Vec<PlayerObservation<G>>,
    ) -> Categorical<ActionIndex> {
        if let ActivePlayer::Player(_p, ref actions) = active {
            Categorical::uniform((0..actions.len() as u32).collect::<Vec<_>>())
        } else {
            panic!("strategy requested for non-player state {:?}", active)
        }
    }
}
