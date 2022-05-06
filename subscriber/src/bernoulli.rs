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
    fn observe(&mut self) {
        self.total_count += 1
    }

    fn observe_event(&mut self) {
        self.event_count += 1
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
    pub fn observe(&self, state: T) {
        self.state_models.upsert(
            state,
            BernEstimator::default,
            |bm| bm.observe(),
        );
    }

    pub fn observe_event(&self, state: T) {
        if let Some(mut sm) = self.state_models.get_mut(&state) {
            sm.observe_event();
        } else {
            log::warn!("Observed event on unseen state {:?}", state);
        }
    }

    pub fn summarize(self) -> CategoryBernSummary<T> {
        self.state_models.into_iter()
            .map(|(context, bm)| (context, bm.summarize()))
            .collect()
    }
}
