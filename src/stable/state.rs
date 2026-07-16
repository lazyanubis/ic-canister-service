use std::cell::RefCell;

use ic_canister_kit::identity::caller;
use ic_canister_kit::types::*;

use super::{InitArgs, RecordTopics, ScheduleTask, UpgradeArgs, schedule_task_must_be_idle, validate_schedule};
use super::{State, State::*};

// 默认值
impl Default for State {
    fn default() -> Self {
        // ? 初始化和升级会先进行迁移, 因此最初的版本无关紧要
        V0(Box::default())
    }
}

// ================= 需要持久化的数据 ================

thread_local! {
    static STATE: RefCell<State> = RefCell::default(); // 存储系统数据
}

// ==================== 初始化方法 ====================

#[ic_cdk::init]
fn initial(args: Option<InitArgs>) {
    with_mut_state_without_record(|s| {
        let record_id = s.record_push(
            caller(),
            RecordTopics::Initial.topic(),
            format!("Initial by {}", caller().to_text()),
        );
        s.upgrade(None); // upgrade to latest version
        s.init(args); // ! 初始化最新版本
        s.schedule_reload(); // * 重置定时任务
        s.record_update(record_id, format!("Version: {}", s.version()));
    })
}

// ==================== 升级时的恢复逻辑 ====================

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<UpgradeArgs>) {
    STATE.with(|state| {
        let stable = ic_cdk::storage::stable_restore();
        let stable: (RecordId, u32, Vec<u8>) = ic_canister_kit::common::trap(stable); // ! 可能读档失败
        let (record_id, version, bytes) = stable;

        // 利用版本号恢复升级前的版本
        let mut last_state = State::from_version(version);
        last_state.heap_from_bytes(&bytes); // 恢复数据
        *state.borrow_mut() = last_state;

        state.borrow_mut().upgrade(args); // ! 恢复后要进行升级到最新版本

        // 无论本次是否传入升级参数，都要校验恢复出的定时任务配置。
        let schedule = state.borrow().schedule_find();
        let schedule = ic_canister_kit::common::trap(validate_schedule(schedule));
        state.borrow_mut().schedule_replace(schedule);
        if state.borrow().pause_is_paused() {
            state.borrow().schedule_stop();
        } else {
            state.borrow_mut().schedule_reload(); // * 重置定时任务
        }

        let version = state.borrow().version(); // 先不可变借用取出版本号
        state
            .borrow_mut()
            .record_update(record_id, format!("Next version: {version}"));
    });
}

// ==================== 升级时的保存逻辑，下次升级执行 ====================

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    let caller = caller();
    STATE.with(|state| {
        use ic_canister_kit::common::trap;
        trap(state.borrow().pause_must_be_paused()); // ! 必须是维护状态, 才可以升级
        trap(schedule_task_must_be_idle()); // ! 运行中的定时任务必须先完成
        state.borrow_mut().schedule_stop(); // * 停止定时任务

        let record_id = state.borrow_mut().record_push(
            caller,
            RecordTopics::Upgrade.topic(),
            format!("Upgrade by {}", caller.to_text()),
        );
        let version = state.borrow().version();
        let bytes = state.borrow().heap_to_bytes();

        let stable: (RecordId, u32, Vec<u8>) = (record_id, version, bytes);
        trap(ic_cdk::storage::stable_save(stable)); // ! 可能存档失败
    });
}

// ==================== 工具方法 ====================

/// 外界需要系统状态时
#[allow(unused)]
pub fn with_state<F, R>(callback: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|state| {
        let state = state.borrow(); // 取得不可变对象
        callback(&state)
    })
}

/// 需要可变系统状态时
#[allow(unused)]
pub fn with_mut_state_without_record<F, R>(callback: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        callback(&mut state)
    })
}

/// 需要可变系统状态时 // ! 变更操作一定要记录
#[allow(unused)]
pub fn with_mut_state<F, R>(callback: F, caller: CallerId, topic: RecordTopic, content: String) -> R
where
    F: FnOnce(&mut State, &mut Option<String>) -> R,
    R: serde::Serialize,
{
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        let record_id = state.record_push(caller, topic, content);
        let mut record_result = None;
        let output = callback(&mut state, &mut record_result);
        state.record_update(
            record_id,
            record_result.unwrap_or_else(|| match serde_json::to_string(&output) {
                Ok(s) => s,
                Err(e) => format!("Serialize failed: {e}"),
            }),
        );
        output
    })
}

/// 新增记录
#[allow(unused)]
pub fn with_record_push(topic: RecordTopic, content: String) -> RecordId {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        state.record_push(caller, topic, content)
    })
}
/// 更新记录
#[allow(unused)]
pub fn with_record_update(record_id: RecordId, result: String) {
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        state.record_update(record_id, result)
    })
}
