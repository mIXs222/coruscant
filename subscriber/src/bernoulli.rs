use std::collections::BTreeMap;
use chashmap::CHashMap;
use std::hash::Hash;


type BernSummary = (usize, usize);
type CategoryBernSummary<T> = BTreeMap<T, BernSummary>;
pub type ManyCategoryBernSummary<T> = BTreeMap<T, CategoryBernSummary<T>>;


/* Bernoulli */
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
        (self.event_count, self.total_count)
    }
}


/* Multiple Bernoulli  */
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


/* Many Multiple Bernoulli */
#[derive(Default, Clone, Debug)]
pub struct ManyCategoryBernEstimator<T> {
    state_models: CHashMap<T, CategoryBernEstimator<T>>
}

impl<T> ManyCategoryBernEstimator<T> 
where T: PartialEq + Eq + Hash + Ord + Default + std::fmt::Debug
{
    pub fn observe_absent(&self, state: T, substate: T) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe_absent(substate)
        } else {
            let cbm = CategoryBernEstimator::default();
            cbm.observe_absent(substate);
            self.state_models.insert_new(state, cbm)
        }
    }

    pub fn observe_present(&self, state: T, substate: T) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe_present(substate)
        } else {
            let cbm = CategoryBernEstimator::default();
            cbm.observe_present(substate);
            self.state_models.insert_new(state, cbm)
        }
    }

    pub fn summarize(self) -> ManyCategoryBernSummary<T> {
        self.state_models.into_iter()
            .map(|(context, cbm)| (context, cbm.summarize()))
            .collect()
    }
}
