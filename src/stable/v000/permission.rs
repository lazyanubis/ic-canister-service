use std::collections::HashSet;

use ic_canister_kit::types::Permission;

use crate::stable::ParsePermissionError;

use super::super::check_permission;

use super::types::{InnerState, ParsePermission};

// 权限常量
// 通用权限
pub const ACTION_PAUSE_QUERY: &str = "PauseQuery"; // 查询维护状态
pub const ACTION_PAUSE_REPLACE: &str = "PauseReplace"; // 设置维护状态
pub const ACTION_PERMISSION_QUERY: &str = "PermissionQuery"; // 查询个人权限信息
pub const ACTION_PERMISSION_FIND: &str = "PermissionFind"; // 查询他人权限
pub const ACTION_PERMISSION_UPDATE: &str = "PermissionUpdate"; // 设置权限
pub const ACTION_RECORD_FIND: &str = "RecordFind"; // 查询记录
pub const ACTION_RECORD_DELETE: &str = "RecordDelete"; // 删除记录
pub const ACTION_SCHEDULE_FIND: &str = "ScheduleFind"; // 查询定时状态
pub const ACTION_SCHEDULE_REPLACE: &str = "ScheduleReplace"; // 设置定时频率
pub const ACTION_SCHEDULE_TRIGGER: &str = "ScheduleTrigger"; // 触发定时任务

// 业务权限

// 所有权限列表
#[allow(unused)]
pub const ACTIONS: &[&str] = &[
    // 通用权限
    ACTION_PAUSE_QUERY,
    ACTION_PAUSE_REPLACE,
    ACTION_PERMISSION_QUERY,
    ACTION_PERMISSION_FIND,
    ACTION_PERMISSION_UPDATE,
    ACTION_RECORD_FIND,
    ACTION_RECORD_DELETE,
    ACTION_SCHEDULE_FIND,
    ACTION_SCHEDULE_REPLACE,
    ACTION_SCHEDULE_TRIGGER,
    // 业务权限
];

pub(super) fn get_all_permissions<'a, F>(parse: F) -> HashSet<Permission>
where
    F: Fn(&'a str) -> Result<Permission, ParsePermissionError<'a>>,
{
    use ic_canister_kit::functions::permission::basic::parse_all_permissions;
    let permissions = parse_all_permissions(ACTIONS, parse);
    let permissions = ic_canister_kit::common::trap(permissions);
    permissions.into_iter().collect()
}

// 权限默认状态
impl ParsePermission for InnerState {
    fn parse_permission<'a>(&self, name: &'a str) -> Result<Permission, ParsePermissionError<'a>> {
        Ok(match name {
            // 通用权限
            ACTION_PAUSE_QUERY => Permission::by_forbid(name),
            ACTION_PAUSE_REPLACE => Permission::by_permit(name),
            ACTION_PERMISSION_QUERY => Permission::by_forbid(name),
            ACTION_PERMISSION_FIND => Permission::by_permit(name),
            ACTION_PERMISSION_UPDATE => Permission::by_permit(name),
            ACTION_RECORD_FIND => Permission::by_permit(name),
            ACTION_RECORD_DELETE => Permission::by_permit(name),
            ACTION_SCHEDULE_FIND => Permission::by_permit(name),
            ACTION_SCHEDULE_REPLACE => Permission::by_permit(name),
            ACTION_SCHEDULE_TRIGGER => Permission::by_permit(name),
            // 业务权限

            // 其他错误
            _ => return Err(ParsePermissionError(name)),
        })
    }
}

// 通用权限

pub fn has_pause_query() -> Result<(), String> {
    check_permission(ACTION_PAUSE_QUERY, false)
}

pub fn has_pause_replace() -> Result<(), String> {
    check_permission(ACTION_PAUSE_REPLACE, false)
}

pub fn has_permission_query() -> Result<(), String> {
    check_permission(ACTION_PERMISSION_QUERY, false)
}

pub fn has_permission_find() -> Result<(), String> {
    check_permission(ACTION_PERMISSION_FIND, false)
}

pub fn has_permission_update() -> Result<(), String> {
    check_permission(ACTION_PERMISSION_UPDATE, false)
}

pub fn has_record_find() -> Result<(), String> {
    check_permission(ACTION_RECORD_FIND, false)
}

pub fn has_record_delete() -> Result<(), String> {
    check_permission(ACTION_RECORD_DELETE, false)
}

pub fn has_schedule_find() -> Result<(), String> {
    check_permission(ACTION_SCHEDULE_FIND, false)
}

pub fn has_schedule_replace() -> Result<(), String> {
    check_permission(ACTION_SCHEDULE_REPLACE, false)
}

pub fn has_schedule_trigger() -> Result<(), String> {
    check_permission(ACTION_SCHEDULE_TRIGGER, true)
}

// 业务权限
