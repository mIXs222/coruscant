use std::collections::BTreeMap;
use chashmap::CHashMap;
use std::hash::Hash;


type BernSummary = f64;
pub type CategoryBernSummary<T> = BTreeMap<T, BernSummary>;


/* Single-state transition model */
#[derive(Default, Clone, Debug)]
struct BernEstimator {
    total_count: usize,
    event_count: usize,
}

impl BernEstimator {
    fn observe_absent(&mut self) {
        self.total_count += 1;
    }

    fn observe_present(&mut self) {
        self.total_count += 1;
        self.event_count += 1;
    }

    fn summarize(self) -> BernSummary {
        self.event_count as f64 / self.total_count as f64
    }
}


/* Transition model between multiple states */
#[derive(Default, Clone, Debug)]
pub struct CategoryBernEstimator<T> {
    state_models: CHashMap<T, BernEstimator>
}

impl<T> CategoryBernEstimator<T> 
where T: PartialEq + Eq + Hash + Ord + Default + std::fmt::Debug
{
    pub fn observe_absent(&self, state: T) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe_absent()
        } else {
            let mut bm = BernEstimator::default();
            bm.observe_absent();
            self.state_models.insert_new(state, bm)
        }
    }

    pub fn observe_present(&self, state: T) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe_present()
        } else {
            let mut bm = BernEstimator::default();
            bm.observe_present();
            self.state_models.insert_new(state, bm)
        }
    }

    pub fn summarize(self) -> CategoryBernSummary<T> {
        self.state_models.into_iter()
            .map(|(context, bm)| (context, bm.summarize()))
            .collect()
    }
}
