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
    }

    // 业务实现
    impl Business for State {
        fn business_example_query(&self) -> String {
            self.get().business_example_query()
        }
        fn business_example_count_query(&self) -> u64 {
            self.get().business_example_count_query()
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
    }
}
pub use mutable::MutableBusiness;
