pub use ic_canister_kit::types::*;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub use super::super::{Business, MutableBusiness, ParsePermission, ScheduleTask};

#[allow(unused)]
pub use super::super::business::*;
#[allow(unused)]
pub use super::business::*;
#[allow(unused)]
pub use super::permission::*;
#[allow(unused)]
pub use super::schedule::schedule_task;

mod _init;
pub use _init::*;
mod _upgrade;
pub use _upgrade::*;
mod _topic;
pub use _topic::*;
mod _canister_kit;
pub use _canister_kit::*;

// 业务类型
mod example;
pub use example::*;
mod stable;
use stable::*;

// 能序列化的和不能序列化的放在一起
// 其中不能序列化的采用如下注解
// #[serde(skip)] 默认初始化方式
// #[serde(skip, default="init_xxx_data")] 指定初始化方式
// ! 如果使用 ic-stable-structures 提供的稳定内存，不能变更 memory_id 的使用类型，否则会出现各个版本不兼容，数据会被清空
#[derive(Serialize, Deserialize)]
pub struct InnerState {
    pub canister_kit: CanisterKit, // 框架需要的数据 // ? 堆内存 序列化

    // 业务数据
    pub example_data: String, // 样例数据 // ? 堆内存 序列化
    pub example_count: u64,   // 样例数据 // ? 堆内存 序列化

    #[serde(skip, default = "init_example_cell_data")]
    pub example_cell: StableCell<ExampleCell>, // 样例数据 // ? 稳定内存
    #[serde(skip, default = "init_example_vec_data")]
    pub example_vec: StableVec<ExampleVec>, // 样例数据 // ? 稳定内存
    #[serde(skip, default = "init_example_map_data")]
    pub example_map: StableBTreeMap<u64, String>, // 样例数据 // ? 稳定内存
    #[serde(skip, default = "init_example_log_data")]
    pub example_log: StableLog<String>, // 样例数据 // ? 稳定内存
    #[serde(skip, default = "init_example_priority_queue_data")]
    pub example_priority_queue: StablePriorityQueue<ExampleVec>, // 样例数据 // ? 稳定内存
}

impl Default for InnerState {
    fn default() -> Self {
        ic_cdk::println!("InnerState::default()");
        Self {
            canister_kit: Default::default(),

            // 业务数据
            example_data: Default::default(),
            example_count: Default::default(),

            example_cell: init_example_cell_data(),
            example_vec: init_example_vec_data(),
            example_map: init_example_map_data(),
            example_log: init_example_log_data(),
            example_priority_queue: init_example_priority_queue_data(),
        }
    }
}

impl InnerState {
    pub fn do_init(&mut self, _arg: InitArg) {
        // maybe do something
    }

    pub fn do_upgrade(&mut self, _arg: UpgradeArg) {
        // maybe do something
    }
}
