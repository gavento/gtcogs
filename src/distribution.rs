use rand::{Rng, thread_rng, distributions::{Distribution, WeightedIndex}};

#[derive(Clone, Debug)]
pub struct Categorical<T>(Vec<f64>, WeightedIndex<f64>, Vec<T>);

impl<T: PartialEq> PartialEq for Categorical<T> {
    fn eq(&self, r: &Self) -> bool {
        self.0 == r.0 && self.2 == r.2
    }
}

impl<T> Categorical<T> {
    #[inline]
    pub fn sample_ref_rng<'a, R: Rng>(&'a self, rng: &mut R) -> &'a T {
        &self.2[self.1.sample(rng)]
    }

    pub fn items<'a>(&'a self) -> &'a Vec<T> {
        &self.2
    }

    #[inline]
    pub fn sample_ref<'a, R: Rng>(&'a self) -> &'a T {
        self.sample_ref_rng(&mut thread_rng())
    }

    pub fn uniform<IT: Into<Vec<T>>>(items: IT) -> Self {
        let is: Vec<T> = items.into();
        let l = is.len();
        Self::new(vec![1.0 / (l as f64); l], is)
    }

    pub fn new_normalized<IT: Into<Vec<T>>, IP: Into<Vec<f64>>>(probs: IP, items: IT) -> Self {
        let mut ps: Vec<f64> = probs.into();
        let s: f64 = ps.iter().sum();
        ps.iter_mut().for_each(|p| { *p = *p / s; });
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
}

