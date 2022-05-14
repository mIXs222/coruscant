use std::collections::BTreeMap;
use chashmap::CHashMap;
use std::hash::Hash;


type NormalSummary = (f64, f64);
type CategoryNormalSummary<T> = BTreeMap<T, NormalSummary>;
pub type ManyCategoryNormalSummary<T> = BTreeMap<T, CategoryNormalSummary<T>>;


/* Bernoulli */
#[derive(Clone, Debug)]
struct NormalEstimator {
    sum: f64,
    square_sum: f64,
    count: f64,
}

impl Default for NormalEstimator {
    fn default() -> Self {
        NormalEstimator {
            sum: 0.0,
            square_sum: 0.0,
            count: 0.0,
        }
    }
}

impl NormalEstimator {
    fn observe(&mut self, number: f64) {
        // log::error!("{}", number);
        self.count += 1.0;
        self.sum += number;
        self.square_sum += number * number;
    }

    fn summarize(self) -> NormalSummary {
        (
            self.sum / self.count,  // mean
            (self.square_sum / self.count - (self.sum / self.count).powi(2)).sqrt(),  // stddev
        )
    }
}


/* Multiple Bernoulli  */
#[derive(Default, Clone, Debug)]
pub struct CategoryNormalEstimator<T> {
    state_models: CHashMap<T, NormalEstimator>
}

impl<T> CategoryNormalEstimator<T> 
where T: PartialEq + Eq + Hash + Ord + Default + std::fmt::Debug
{
    pub fn observe(&self, state: T, number: f64) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe(number)
        } else {
            let mut bm = NormalEstimator::default();
            bm.observe(number);
            self.state_models.insert_new(state, bm)
        }
    }

    pub fn summarize(self) -> CategoryNormalSummary<T> {
        self.state_models.into_iter()
            .map(|(context, bm)| (context, bm.summarize()))
            .collect()
    }
}


/* Many Multiple Bernoulli */
#[derive(Default, Clone, Debug)]
pub struct ManyCategoryNormalEstimator<T> {
    state_models: CHashMap<T, CategoryNormalEstimator<T>>
}

impl<T> ManyCategoryNormalEstimator<T> 
where T: PartialEq + Eq + Hash + Ord + Default + std::fmt::Debug
{
    pub fn observe(&self, state: T, substate: T, number: f64) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe(substate, number)
        } else {
            let cbm = CategoryNormalEstimator::default();
            cbm.observe(substate, number);
            self.state_models.insert_new(state, cbm)
        }
    }

    pub fn summarize(self) -> ManyCategoryNormalSummary<T> {
        self.state_models.into_iter()
            .map(|(context, cbm)| (context, cbm.summarize()))
            .collect()
    }
}
