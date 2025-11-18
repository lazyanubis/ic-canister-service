#[allow(unused)]
use super::*;
#[allow(unused)]
pub use ic_canister_kit::identity::self_canister_id;
#[allow(unused)]
pub use ic_canister_kit::types::{CanisterId, PauseReason, UserId};
#[allow(unused)]
pub use std::collections::{HashMap, HashSet};
#[allow(unused)]
pub use std::fmt::Display;

pub(super) mod immutable {
    use super::*;

    pub trait GetImmutable {
        fn get(&self) -> &dyn Business;
    }

    #[allow(unused_variables)]
    pub trait Business:
        Pausable<PauseReason>
        + ParsePermission
        + Permissable<Permission>
        + Recordable<Record, RecordTopic, RecordSearch>
        + Schedulable
        + ScheduleTask
        + StableHeap
    {
        fn business_example_query(&self) -> String {
            ic_cdk::trap("Not supported operation by this version.")
        }
        fn business_example_count_query(&self) -> u64 {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_cell_query(&self) -> crate::stable::ExampleCell {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_vec_query(&self) -> Vec<crate::stable::ExampleVec> {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_map_query(&self) -> HashMap<u64, String> {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_log_query(&self) -> Vec<String> {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_priority_queue_query(&self) -> Vec<crate::stable::ExampleVec> {
            ic_cdk::trap("Not supported operation by this version.")
        }
    }

    // 业务实现
    impl Business for State {
        fn business_example_query(&self) -> String {
            self.get().business_example_query()
        }
        fn business_example_count_query(&self) -> u64 {
            self.get().business_example_count_query()
        }

        fn business_example_cell_query(&self) -> ExampleCell {
            self.get().business_example_cell_query()
        }

        fn business_example_vec_query(&self) -> Vec<ExampleVec> {
            self.get().business_example_vec_query()
        }

        fn business_example_map_query(&self) -> HashMap<u64, String> {
            self.get().business_example_map_query()
        }

        fn business_example_log_query(&self) -> Vec<String> {
            self.get().business_example_log_query()
        }

        fn business_example_priority_queue_query(&self) -> Vec<ExampleVec> {
            self.get().business_example_priority_queue_query()
        }
    }
}
pub use immutable::Business;

pub mod mutable {
    use super::*;

    pub trait GetMutable {
        fn get_mut(&mut self) -> &mut dyn MutableBusiness;
    }

    #[allow(clippy::panic)] // ? 允许回滚
    #[allow(clippy::unwrap_used)] // ? 允许回滚
    #[allow(clippy::expect_used)] // ? 允许回滚
    #[allow(unused_variables)]
    pub trait MutableBusiness: Business {
        fn business_example_update(&mut self, test: String) {
            ic_cdk::trap("Not supported operation by this version.")
        }
        fn business_example_count_update(&mut self, value: u64) {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_cell_update(&mut self, test: String) {
            ic_cdk::trap("Not supported operation by this version.")
        }
        fn business_example_cell_update_panic_in_business(&mut self, test: String) {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_vec_push(&mut self, test: u64) {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_vec_pop(&mut self) -> Option<crate::stable::ExampleVec> {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_map_update(&mut self, key: u64, value: Option<String>) -> Option<String> {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_log_update(&mut self, item: String) -> u64 {
            ic_cdk::trap("Not supported operation by this version.")
        }

        fn business_example_priority_queue_push(&mut self, item: u64) {
            ic_cdk::trap("Not supported operation by this version.")
        }
        fn business_example_priority_queue_pop(&mut self) -> Option<crate::stable::ExampleVec> {
            ic_cdk::trap("Not supported operation by this version.")
        }
    }

    // 业务实现
    #[allow(clippy::panic)] // ? 允许回滚
    #[allow(clippy::unwrap_used)] // ? 允许回滚
    #[allow(clippy::expect_used)] // ? 允许回滚
    impl MutableBusiness for State {
        fn business_example_update(&mut self, test: String) {
            self.get_mut().business_example_update(test)
        }
        fn business_example_count_update(&mut self, value: u64) {
            self.get_mut().business_example_count_update(value)
        }

        fn business_example_cell_update(&mut self, test: String) {
            self.get_mut().business_example_cell_update(test)
        }
        fn business_example_cell_update_panic_in_business(&mut self, test: String) {
            self.get_mut().business_example_cell_update_panic_in_business(test)
        }

        fn business_example_vec_push(&mut self, test: u64) {
            self.get_mut().business_example_vec_push(test)
        }
        fn business_example_vec_pop(&mut self) -> Option<ExampleVec> {
            self.get_mut().business_example_vec_pop()
        }

        fn business_example_map_update(&mut self, key: u64, value: Option<String>) -> Option<String> {
            self.get_mut().business_example_map_update(key, value)
        }

        fn business_example_log_update(&mut self, item: String) -> u64 {
            self.get_mut().business_example_log_update(item)
        }

        fn business_example_priority_queue_push(&mut self, item: u64) {
            self.get_mut().business_example_priority_queue_push(item)
        }
        fn business_example_priority_queue_pop(&mut self) -> Option<ExampleVec> {
            self.get_mut().business_example_priority_queue_pop()
        }
    }
}
pub use mutable::MutableBusiness;
