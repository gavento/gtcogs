#![feature(test)]
extern crate gtcogs;
extern crate rand;
extern crate test;

use gtcogs::{goofspiel, Goofspiel, OuterMCCFR, TreeGame};
use rand::{rngs::SmallRng, SeedableRng};
use test::Bencher;

#[bench]
fn bench_os_mccfr_goofspiel4(b: &mut Bencher) {
    let g = Goofspiel::new(4, goofspiel::Scoring::ZeroSum);
    let mut mc = OuterMCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(1);
    b.iter(|| mc.compute_rng(1, 0.6, &mut rng));
}

#[bench]
fn bench_os_mccfr_goofspiel5(b: &mut Bencher) {
    let g = Goofspiel::new(5, goofspiel::Scoring::ZeroSum);
    let mut mc = OuterMCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(2);
    b.iter(|| mc.compute_rng(1, 0.6, &mut rng));
}

#[bench]
fn bench_os_mccfr_goofspiel4_tree(b: &mut Bencher) {
    let g = Goofspiel::new(4, goofspiel::Scoring::ZeroSum);
    let t = TreeGame::from_game(&g);
    let mut mc = OuterMCCFR::new(t);
    let mut rng = SmallRng::seed_from_u64(3);
    b.iter(|| mc.compute_rng(1, 0.6, &mut rng));
}

#[bench]
fn bench_os_mccfr_goofspiel5_tree(b: &mut Bencher) {
    let g = Goofspiel::new(5, goofspiel::Scoring::ZeroSum);
    let t = TreeGame::from_game(&g);
    let mut mc = OuterMCCFR::new(t);
    let mut rng = SmallRng::seed_from_u64(4);
    b.iter(|| mc.compute_rng(1, 0.6, &mut rng));
}
