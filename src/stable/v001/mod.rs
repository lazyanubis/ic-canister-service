use std::collections::{HashMap, HashSet};

use ic_canister_kit::{functions::permission::basic::supers_updated, identity::caller, types::*};

pub mod types;

mod upgrade;

mod permission;

mod schedule;

mod business;

use types::*;

// 初始化
// ! 第一次部署会执行
impl Initial<Option<InitArg>> for InnerState {
    fn init(&mut self, arg: Option<InitArg>) {
        let mut arg = arg.unwrap_or_default(); // ! 就算是 None，也要执行一次
        arg.schedule = ic_canister_kit::common::trap(super::validate_schedule(arg.schedule));

        // 超级管理员初始化
        let supers = arg.supers.clone().unwrap_or_else(|| {
            vec![caller()] // 默认调用者为超级管理员
        });

        let permissions = get_all_permissions(|n| self.parse_permission(n));
        let updated = supers_updated(&supers, &permissions);

        // 刷新权限
        self.permission_reset(permissions);
        // 超级管理员赋予所有权限
        assert!(self.permission_update(updated).is_ok()); // 插入权限

        // 定时任务
        self.schedule_replace(arg.schedule);

        // 业务数据
        self.do_init(arg);
    }
}

// 升级
// ! 升级时执行
impl Upgrade<Option<UpgradeArg>> for InnerState {
    fn upgrade(&mut self, arg: Option<UpgradeArg>) {
        let mut arg = match arg {
            Some(arg) => arg,
            None => return, // ! None 表示升级无需处理数据
        };
        arg.schedule = ic_canister_kit::common::trap(super::validate_schedule(arg.schedule));

        // 超级管理员初始化
        let supers = arg.supers.clone();

        let permissions = get_all_permissions(|n| self.parse_permission(n));
        let updated = supers.as_ref().map(|supers| supers_updated(supers, &permissions));

        // 刷新权限
        self.permission_reset(permissions);
        // 超级管理员赋予所有权限
        if let Some(updated) = updated {
            assert!(self.permission_update(updated).is_ok()); // 插入权限
        }

        // 定时任务
        self.schedule_replace(arg.schedule);

        // 业务数据
        self.do_upgrade(arg);
    }
}

impl Pausable<PauseReason> for InnerState {
    // 查询
    fn pause_query(&self) -> &Option<PauseReason> {
        self.canister_kit.pause.pause_query()
    }
    // 修改
    fn pause_replace(&mut self, reason: Option<PauseReason>) {
        self.canister_kit.pause.pause_replace(reason)
    }
}

impl Permissable<Permission> for InnerState {
    // 查询
    fn permission_users(&self) -> HashSet<&UserId> {
        self.canister_kit.permissions.permission_users()
    }
    fn permission_roles(&self) -> HashSet<&String> {
        self.canister_kit.permissions.permission_roles()
    }
    fn permission_assigned(&self, user_id: &UserId) -> Option<&HashSet<Permission>> {
        self.canister_kit.permissions.permission_assigned(user_id)
    }
    fn permission_role_assigned(&self, role: &str) -> Option<&HashSet<Permission>> {
        self.canister_kit.permissions.permission_role_assigned(role)
    }
    fn permission_user_roles(&self, user_id: &UserId) -> Option<&HashSet<String>> {
        self.canister_kit.permissions.permission_user_roles(user_id)
    }
    fn permission_has(&self, user_id: &UserId, permission: &Permission) -> bool {
        self.canister_kit.permissions.permission_has(user_id, permission)
    }
    fn permission_owned(&self, user_id: &UserId) -> HashMap<&Permission, bool> {
        self.canister_kit.permissions.permission_owned(user_id)
    }

    // 修改
    fn permission_reset(&mut self, permissions: HashSet<Permission>) {
        self.canister_kit.permissions.permission_reset(permissions)
    }
    fn permission_update(
        &mut self,
        args: Vec<PermissionUpdatedArg<Permission>>,
    ) -> Result<(), PermissionUpdatedError<Permission>> {
        self.canister_kit.permissions.permission_update(args)
    }
}

impl Recordable<Record, RecordTopic, RecordSearch> for InnerState {
    // 查询
    fn record_find_all(&self) -> &[Record] {
        self.canister_kit.records.record_find_all()
    }
    // 修改
    fn record_push(&mut self, caller: CallerId, topic: RecordTopic, content: String) -> RecordId {
        self.canister_kit.records.record_push(caller, topic, content)
    }
    fn record_update(&mut self, record_id: RecordId, result: String) {
        self.canister_kit.records.record_update(record_id, result)
    }
    // 迁移
    fn record_migrate(&mut self, max: u32) -> MigratedRecords<Record> {
        self.canister_kit.records.record_migrate(max)
    }
}

impl Schedulable for InnerState {
    // 查询
    fn schedule_find(&self) -> Option<DurationNanos> {
        self.canister_kit.schedule.schedule_find()
    }
    // 修改
    fn schedule_replace(&mut self, schedule: Option<DurationNanos>) {
        self.canister_kit.schedule.schedule_replace(schedule)
    }
}

impl ScheduleTask for InnerState {}

impl StableHeap for InnerState {
    fn heap_to_bytes(&self) -> Vec<u8> {
        let bytes = ic_canister_kit::functions::stable::to_bytes(self);
        ic_canister_kit::common::trap(bytes)
    }

    fn heap_from_bytes(&mut self, bytes: &[u8]) {
        let state = ic_canister_kit::functions::stable::from_bytes(bytes);
        *self = ic_canister_kit::common::trap(state);
    }
}
