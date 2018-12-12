use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng, Rng,
};

#[derive(Clone, Debug)]
pub struct Categorical<T>(Vec<f64>, WeightedIndex<f64>, Vec<T>);

impl<T: PartialEq> PartialEq for Categorical<T> {
    fn eq(&self, r: &Self) -> bool {
        self.0 == r.0 && self.2 == r.2
    }
}

impl<T> Categorical<T> {
    #[inline]
    pub fn items<'a>(&'a self) -> &'a Vec<T> {
        &self.2
    }

    #[inline]
    pub fn probs<'a>(&'a self) -> &'a Vec<f64> {
        &self.0
    }

    #[inline]
    pub fn sample_idx_rng<'a, R: Rng>(&'a self, rng: &mut R) -> usize {
        self.1.sample(rng)
    }

    #[inline]
    pub fn sample_ref_rng<'a, R: Rng>(&'a self, rng: &mut R) -> &'a T {
        &self.2[self.1.sample(rng)]
    }

    #[inline]
    pub fn sample_ref<'a>(&'a self) -> &'a T {
        self.sample_ref_rng(&mut thread_rng())
    }

    #[inline]
    pub fn rample_ref_pair_rng<'a, R: Rng>(&'a self, rng: &mut R) -> (f64, &'a T) {
        let idx = self.1.sample(rng);
        (self.0[idx], &self.2[idx])
    }

    pub fn uniform<IT: Into<Vec<T>>>(items: IT) -> Self {
        let is: Vec<T> = items.into();
        let l = is.len();
        Self::new(vec![1.0 / (l as f64); l], is)
    }

    pub fn new_normalized<IT: Into<Vec<T>>, IP: Into<Vec<f64>>>(probs: IP, items: IT) -> Self {
        let mut ps: Vec<f64> = probs.into();
        let s: f64 = ps.iter().sum();
        ps.iter_mut().for_each(|p| {
            *p = *p / s;
        });
        Self::new(ps, items)
    }

    pub fn new<IT: Into<Vec<T>>, IP: Into<Vec<f64>>>(probs: IP, items: IT) -> Self {
        let ps: Vec<f64> = probs.into();
        let is: Vec<T> = items.into();
        assert_eq!(ps.len(), is.len());
        debug_assert!((ps.iter().sum::<f64>() - 1.0) < 1e-3);
        let wi = WeightedIndex::new(&ps).expect("invalid distribution");
        Categorical(ps, wi, is)
    }

    /*
    pub fn epsilon_smooth(&mut self, epsilon: f64) {
        assert!(epsilon >= 0.0 && epsilon <= 1.0);
        self.0.iter_mut().for_each(|mut pr| *pr = *pr * (1.0 - epsilon) + epsilon / self.2.len() as f64);
        self.1 = WeightedIndex::new(self.0.clone()).expect("invalid distribution");
    }*/
}

impl<T: Clone> Categorical<T> {
    #[inline]
    pub fn sample_rng<R: Rng>(&self, rng: &mut R) -> T {
        self.sample_ref_rng(rng).clone()
    }

    #[inline]
    pub fn sample(&self) -> T {
        self.sample_rng(&mut thread_rng())
    }

    #[inline]
    pub fn sample_pair_rng<'a, R: Rng>(&'a self, rng: &mut R) -> (f64, T) {
        let idx = self.1.sample(rng);
        (self.0[idx], self.2[idx].clone())
    }
}

pub fn sample_weighted<R: Rng>(ps: &[f64], rng: &mut R) -> usize {
    debug_assert!((ps.iter().sum::<f64>() - 1.0).abs() < 1e-6);
    let mut s: f64 = rng.sample(rand::distributions::Standard);
    for (i, p) in ps.iter().enumerate() {
        s -= p;
        if s < 0.0 {
            return i;
        }
    }
    return ps.len() - 1;
}
