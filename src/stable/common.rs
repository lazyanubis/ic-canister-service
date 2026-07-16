use std::collections::{HashMap, HashSet};

use ic_canister_kit::functions::schedule::try_schedule_task_guard;
pub use ic_canister_kit::functions::schedule::validate_schedule;
use ic_canister_kit::types::*;

use super::{ParsePermission, ParsePermissionError, business::immutable::GetImmutable, business::mutable::GetMutable};
use super::{State, schedule_task, with_state};

/// 检查是否拥有某权限
pub fn check_permission(
    permission: &str,
    running: bool, // 是否要求必须处于正常运行状态
) -> Result<(), String> {
    let caller = ic_canister_kit::identity::caller();
    with_state(|s| {
        let _permission = s.parse_permission(permission).map_err(|e| e.to_string())?;
        if s.permission_has(&caller, &_permission) {
            if running {
                s.pause_must_be_running()?;
            }
            return Ok(());
        }
        Err(format!("Permission '{permission}' is required"))
    })
}

impl Pausable<PauseReason> for State {
    // 查询
    fn pause_query(&self) -> &Option<PauseReason> {
        self.get().pause_query()
    }
    // 修改
    fn pause_replace(&mut self, reason: Option<PauseReason>) {
        self.get_mut().pause_replace(reason)
    }
}

impl ParsePermission for State {
    fn parse_permission<'a>(&self, name: &'a str) -> Result<Permission, ParsePermissionError<'a>> {
        self.get().parse_permission(name)
    }
}

impl Permissable<Permission> for State {
    // 查询
    fn permission_users(&self) -> HashSet<&UserId> {
        self.get().permission_users()
    }
    fn permission_roles(&self) -> HashSet<&String> {
        self.get().permission_roles()
    }
    fn permission_assigned(&self, user_id: &UserId) -> Option<&HashSet<Permission>> {
        self.get().permission_assigned(user_id)
    }
    fn permission_role_assigned(&self, role: &str) -> Option<&HashSet<Permission>> {
        self.get().permission_role_assigned(role)
    }
    fn permission_user_roles(&self, user_id: &UserId) -> Option<&HashSet<String>> {
        self.get().permission_user_roles(user_id)
    }
    fn permission_has(&self, user_id: &UserId, permission: &Permission) -> bool {
        self.get().permission_has(user_id, permission)
    }
    fn permission_owned(&self, user_id: &UserId) -> HashMap<&Permission, bool> {
        self.get().permission_owned(user_id)
    }

    // 修改
    fn permission_reset(&mut self, permissions: HashSet<Permission>) {
        self.get_mut().permission_reset(permissions)
    }
    fn permission_update(
        &mut self,
        args: Vec<PermissionUpdatedArg<Permission>>,
    ) -> Result<(), PermissionUpdatedError<Permission>> {
        self.get_mut().permission_update(args)
    }
}

impl Recordable<Record, RecordTopic, RecordSearch> for State {
    // 查询
    fn record_find_all(&self) -> &[Record] {
        self.get().record_find_all()
    }

    // 修改
    fn record_push(&mut self, caller: CallerId, topic: RecordTopic, content: String) -> RecordId {
        self.get_mut().record_push(caller, topic, content)
    }
    fn record_update(&mut self, record_id: RecordId, result: String) {
        self.get_mut().record_update(record_id, result)
    }

    // 删除
    fn record_delete(&mut self, ids: &HashSet<RecordId>) -> u64 {
        self.get_mut().record_delete(ids)
    }
}

impl Schedulable for State {
    // 查询
    fn schedule_find(&self) -> Option<DurationNanos> {
        self.get().schedule_find()
    }
    // 修改
    fn schedule_replace(&mut self, schedule: Option<DurationNanos>) {
        self.get_mut().schedule_replace(schedule)
    }
}

/// 确保当前没有正在执行的定时任务。
///
/// 通过尝试取得同一个防重入 guard 判断任务是否空闲；函数返回时 guard
/// 会立即释放。调用方必须在检查后、修改状态前保持同步执行，不能插入 `await`。
pub fn schedule_task_must_be_idle() -> Result<(), String> {
    let _guard = try_schedule_task_guard()?;
    Ok(())
}

/// 执行定时任务，自动任务与手动任务共享同一个防重入锁
pub async fn run_schedule_task(record_by: Option<CallerId>) -> Result<(), String> {
    with_state(|s| s.pause_must_be_running())?;
    let _guard = try_schedule_task_guard()?;
    schedule_task(record_by).await;
    Ok(())
}

#[allow(unused)]
async fn static_schedule_task() {
    if with_state(|s| s.pause_is_paused()) {
        return; // 维护中不允许执行任务
    }

    if let Err(error) = run_schedule_task(None).await {
        ic_cdk::println!("Skip schedule task: {error}");
    }
}

pub trait ScheduleTask: Schedulable {
    fn schedule_stop(&self) {
        ic_canister_kit::functions::schedule::stop_schedule();
    }
    fn schedule_reload(&mut self) {
        let schedule = self.schedule_find();
        ic_canister_kit::functions::schedule::start_schedule(&schedule, static_schedule_task);
    }
}

impl ScheduleTask for State {}

impl StableHeap for State {
    fn heap_to_bytes(&self) -> Vec<u8> {
        self.get().heap_to_bytes()
    }

    fn heap_from_bytes(&mut self, bytes: &[u8]) {
        self.get_mut().heap_from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{schedule_task_must_be_idle, try_schedule_task_guard};

    #[test]
    fn schedule_task_idle_check_uses_the_execution_guard() {
        assert!(schedule_task_must_be_idle().is_ok());

        let guard = try_schedule_task_guard();
        assert!(guard.is_ok());
        if let Ok(guard) = guard {
            assert_eq!(
                schedule_task_must_be_idle(),
                Err("Schedule task is already running.".to_string())
            );
            drop(guard);
        }
        assert!(schedule_task_must_be_idle().is_ok());
    }
}
