use super::super::business::*;
use super::types::*;

impl Business for InnerState {
    fn business_example_query(&self) -> String {
        self.example_data.clone()
    }

    fn business_example_count_query(&self) -> u64 {
        self.example_count
    }

    fn business_example_cell_query(&self) -> ExampleCell {
        self.example_cell.get().clone()
    }

    fn business_example_vec_query(&self) -> Vec<ExampleVec> {
        self.example_vec.iter().collect()
    }

    fn business_example_map_query(&self) -> HashMap<u64, String> {
        self.example_map
            .iter()
            .map(|item| (*item.key(), item.value()))
            .collect()
    }

    fn business_example_log_query(&self) -> Vec<String> {
        self.example_log.iter().collect()
    }

    fn business_example_priority_queue_query(&self) -> Vec<ExampleVec> {
        self.example_priority_queue.iter().collect()
    }
}

#[allow(clippy::panic)] // ? 允许回滚
#[allow(clippy::unwrap_used)] // ? 允许回滚
#[allow(clippy::expect_used)] // ? 允许回滚
impl MutableBusiness for InnerState {
    fn business_example_update(&mut self, test: String) {
        self.example_data = test
    }

    fn business_example_count_update(&mut self, value: u64) {
        self.example_count = value
    }

    fn business_example_cell_update(&mut self, test: String) {
        let mut cell = self.example_cell.get().to_owned();
        cell.cell_data = test;
        self.example_cell.set(cell);
    }
    fn business_example_cell_update_panic_in_business(&mut self, _test: String) {
        ic_cdk::trap("panic in business");
    }

    fn business_example_vec_push(&mut self, test: u64) {
        self.example_vec.push(&ExampleVec { vec_data: test })
    }
    fn business_example_vec_pop(&mut self) -> Option<ExampleVec> {
        self.example_vec.pop()
    }

    fn business_example_map_update(&mut self, key: u64, value: Option<String>) -> Option<String> {
        if let Some(value) = value {
            self.example_map.insert(key, value)
        } else {
            self.example_map.remove(&key)
        }
    }

    fn business_example_log_update(&mut self, item: String) -> u64 {
        use ic_canister_kit::common::trap_debug;
        trap_debug(self.example_log.append(&item))
    }

    fn business_example_priority_queue_push(&mut self, item: u64) {
        self.example_priority_queue.push(&ExampleVec { vec_data: item })
    }
    fn business_example_priority_queue_pop(&mut self) -> Option<ExampleVec> {
        self.example_priority_queue.pop()
    }
}
