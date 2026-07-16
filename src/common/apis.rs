use ic_canister_kit::{common::option::display_option, identity::caller};

use crate::stable::*;
use crate::types::*;

// ================== 通用接口 ==================

#[ic_cdk::query]
pub fn wallet_balance() -> candid::Nat {
    ic_canister_kit::canister::cycles::wallet_balance()
}

#[ic_cdk::update]
pub fn wallet_receive() -> candid::Nat {
    let caller = caller();
    ic_canister_kit::canister::cycles::wallet_receive(|accepted| {
        let record_id = with_record_push(
            RecordTopics::CyclesCharge.topic(),
            format!("{} recharge {} cycles", caller.to_text(), accepted),
        );
        with_record_update(record_id, String::new());
    })
}

#[ic_cdk::update]
async fn canister_status() -> ic_canister_kit::types::CanisterStatusResult {
    use ic_canister_kit::{canister::status::canister_status, identity::self_canister_id};
    let response = canister_status(self_canister_id()).await;
    ic_canister_kit::common::trap(response)
}

#[ic_cdk::query]
async fn whoami() -> ic_canister_kit::types::UserId {
    ic_canister_kit::identity::caller()
}

// ================== 数据版本 ==================

// 当前数据库版本
#[ic_cdk::query]
fn version() -> u32 {
    with_state(|s| s.version())
}

// ================== 维护接口 ==================

// 查询维护状态
#[ic_cdk::query(guard = "has_pause_query")]
fn pause_query() -> bool {
    with_state(|s| s.pause_is_paused())
}

// 查询维护状态
#[ic_cdk::query(guard = "has_pause_query")]
fn pause_query_reason() -> Option<PauseReason> {
    with_state(|s| s.pause_query().clone())
}

// 设置维护状态
#[ic_cdk::update(guard = "has_pause_replace")]
fn pause_replace(reason: Option<String>) {
    let old = with_state(|s| s.pause_query().clone());

    if old.is_none() && reason.is_none() {
        return; // 未改变内容
    }

    let caller = caller();
    let pausing = reason.is_some();
    let arg_content = format!("{} -> {}", display_option(&old), display_option(&reason)); // * 记录参数内容

    with_mut_state(
        |s, _result| {
            s.pause_replace(reason.map(PauseReason::new));
            if pausing {
                s.schedule_stop();
            } else {
                s.schedule_reload();
            }
        },
        caller,
        RecordTopics::Pause.topic(),
        arg_content,
    )
}

// ================== 权限接口 ==================

// 所有权限
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_all() -> Vec<Permission> {
    use ic_canister_kit::common::trap;
    use ic_canister_kit::functions::permission::basic::parse_all_permissions;
    with_state(|s| {
        let permissions = parse_all_permissions(ACTIONS, |name| s.parse_permission(name));
        trap(permissions.map_err(|e| e.to_string()))
    })
}

// 查询自己权限
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_query() -> Vec<&'static str> {
    permission_find_by_user(ic_canister_kit::identity::caller())
}

// 查询他人权限
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_find_by_user(user_id: UserId) -> Vec<&'static str> {
    with_state(|s| {
        use ic_canister_kit::common::trap;
        use ic_canister_kit::functions::permission::basic::parse_all_permissions;
        let permissions = parse_all_permissions(ACTIONS, |name| s.parse_permission(name));
        trap(permissions)
            .iter()
            .zip(ACTIONS)
            .filter(|(permission, _)| s.permission_has(&user_id, permission))
            .map(|(_, p)| *p)
            .collect()
    })
}

// 查询自己指定权限
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_assigned_query() -> Option<HashSet<Permission>> {
    permission_assigned_by_user(ic_canister_kit::identity::caller())
}

// 查询他人指定权限
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_assigned_by_user(user_id: UserId) -> Option<HashSet<Permission>> {
    with_state(|s| s.permission_assigned(&user_id).cloned())
}

// 所有角色
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_roles_all() -> HashMap<String, HashSet<Permission>> {
    with_state(|s| {
        s.permission_roles()
            .into_iter()
            .map(|role| {
                (
                    role.to_owned(),
                    s.permission_role_assigned(role).cloned().unwrap_or_default(),
                )
            })
            .collect()
    })
}

// 查询自己角色
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_roles_query() -> Option<HashSet<String>> {
    permission_roles_by_user(ic_canister_kit::identity::caller())
}

// 查询他人角色
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_roles_by_user(user_id: UserId) -> Option<HashSet<String>> {
    with_state(|s| s.permission_user_roles(&user_id).cloned())
}

// 更新权限
#[ic_cdk::update(guard = "has_permission_update")]
fn permission_update(args: Vec<PermissionUpdatedArg<String>>) {
    let caller = caller();
    let arg_content = format!(
        "permission update: [{}]",
        args.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(", ")
    ); // * 记录参数内容

    with_mut_state(
        |s, _result| {
            use ic_canister_kit::common::trap;
            let args = args
                .into_iter()
                .map(|a| a.parse_permission(|n| s.parse_permission(n).map_err(|e| e.to_string())))
                .collect::<Result<Vec<_>, _>>();
            let args = trap(args);
            trap(s.permission_update(args))
        },
        caller,
        RecordTopics::Permission.topic(),
        arg_content,
    )
}

// ================== 记录接口 ==================

// 查询所有主题
#[ic_cdk::query(guard = "has_record_find")]
fn record_topics() -> Vec<String> {
    RecordTopics::topics()
}

// 查询所有 分页
#[ic_cdk::query(guard = "has_record_find")]
fn record_find_by_page(page: QueryPage, search: Option<RecordSearchArg>) -> PageData<Record> {
    use ic_canister_kit::common::trap;
    let search = search
        .map(|s| s.into(|t| RecordTopics::from(t).map(|t| t.topic())))
        .transpose();
    let search = trap(search);
    let result = with_state(|s| s.record_find_by_page(&page, 1000, &search).map(|p| p.into()));
    trap(result)
}

// 移动
#[ic_cdk::update(guard = "has_record_migrate")]
fn record_migrate(max: u32) -> MigratedRecords<Record> {
    if max == 0 {
        ic_cdk::trap("Record migration max must be greater than 0.");
    }

    let caller = caller();
    let arg_content = format!("wanna migrate {max} records"); // * 记录参数内容
    let record_count = with_state(|s| u32::try_from(s.record_find_all().len()).unwrap_or(u32::MAX));
    let max = max.min(record_count);

    with_mut_state(
        |s, record_result| {
            let result = s.record_migrate(max);
            *record_result = Some(format!(
                "retention evicted: {} next id: {} record size: {} ",
                result.retention_evicted_count,
                result.next_id,
                result.records.len(),
            ));
            result
        },
        caller,
        RecordTopics::Record.topic(),
        arg_content,
    )
}

// ================== 定时任务 ==================

// 查询定时状态
#[ic_cdk::query(guard = "has_schedule_find")]
fn schedule_find() -> Option<u64> {
    let schedule = with_state(|s| {
        s.schedule_find()
            .map(|schedule| {
                u64::try_from(schedule.into_inner())
                    .map_err(|_| "Stored schedule interval exceeds the u64 nanosecond range.")
            })
            .transpose()
    });
    ic_canister_kit::common::trap(schedule)
}

// 设置定时状态
#[ic_cdk::update(guard = "has_schedule_replace")]
fn schedule_replace(schedule: Option<u64>) {
    let schedule = schedule.map(|schedule| (schedule as u128).into());
    let schedule = ic_canister_kit::common::trap(validate_schedule(schedule));
    let old = with_state(|s| s.schedule_find());

    let caller = caller();
    let arg_content = format!("{} -> {}", display_option(&old), display_option(&schedule)); // * 记录参数内容

    with_mut_state(
        |s, _result| {
            s.schedule_replace(schedule);
            if s.pause_is_paused() {
                s.schedule_stop();
            } else {
                s.schedule_reload(); // * 重置定时任务
            }
        },
        caller,
        RecordTopics::Schedule.topic(),
        arg_content,
    )
}

// 手动触发定时任务
#[ic_cdk::update(guard = "has_schedule_trigger")]
async fn schedule_trigger() {
    let caller = caller();
    ic_canister_kit::common::trap(run_schedule_task(Some(caller)).await)
}
