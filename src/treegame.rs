use crate::{ActionIndex, ActivePlayer, Categorical, Game, HistoryInfo};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TreeGame<Obs>
where
    Obs: Clone + Debug + PartialEq + Hash + Eq,
{
    players: usize,
    tree: Arc<TreeGameNode<Obs>>,
    obs_index: HashMap<Obs, usize>,
}

#[derive(Clone, Debug)]
pub struct TreeGameNode<Obs>
where
    Obs: Clone + Debug + PartialEq + Hash + Eq,
{
    active: ActivePlayer<TreeGame<Obs>>,
    children: Vec<Arc<TreeGameNode<Obs>>>,
    observations: Vec<Option<usize>>,
}

impl<Obs> Game for TreeGame<Obs>
where
    Obs: Clone + Debug + PartialEq + Hash + Eq,
{
    type State = Arc<TreeGameNode<Obs>>;
    type Observation = usize;
    type Action = ActionIndex;

    fn players(&self) -> usize {
        self.players
    }

    fn initial_state(&self) -> (Self::State, ActivePlayer<Self>) {
        (self.tree.clone(), self.tree.active.clone())
    }

    fn update_state(
        &self,
        hist: &HistoryInfo<Self>,
        action: &Self::Action,
    ) -> (
        Self::State,
        ActivePlayer<Self>,
        Vec<Option<Self::Observation>>,
    ) {
        let node = &hist.state.children[*action as usize];
        (node.clone(), node.active.clone(), node.observations.clone())
    }
}

impl<Obs> TreeGame<Obs>
where
    Obs: Clone + Debug + PartialEq + Hash + Eq,
{
    pub fn from_game<G: Game<Observation = Obs>>(game: &G) -> Self {
        let mut obs_index: HashMap<G::Observation, usize> = HashMap::new();
        let (s0, a0) = game.initial_state();
        TreeGame {
            players: game.players(),
            tree: traverse_game(
                game,
                &game.start(),
                vec![None; game.players() + 1],
                &mut obs_index,
            ),
            obs_index,
        }
    }
}

fn traverse_game<Obs, G: Game<Observation = Obs>>(
    game: &G,
    hist: &HistoryInfo<G>,
    last_obs: Vec<Option<usize>>,
    obs_index: &mut HashMap<Obs, usize>,
) -> Arc<TreeGameNode<Obs>>
where
    Obs: Clone + Debug + PartialEq + Hash + Eq,
{
    let ch: Vec<_> = hist
        .active
        .actions()
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let (_, _, new_obs) = game.update_state(hist, a);
            let h2 = game.play(hist, i); // TODO: remove duplicate evaluation
            let obs_len = obs_index.len();
            let obs = new_obs
                .into_iter()
                .map(|o| o.map(|someo| *obs_index.entry(someo).or_insert(obs_len + 1)))
                .collect();
            traverse_game(game, &h2, obs, obs_index)
        })
        .collect();
    let active = match hist.active {
        ActivePlayer::Terminal(ref t) => ActivePlayer::Terminal(t.clone()),
        ActivePlayer::Player(p, ref a) => {
            ActivePlayer::Player(p, (0..a.len() as ActionIndex).collect())
        }
        ActivePlayer::Chance(ref d) => ActivePlayer::Chance(Categorical::new(
            d.probs().clone(),
            (0..d.items().len() as ActionIndex).collect::<Vec<_>>(),
        )),
    };
    Arc::new(TreeGameNode {
        active,
        children: ch,
        observations: last_obs,
    })
}

#[cfg(test)]
mod test {
    use crate::{goofspiel, Game, TreeGame, ActivePlayer};

    #[test]
    fn treegame_goofspiel() {
        let g = goofspiel::Goofspiel::new(3, goofspiel::Scoring::Absolute);
        let t = TreeGame::from_game(&g);
        let mut h = t.start();
        for ai in &[0, 1, 0, 0, 0, 0, 0, 0, 0] {
            h = t.play_owned(h, *ai);
        }
        assert_eq!(h.active.actions(), &[]);
        if let ActivePlayer::Terminal(ref u) = h.active {
            assert_eq!(u, &[1.0, 2.0]);
        } else {
            panic!("expected terminal node");
        }
    }
}