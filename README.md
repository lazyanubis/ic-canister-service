# ic-canister-service

一个使用 Rust 2024 编写的 Internet Computer Canister 服务模板。项目提供版本化状态、维护模式、权限、
操作记录、定时任务和 PocketIC 升级回归，并同时演示堆内存状态与 Stable Structures 的使用方式。

## 系统功能

- Canister 基础能力：cycles 余额与充值、Canister 状态、调用者身份和当前状态版本查询。
- 维护模式：记录维护原因；进入维护前等待定时任务空闲并停止 timer，退出维护后恢复 timer。
- 权限与角色：支持用户权限、角色权限和用户角色关系；`Permitted` 默认无权，`Forbidden` 默认有权。
- 操作 Record：记录调用人、主题、参数、完成时间和结果，支持保留上限、带条件的分页查询与按
  `RecordId` 批量删除。
- 定时任务：支持配置周期、自动执行和手动触发；自动与手动执行共用防重入锁。
- 示例业务：演示普通堆字段以及 Stable Cell、Vec、BTreeMap、Log 和 PriorityQueue 的读写与升级保持。

## 内存与持久化模型

本项目不是“只使用堆内存”，而是混合使用 Wasm 堆内存和稳定内存：

| 区域 | 当前用途 | 升级行为 | IC 平台上限 |
| --- | --- | --- | --- |
| Wasm 堆内存 | `CanisterKit`、`example_data`、`example_count` 等普通 Rust 状态 | 升级会清空；本项目在 `pre_upgrade` 中序列化到稳定内存，并在 `post_upgrade` 中恢复 | 当前目标是 wasm32，理论硬上限为 **4 GiB** |
| 稳定内存 | 升级快照，以及 `StableCell`、`StableVec`、`StableBTreeMap`、`StableLog`、`StablePriorityQueue` | Canister 升级后原地保留；Stable Structures 不进入堆快照 | 每个 Canister 理论上限为 **500 GiB** |

这里的 **500 GiB** 是当前 IC 官方资源上限，不是旧文档中的 400G。它不表示单次调用可以读写全部空间：
稳定内存仍受单消息访问/写入上限、子网可用容量、Canister memory settings 和 cycles 余额约束。当前限制以
[IC resource limits](https://docs.internetcomputer.org/references/resource-limits/) 为准。

堆内存同样不能按 4 GiB 用满后再考虑升级。序列化堆状态会额外消耗 heap、指令和升级消息的稳定内存
读写预算；状态越接近上限，`pre_upgrade` 越可能无法完成。长期增长、必须直接跨升级保留的数据应优先评估
Stable Structures 或拆分 Canister，而不是无限扩大堆快照。参见
[IC data persistence](https://docs.internetcomputer.org/guides/backends/data-persistence/)。

## 稳定内存分区

`ic-canister-kit` 使用 Memory Manager 将稳定内存划分为虚拟分区。当前分配如下：

| MemoryId | 用途 |
| --- | --- |
| `0` | `StableCell<ExampleCell>` |
| `1` | `StableVec<ExampleVec>` |
| `2` | `StableBTreeMap<u64, String>` |
| `3` | `StableLog<String>` 的索引区 |
| `4` | `StableLog<String>` 的数据区 |
| `5` | `StablePriorityQueue<ExampleVec>` |
| `254` | `pre_upgrade` 写入的堆状态快照，由 `ic-canister-kit` 保留 |

MemoryId 和对应数据类型是持久化格式的一部分。已经使用的 ID 不得复用，也不能在后续版本中直接更换数据
结构类型；新业务需要分配新的 ID，并为数据迁移和升级回归补充测试。

## 状态版本与升级流程

- `State::V*`、`StateUpgrade`、版本化的 `InitArgs`/`UpgradeArgs` 和逐版本转换共同约束状态迁移。
- `version()` 只表示存储状态版本，不表示 API、发布版本或 Candid 版本。
- `pre_upgrade` 要求 Canister 已进入维护状态且定时任务空闲，然后停止 timer、创建升级 Record，并把
  `(record_id, state_version, byte_length, heap_bytes)` 写入 MemoryId `254`。
- `post_upgrade` 按旧版本恢复堆状态，逐步迁移到最新版本，处理升级参数，校验 schedule，重启 timer，
  最后完成升级 Record。
- Stable Structures 字段使用 `#[serde(skip)]`，升级时重新绑定原 MemoryId，不会被重复写入堆快照。
- 新增或改变持久化字段时，应新增 `src/stable/vNNN/` 和显式迁移；不要原地改变旧版本的序列化布局。

## 运行约束

- 所有用户输入、文本、bytes、列表和递归结构都应在 Canister 入口校验格式、业务范围、数量和字节大小。
- 新状态的 Record 默认最多保留 `65,536` 条；达到 `retention_limit` 后，写入新 Record 会淘汰最旧条目并
  累计淘汰数量。该限制按条目数计算，不限制单条内容的字节数，因此每项业务仍需约束写入 Record 的大小。
- Record 查询与删除的单次最大数量为 `1000`。需要完整审计历史时，应在自动淘汰前定期执行“分页 query
  -> 外部持久化 -> 按 RecordId 批量删除”。删除会去重 ID，返回实际删除数量；不存在的 ID 不报错，调用方
  可以安全重试。删除操作本身不再创建 Record，避免无法清空日志。
- schedule 参数单位为纳秒，启用时不得小于 1 秒，并且必须落入 IC timer 的 `u64` 纳秒范围。
- `schedule_trigger` 只允许在非维护模式下执行；自动与手动任务不会并发运行。
- 管理员可以主动移除全部权限，这是模板保留的运营语义，不强制要求至少存在一个管理员。
- 同一条 Canister 消息内发生 trap 时，状态和本次 Record 一同回滚；跨 `await` 的回滚边界仍遵循 IC 消息模型。

## Candid 与构建

公开 query/update、参数或返回类型变化后，重新生成并检查 `sources/source.did`：

```bash
cargo test -p service update_candid -- --ignored --nocapture
```

常用验证命令：

```bash
cargo fmt --all -- --check
cargo clippy
cargo test
cargo build --target wasm32-unknown-unknown --release
```

`dfx build service` 会执行 `dfx.json` 中的完整流程：生成 Candid、构建 release Wasm、注入 Candid
metadata、shrink 并 gzip。

## PocketIC 回归

- `bash tests/test.sh`：优先复用已有 `sources/source_opt.wasm.gz`；文件缺失时会先生成当前 Wasm。
- `bash tests/test.sh update`：运行普通测试和 Clippy，重新生成 Candid 和当前 Wasm，再执行升级、通用 API 和
  业务 API 三组 ignored PocketIC 测试。
- `tests/test.sh` 在缺少 `sources/source_opt_0_0_1.wasm.gz` 时会把当前 Wasm 复制为旧版占位文件。验证真实
  跨版本迁移前，必须确认该 fixture 确实来自目标旧版本。
- `deploy.sh` 会直接操作 `--network ic` 上的 Canister，不应作为普通本地验证命令运行。
