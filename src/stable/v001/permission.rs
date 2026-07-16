use std::collections::HashSet;

use ic_canister_kit::types::Permission;

use crate::stable::ParsePermissionError;

use super::super::check_permission;

use super::types::{InnerState, ParsePermission};

// 权限常量
// 通用权限
pub use super::super::v000::types::{
    ACTION_PAUSE_QUERY, ACTION_PAUSE_REPLACE, ACTION_PERMISSION_FIND, ACTION_PERMISSION_QUERY,
    ACTION_PERMISSION_UPDATE, ACTION_RECORD_DELETE, ACTION_RECORD_FIND, ACTION_SCHEDULE_FIND, ACTION_SCHEDULE_REPLACE,
    ACTION_SCHEDULE_TRIGGER,
};

// 业务权限
pub const ACTION_BUSINESS_EXAMPLE_QUERY: &str = "BusinessExampleQuery"; // 业务查询权限
pub const ACTION_BUSINESS_EXAMPLE_SET: &str = "BusinessExampleSet"; // 业务更新权限

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
    ACTION_BUSINESS_EXAMPLE_QUERY,
    ACTION_BUSINESS_EXAMPLE_SET,
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
            ACTION_BUSINESS_EXAMPLE_QUERY => Permission::by_forbid(name),
            ACTION_BUSINESS_EXAMPLE_SET => Permission::by_permit(name),
            // 其他错误
            _ => return Err(ParsePermissionError(name)),
        })
    }
}

// 通用权限
#[allow(unused)]
pub use super::super::v000::types::{
    has_pause_query, has_pause_replace, has_permission_find, has_permission_query, has_permission_update,
    has_record_delete, has_record_find, has_schedule_find, has_schedule_replace, has_schedule_trigger,
};

// 业务权限

#[allow(unused)]
pub fn has_business_example_query() -> Result<(), String> {
    check_permission(ACTION_BUSINESS_EXAMPLE_QUERY, false)
}

#[allow(unused)]
pub fn has_business_example_set() -> Result<(), String> {
    check_permission(ACTION_BUSINESS_EXAMPLE_SET, true)
}
