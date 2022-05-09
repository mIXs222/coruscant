use chashmap::CHashMap;
use std::collections::BTreeMap;
use std::hash::Hash;


/*
 * Model hierarchy
 *
 *  ContextMarkovEstimator: parent_span --> MarkovEstimator
 *  MarkovEstimator: current_span --> StateEstimator
 *  StateEstimator: next_span --> probability
 */

type StateSummary<T> = BTreeMap<T, f64>;
type MarkovSummary<T> = BTreeMap<T, StateSummary<T>>;
pub type ContextMarkovSummary<T> = BTreeMap<T, MarkovSummary<T>>;


/* Single-state transition model */
#[derive(Default, Clone, Debug)]
struct StateEstimator<T> {
    state_count: usize,
    transition_count: CHashMap<T, usize>
}

// impl<T: PartialEq + Eq + Hash + Default> StateEstimator<T> {
impl<T: PartialEq + Eq + Hash + Ord + Default> StateEstimator<T> {
    fn observe(&mut self, next_state: T) {
        self.state_count += 1;
        self.transition_count.upsert(
            next_state,
            || 1,
            |tc|
            *tc += 1,
        );
    }

    fn summarize(self) -> StateSummary<T> {
        self.transition_count.into_iter()
            .map(|(next_state, count)|
                (next_state, count as f64 / self.state_count as f64))
            .collect()
    }
}


/* Transition model between multiple states */
#[derive(Default, Clone, Debug)]
struct MarkovEstimator<T> {
    state_models: CHashMap<T, StateEstimator<T>>
}

// impl<T: PartialEq + Eq + Hash + Default> MarkovEstimator<T> {
impl<T: PartialEq + Eq + Hash + Ord + Default> MarkovEstimator<T> {
    fn observe(&self, state: T, next_state: T) {
        if self.state_models.contains_key(&state) {
            self.state_models.get_mut(&state).unwrap().observe(next_state)
        } else {
            let mut sm = StateEstimator::default();
            sm.observe(next_state);
            self.state_models.insert_new(state, sm)
        }
    }

    fn summarize(self) -> MarkovSummary<T> {
        self.state_models.into_iter()
            .map(|(context, sm)| (context, sm.summarize()))
            .collect()
    }
}


/* Transition model between multiple states */
#[derive(Default, Clone, Debug)]
pub struct ContextMarkovEstimator<T> {
    span_models: CHashMap<T, MarkovEstimator<T>>
}

// impl<T: PartialEq + Eq + Hash + Default> ContextMarkovEstimator<T> {
impl<T: PartialEq + Eq + Hash + Ord + Default> ContextMarkovEstimator<T> {
    pub fn observe(&self, context: T, state: T, next_state: T) {
        if self.span_models.contains_key(&context) {
            self.span_models.get_mut(&context).unwrap().observe(state, next_state)
        } else {
            let mm = MarkovEstimator::default();
            mm.observe(state, next_state);
            self.span_models.insert_new(context, mm)
        }
    }

    pub fn summarize(self) -> ContextMarkovSummary<T> {
        self.span_models.into_iter()
            .map(|(context, mm)| (context, mm.summarize()))
            .collect()
    }
}