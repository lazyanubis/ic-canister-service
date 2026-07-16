//! https://github.com/dfinity/pocketic
use candid::encode_one;
use pocket_ic::PocketIc;

mod util;

mod service;

const INIT_CYCLES: u128 = 2 * 10_u128.pow(12); // 2T cycles

const WASM_MODULE_NEXT: &[u8] = include_bytes!("../sources/source_opt.wasm.gz");

#[ignore]
#[test]
#[rustfmt::skip]
fn test_common_apis() {
    let pic = PocketIc::new();

    let (default_identity, alice_identity, bob_identity, carol_identity, anonymous_identity) = util::get_identity();

    let canister_id = pic.create_canister_with_settings(Some(default_identity), None);
    pic.add_cycles(canister_id, INIT_CYCLES);

    pic.install_canister(canister_id, WASM_MODULE_NEXT.to_vec(), encode_one(None::<()>).unwrap(), Some(default_identity));

    use service::*;
    use Permission::*;

    let pocketed_canister_id = PocketedCanisterId::new(canister_id, &pic);
    #[allow(unused)] let default = pocketed_canister_id.sender(default_identity);
    #[allow(unused)] let alice = pocketed_canister_id.sender(alice_identity);
    #[allow(unused)] let bob = pocketed_canister_id.sender(bob_identity);
    #[allow(unused)] let carol = pocketed_canister_id.sender(carol_identity);
    #[allow(unused)] let anonymous = pocketed_canister_id.sender(anonymous_identity);

    let public_permissions = ["PauseQuery", "PermissionQuery", "BusinessExampleQuery"].iter().map(|p| p.to_string()).collect::<Vec<_>>();
    let super_permissions = ["PauseQuery", "PauseReplace", "PermissionQuery", "PermissionFind", "PermissionUpdate", "RecordFind", "RecordDelete", "ScheduleFind", "ScheduleReplace", "ScheduleTrigger", "BusinessExampleQuery", "BusinessExampleSet"].iter().map(|p| p.to_string()).collect::<Vec<_>>();
    let all_permissions = super_permissions.iter().map(|p| if public_permissions.contains(p) { Forbidden(p.clone()) } else { Permitted(p.clone()) }).collect::<Vec<_>>();

    // 🚩 1.1 permission permission_query
    assert_eq!(alice.version().unwrap(), 1_u32, "version");
    assert_eq!(alice.permission_all().unwrap(), all_permissions);
    assert_eq!(alice.permission_query().unwrap(), public_permissions);
    assert_eq!(default.permission_query().unwrap(), super_permissions);
    assert_eq!(bob.permission_update(vec![PermissionUpdatedArg::UpdateUserPermission(alice_identity, Some(vec!["PermissionUpdate".to_string(), "PermissionQuery".to_string()]))]).unwrap_err().reject_message, "Permission 'PermissionUpdate' is required".to_string());
    assert_eq!(default.permission_update(vec![PermissionUpdatedArg::UpdateUserPermission(alice_identity, Some(vec!["PermissionUpdate".to_string(), "PermissionQuery".to_string()]))]).unwrap(), ());
    assert_eq!(alice.permission_query().unwrap_err().reject_message, "Permission 'PermissionQuery' is required".to_string());
    assert_eq!(default.permission_query().unwrap(), super_permissions);
    assert_eq!(default.permission_find_by_user(alice_identity).unwrap(), ["PauseQuery", "PermissionUpdate", "BusinessExampleQuery"].iter().map(|p| p.to_string()).collect::<Vec<_>>());
    assert_eq!(alice.permission_update(vec![PermissionUpdatedArg::UpdateUserPermission(alice_identity, None)]).unwrap(), ());
    assert_eq!(alice.permission_query().unwrap(), public_permissions);
    assert_eq!(default.permission_query().unwrap(), super_permissions);

    // 🚩 1.2 permission permission update
    assert_eq!(default.permission_query().unwrap(), super_permissions);
    assert_eq!(alice.permission_query().unwrap(), public_permissions);
    assert_eq!(default.permission_find_by_user(default_identity).unwrap(), super_permissions);
    assert_eq!(default.permission_find_by_user(alice_identity).unwrap(), public_permissions);
    assert_eq!(alice.permission_find_by_user(default_identity).unwrap_err().reject_message, "Permission 'PermissionFind' is required".to_string());
    assert_eq!(alice.permission_find_by_user(alice_identity).unwrap_err().reject_message, "Permission 'PermissionFind' is required".to_string());
    assert_eq!(alice.permission_roles_all().unwrap_err().reject_message, "Permission 'PermissionFind' is required".to_string());

    // 🚩 1.3 permission roles
    assert_eq!(alice.permission_query().unwrap(), public_permissions);
    assert_eq!(default.permission_update(vec![PermissionUpdatedArg::UpdateRolePermission("Admin".to_string(), Some(vec!["PauseReplace".to_string(), "PauseQuery".to_string()]))]).unwrap(), ());
    assert!(default.permission_roles_all().unwrap().iter().any(|(role, _)| role == "Admin"));
    assert_eq!(default.permission_update(vec![PermissionUpdatedArg::UpdateUserRole(alice_identity, Some(vec!["Admin".to_string()]))]).unwrap(), ());
    assert_eq!(alice.permission_query().unwrap(), ["PauseReplace", "PermissionQuery", "BusinessExampleQuery"].iter().map(|p| p.to_string()).collect::<Vec<_>>());
    assert_eq!(default.permission_update(vec![PermissionUpdatedArg::UpdateUserRole(alice_identity, None)]).unwrap(), ());
    assert_eq!(alice.permission_query().unwrap(), public_permissions);

    // 🚩 2.1 pause permission
    assert!(!default.pause_query().unwrap());
    assert_eq!(default.pause_query_reason().unwrap(), None);
    assert_eq!(default.pause_replace(Some("reason".to_string())).unwrap(), ());
    assert!(default.pause_query().unwrap());
    assert_eq!(default.pause_query_reason().unwrap().unwrap().message, "reason".to_string());

    // 🚩 2.2 pause permission by alice
    assert!(alice.pause_query().unwrap());
    assert_eq!(alice.pause_query_reason().unwrap().unwrap().message, "reason".to_string());

    // 🚩 2.3 pause no permission
    assert_eq!(alice.pause_replace(None).unwrap_err().reject_message, "Permission 'PauseReplace' is required".to_string());
    assert_eq!(default.permission_update(vec![PermissionUpdatedArg::UpdateUserPermission(alice_identity, Some(vec!["PauseReplace".to_string(), "PauseQuery".to_string()]))]).unwrap(), ());
    assert_eq!(alice.pause_replace(None).unwrap(), ());
    assert_eq!(alice.pause_query().unwrap_err().reject_message, "Permission 'PauseQuery' is required".to_string());
    assert_eq!(alice.pause_query_reason().unwrap_err().reject_message, "Permission 'PauseQuery' is required".to_string());
    assert!(!default.pause_query().unwrap());
    assert_eq!(default.pause_query_reason().unwrap(), None);

    // 🚩 3 record no permission
    assert_eq!(alice.record_topics().unwrap_err().reject_message, "Permission 'RecordFind' is required".to_string());
    assert_eq!(default.record_topics().unwrap(), ["Example", "CyclesCharge", "Upgrade", "Schedule", "Record", "Permission", "Pause", "Initial"].iter().map(|t| t.to_string()).collect::<Vec<_>>());
    let mut page_data = default.record_find_by_page(QueryPage { page: 1, size: 1 }, Some(RecordSearchArg{ id_range: None, created_at_nanos_range: None, topic: Some(vec!["Pause".to_string()]), content: None, caller: None })).unwrap();
    assert_eq!(page_data.total, 2);
    assert_eq!(page_data.page, 1);
    assert_eq!(page_data.size, 1);
    assert_eq!(page_data.data.len(), 1);
    let record_id = page_data.data.pop().unwrap().id;
    let record_operation_count_before = default.record_find_by_page(QueryPage { page: 1, size: 10 }, Some(RecordSearchArg{ id_range: None, created_at_nanos_range: None, topic: Some(vec!["Record".to_string()]), content: None, caller: None })).unwrap().total;
    assert_eq!(alice.record_delete(vec![record_id]).unwrap_err().reject_message, "Permission 'RecordDelete' is required".to_string());
    assert_eq!(default.record_delete(Vec::new()).unwrap(), 0);
    assert!(default.record_delete(vec![u64::MAX; 1001]).unwrap_err().reject_message.contains("Record delete size must not exceed 1000."));
    assert_eq!(default.record_delete(vec![record_id, record_id, u64::MAX]).unwrap(), 1);
    let page_data = default.record_find_by_page(QueryPage { page: 1, size: 10 }, Some(RecordSearchArg{ id_range: Some((Some(record_id), Some(record_id))), created_at_nanos_range: None, topic: None, content: None, caller: None })).unwrap();
    assert_eq!(page_data.total, 0);
    assert_eq!(default.record_delete(vec![record_id, u64::MAX]).unwrap(), 0);
    let record_operation_count_after = default.record_find_by_page(QueryPage { page: 1, size: 10 }, Some(RecordSearchArg{ id_range: None, created_at_nanos_range: None, topic: Some(vec!["Record".to_string()]), content: None, caller: None })).unwrap().total;
    assert_eq!(record_operation_count_after, record_operation_count_before);

    // 🚩 4 schedule
    assert_eq!(alice.schedule_find().unwrap_err().reject_message, "Permission 'ScheduleFind' is required".to_string());
    assert_eq!(default.schedule_find().unwrap(), None);
    assert_eq!(alice.schedule_replace(Some(1000000000)).unwrap_err().reject_message, "Permission 'ScheduleReplace' is required".to_string());
    assert!(default.schedule_replace(Some(0)).unwrap_err().reject_message.contains("Schedule interval must be at least 1000000000 nanoseconds."));
    assert_eq!(default.schedule_replace(Some(1000000000)).unwrap(), ());
    std::thread::sleep(std::time::Duration::from_secs(3)); // 🕰︎
    let page_data = default.record_find_by_page(QueryPage { page: 1, size: 1000 }, Some(RecordSearchArg{ id_range: None, created_at_nanos_range: None, topic: Some(vec!["Schedule".to_string()]), content: None, caller: None })).unwrap();
    let scheduled_tasks_before_pause = page_data.data.iter().filter(|record| record.content.is_empty()).count();
    assert_eq!(default.pause_replace(Some("schedule maintenance".to_string())).unwrap(), ());
    assert_eq!(default.schedule_find().unwrap(), Some(1000000000));
    std::thread::sleep(std::time::Duration::from_secs(2)); // 🕰︎
    let page_data = default.record_find_by_page(QueryPage { page: 1, size: 1000 }, Some(RecordSearchArg{ id_range: None, created_at_nanos_range: None, topic: Some(vec!["Schedule".to_string()]), content: None, caller: None })).unwrap();
    let scheduled_tasks_during_pause = page_data.data.iter().filter(|record| record.content.is_empty()).count();
    assert_eq!(scheduled_tasks_during_pause, scheduled_tasks_before_pause);
    assert!(default.schedule_trigger().unwrap_err().reject_message.contains("Canister is paused: schedule maintenance"));
    assert_eq!(default.schedule_replace(None).unwrap(), ());
    assert_eq!(default.schedule_find().unwrap(), None);
    assert_eq!(default.pause_replace(None).unwrap(), ());
    assert_eq!(alice.schedule_trigger().unwrap_err().reject_message, "Permission 'ScheduleTrigger' is required".to_string());
    assert_eq!(default.schedule_trigger().unwrap(), ());
}
