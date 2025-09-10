// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{
    self, CandidType, Decode, Deserialize, Encode, Principal, encode_args, encode_one, utils::ArgumentEncoder,
};
use pocket_ic::RejectResponse;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct InitArg {
    pub supers: Option<Vec<Principal>>,
    pub schedule: Option<candid::Nat>,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum InitArgs {
    V0(InitArg),
    V1(InitArg),
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct ExampleVec {
    pub vec_data: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct MemoryMetrics {
    pub wasm_binary_size: candid::Nat,
    pub wasm_chunk_store_size: candid::Nat,
    pub canister_history_size: candid::Nat,
    pub stable_memory_size: candid::Nat,
    pub snapshots_size: candid::Nat,
    pub wasm_memory_size: candid::Nat,
    pub global_memory_size: candid::Nat,
    pub custom_sections_size: candid::Nat,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum CanisterStatusType {
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "running")]
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum LogVisibility {
    #[serde(rename = "controllers")]
    Controllers,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "allowed_viewers")]
    AllowedViewers(Vec<Principal>),
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct DefiniteCanisterSettings {
    pub freezing_threshold: candid::Nat,
    pub wasm_memory_threshold: candid::Nat,
    pub controllers: Vec<Principal>,
    pub reserved_cycles_limit: candid::Nat,
    pub log_visibility: LogVisibility,
    pub wasm_memory_limit: candid::Nat,
    pub memory_allocation: candid::Nat,
    pub compute_allocation: candid::Nat,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct QueryStats {
    pub response_payload_bytes_total: candid::Nat,
    pub num_instructions_total: candid::Nat,
    pub num_calls_total: candid::Nat,
    pub request_payload_bytes_total: candid::Nat,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct CanisterStatusResult {
    pub memory_metrics: MemoryMetrics,
    pub status: CanisterStatusType,
    pub memory_size: candid::Nat,
    pub cycles: candid::Nat,
    pub settings: DefiniteCanisterSettings,
    pub query_stats: QueryStats,
    pub idle_cycles_burned_per_day: candid::Nat,
    pub module_hash: Option<serde_bytes::ByteBuf>,
    pub reserved_cycles: candid::Nat,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct PauseReason {
    pub timestamp_nanos: candid::Int,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum Permission {
    Permitted(String),
    Forbidden(String),
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum PermissionUpdatedArg {
    UpdateRolePermission(String, Option<Vec<String>>),
    UpdateUserPermission(Principal, Option<Vec<String>>),
    UpdateUserRole(Principal, Option<Vec<String>>),
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct QueryPage {
    pub page: u64,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct RecordSearchArg {
    pub id: Option<(Option<u64>, Option<u64>)>,
    pub created: Option<(Option<u64>, Option<u64>)>,
    pub topic: Option<Vec<String>>,
    pub content: Option<String>,
    pub caller: Option<Vec<Principal>>,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Record {
    pub id: u64,
    pub created: candid::Int,
    pub topic: u8,
    pub content: String,
    pub done: Option<(candid::Int, String)>,
    pub caller: Principal,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct PageData {
    pub total: u64,
    pub data: Vec<Record>,
    pub page: u64,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct MigratedRecords {
    pub records: Vec<Record>,
    pub next_id: u64,
    pub removed: u64,
}

#[derive(Clone, Copy)]
pub struct PocketedCanisterId<'a> {
    pub canister_id: Principal,
    pub pic: &'a pocket_ic::PocketIc,
}

impl<'a> PocketedCanisterId<'a> {
    pub fn new(canister_id: Principal, pic: &'a pocket_ic::PocketIc) -> Self {
        Self { canister_id, pic }
    }
    pub fn sender(&self, sender: Principal) -> Service<'a> {
        Service { pocket: *self, sender }
    }
}

type Result<R> = std::result::Result<R, RejectResponse>;
pub struct Service<'a> {
    pub pocket: PocketedCanisterId<'a>,
    pub sender: Principal,
}
impl Service<'_> {
    fn query_call<R: CandidType + DeserializeOwned>(&self, method: &str, payload: Vec<u8>) -> Result<R> {
        let response = self
            .pocket
            .pic
            .query_call(self.pocket.canister_id, self.sender, method, payload)?;
        let result = Decode!(response.as_slice(), R).unwrap();
        Ok(result)
    }
    fn update_call<R: CandidType + DeserializeOwned>(&self, method: &str, payload: Vec<u8>) -> Result<R> {
        let response = self
            .pocket
            .pic
            .update_call(self.pocket.canister_id, self.sender, method, payload)?;
        let result = Decode!(response.as_slice(), R).unwrap();
        Ok(result)
    }

    // ======================= common apis =======================

    pub fn get_candid_interface_tmp_hack(&self) -> Result<String> {
        self.query_call("__get_candid_interface_tmp_hack", Encode!(&()).unwrap())
    }
    pub fn canister_status(&self) -> Result<CanisterStatusResult> {
        self.update_call("canister_status", Encode!(&()).unwrap())
    }
    pub fn pause_query(&self) -> Result<bool> {
        self.query_call("pause_query", Encode!(&()).unwrap())
    }
    pub fn pause_query_reason(&self) -> Result<Option<PauseReason>> {
        self.query_call("pause_query_reason", Encode!(&()).unwrap())
    }
    pub fn pause_replace(&self, arg0: Option<String>) -> Result<()> {
        self.update_call("pause_replace", encode_one(&arg0).unwrap())
    }
    pub fn permission_all(&self) -> Result<Vec<Permission>> {
        self.query_call("permission_all", Encode!(&()).unwrap())
    }
    pub fn permission_assigned_by_user(&self, arg0: Principal) -> Result<Option<Vec<Permission>>> {
        self.query_call("permission_assigned_by_user", encode_one(&arg0).unwrap())
    }
    pub fn permission_assigned_query(&self) -> Result<Option<Vec<Permission>>> {
        self.query_call("permission_assigned_query", Encode!(&()).unwrap())
    }
    pub fn permission_find_by_user(&self, arg0: Principal) -> Result<Vec<String>> {
        self.query_call("permission_find_by_user", encode_one(&arg0).unwrap())
    }
    pub fn permission_query(&self) -> Result<Vec<String>> {
        self.query_call("permission_query", Encode!(&()).unwrap())
    }
    pub fn permission_roles_all(&self) -> Result<Vec<(String, Vec<Permission>)>> {
        self.query_call("permission_roles_all", Encode!(&()).unwrap())
    }
    pub fn permission_roles_by_user(&self, arg0: Principal) -> Result<Option<Vec<String>>> {
        self.query_call("permission_roles_by_user", encode_one(&arg0).unwrap())
    }
    pub fn permission_roles_query(&self) -> Result<Option<Vec<String>>> {
        self.query_call("permission_roles_query", Encode!(&()).unwrap())
    }
    pub fn permission_update(&self, arg0: Vec<PermissionUpdatedArg>) -> Result<()> {
        self.update_call("permission_update", encode_one(&arg0).unwrap())
    }
    pub fn record_find_by_page(&self, arg0: QueryPage, arg1: Option<RecordSearchArg>) -> Result<PageData> {
        self.query_call("record_find_by_page", encode_args((&arg0, &arg1)).unwrap())
    }
    pub fn record_migrate(&self, arg0: u32) -> Result<MigratedRecords> {
        self.update_call("record_migrate", encode_one(&arg0).unwrap())
    }
    pub fn record_topics(&self) -> Result<Vec<String>> {
        self.query_call("record_topics", Encode!(&()).unwrap())
    }
    pub fn schedule_find(&self) -> Result<Option<u64>> {
        self.query_call("schedule_find", Encode!(&()).unwrap())
    }
    pub fn schedule_replace(&self, arg0: Option<u64>) -> Result<()> {
        self.update_call("schedule_replace", encode_one(&arg0).unwrap())
    }
    pub fn schedule_trigger(&self) -> Result<()> {
        self.update_call("schedule_trigger", Encode!(&()).unwrap())
    }
    pub fn version(&self) -> Result<u32> {
        self.query_call("version", Encode!(&()).unwrap())
    }
    pub fn wallet_balance(&self) -> Result<candid::Nat> {
        self.query_call("wallet_balance", Encode!(&()).unwrap())
    }
    pub fn wallet_receive(&self) -> Result<candid::Nat> {
        self.query_call("wallet_receive", Encode!(&()).unwrap())
    }
    pub fn whoami(&self) -> Result<Principal> {
        self.query_call("whoami", Encode!(&()).unwrap())
    }

    // ======================= business apis =======================

    pub fn business_example_cell_query(&self) -> Result<String> {
        self.query_call("business_example_cell_query", Encode!(&()).unwrap())
    }
    pub fn business_example_cell_set(&self, arg0: String) -> Result<()> {
        self.update_call("business_example_cell_set", encode_one(&arg0).unwrap())
    }
    pub fn business_example_cell_set2(&self, arg0: String) -> Result<()> {
        self.update_call("business_example_cell_set2", encode_one(&arg0).unwrap())
    }
    pub fn business_example_cell_set3(&self, arg0: String) -> Result<()> {
        self.update_call("business_example_cell_set3", encode_one(&arg0).unwrap())
    }
    pub fn business_example_cell_set4(&self, arg0: String) -> Result<()> {
        self.update_call("business_example_cell_set4", encode_one(&arg0).unwrap())
    }
    pub fn business_example_log_query(&self) -> Result<Vec<String>> {
        self.query_call("business_example_log_query", Encode!(&()).unwrap())
    }
    pub fn business_example_log_update(&self, arg0: String) -> Result<u64> {
        self.update_call("business_example_log_update", encode_one(&arg0).unwrap())
    }
    pub fn business_example_map_query(&self) -> Result<Vec<(u64, String)>> {
        self.query_call("business_example_map_query", Encode!(&()).unwrap())
    }
    pub fn business_example_map_update(&self, arg0: u64, arg1: Option<String>) -> Result<Option<String>> {
        self.update_call("business_example_map_update", encode_args((&arg0, &arg1)).unwrap())
    }
    pub fn business_example_priority_queue_pop(&self) -> Result<Option<u64>> {
        self.update_call("business_example_priority_queue_pop", Encode!(&()).unwrap())
    }
    pub fn business_example_priority_queue_push(&self, arg0: u64) -> Result<()> {
        self.update_call("business_example_priority_queue_push", encode_one(&arg0).unwrap())
    }
    pub fn business_example_priority_queue_query(&self) -> Result<Vec<u64>> {
        self.query_call("business_example_priority_queue_query", Encode!(&()).unwrap())
    }
    pub fn business_example_query(&self) -> Result<String> {
        self.query_call("business_example_query", Encode!(&()).unwrap())
    }
    pub fn business_example_set(&self, arg0: String) -> Result<()> {
        self.update_call("business_example_set", encode_one(&arg0).unwrap())
    }
    pub fn business_example_count_query(&self) -> Result<u64> {
        self.query_call("business_example_count_query", Encode!(&()).unwrap())
    }
    pub fn business_example_count_set(&self, arg0: u64) -> Result<()> {
        self.update_call("business_example_count_set", encode_one(&arg0).unwrap())
    }
    pub fn business_example_count_set2(&self, arg0: u64) -> Result<()> {
        self.update_call("business_example_count_set2", encode_one(&arg0).unwrap())
    }
    pub fn business_example_count_set3(&self, arg0: u64) -> Result<()> {
        self.update_call("business_example_count_set3", encode_one(&arg0).unwrap())
    }
    pub fn business_example_vec_pop(&self) -> Result<Option<ExampleVec>> {
        self.update_call("business_example_vec_pop", Encode!(&()).unwrap())
    }
    pub fn business_example_vec_push(&self, arg0: u64) -> Result<()> {
        self.update_call("business_example_vec_push", encode_one(&arg0).unwrap())
    }
    pub fn business_example_vec_query(&self) -> Result<Vec<ExampleVec>> {
        self.query_call("business_example_vec_query", Encode!(&()).unwrap())
    }
}
