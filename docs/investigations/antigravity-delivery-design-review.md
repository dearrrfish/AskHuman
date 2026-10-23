# Antigravity 本机 v1 交付设计审阅

审阅日期：2026-10-18。仅报告，不修改产品、原计划或运行状态，不安装、不提交。

## 结论

**建议基于当前版本完善，保留已有基础实现；重新组织交付计划，并对可信来源这一未解决的局部设计重新作可行性裁决。没有证据支持整套推倒重写。**

进度缓慢不是简单的“代码写得慢”或“review 太严格”。主要问题是：把尚未证明可行的可信 MCP 当作后续集成工作；多次以局部探针和文档完成收口，却没有关闭 v1 的关键前提；验收采集器与真实宿主输出合同不一致，导致测完才补证据；完整需求到具体 case 的映射存在遗漏。版本升级、共享路径回归与审查流程又放大了这些成本。

截至本次快照，**本机完整 v1 未完成**。基础 CLI 已有实现和历史实机证据；可信 MCP 仍被产品显式关闭。最新工作是 1.3.5 CLI 运输观测与验收准备，没有当前完整真实运输验收或可信 adapter 的完成证据。不能把 CLI-only 作为原目标的替代交付。

## 审阅口径与证据身份

- 用户指定的 `.git/worktrees/antigravity-support` 是 Git 管理目录；实际源码位于 `.worktrees/antigravity-support`，分支 `docs/antigravity-support-plan`。
- HEAD 为 `e8f1267554f43ead0c00870e53ab1be91d0067ee`；主 checkout 审阅前干净，目标 worktree 有 83 条 porcelain 状态记录，包含未跟踪目录，不能据此称“83 个修改文件”。未提交源码也纳入审阅。
- 追踪文件 `git diff --binary HEAD` SHA256：`f9656da2f12f472fef00395b8c1d5df017b9443a1e5ab2755218f8e3e46d9846`。包含未跟踪材料在内的 948 个相关文件哈希、23 个 manifest 摘要另存于本次审阅的 `evidence-snapshot.json`。
- 最高边界为目标 worktree 的 `docs/specs/antigravity-v1.md`，尤其“必需能力”“V1-D01/D02”“与旧计划的优先级”。2026-10-18 UTC 的 1.3.5 更新只换验收 profile，不撤销 10/17 的 v1 边界。
- 读取当前源码、run manifest、计划、审查报告、调查及现存测试日志；没有重跑产品测试、安装或 native 用例。下文通过数字均属于原执行者的历史日志，不是本次重新验收。
- 时间按 manifest UTC 排序。大多数 `updated_at` 等于 `created_at`，不能拿它计算完成时间或实际工时。`running` 仅是持久化任务记录，本报告未据此断言 worker 仍在运行。

下文源码、文档和 run 引用均相对目标 worktree；`runs/` 简写 `.agent-team/runs/`。本次审阅材料保存在主项目 `.agent-team/runs/antigravity-delivery-design-audit/`，不改历史记录。

## 当前完成程度：按 v1 而非旧家族计划衡量

| v1 要求 | 当前证据 | 交付判断 |
| --- | --- | --- |
| 本机 NixOS、agy CLI、Popup | v1 已明确；当前准备 profile 为 agy 1.3.5 / AskHuman 0.15.0 | 保持范围；IDE、其它 OS、IM、高级 N2–N4 不进入完成依赖 |
| None/CLI/MCP 与真实产物一致 | `integrations/agent_mode.rs:308–320` 实现 None/CLI，拒绝 MCP；`mcp_config.rs:125–126` 排除 Antigravity | 基础接线可复用；完整三模式尚未完成 |
| CLI 问答、附件、待办、交接、正确等待 | 旧 P0 与 N0 有真实问答、路径/附件等证据；最新版本 schema 已有补充观察 | 已有基础价值；不能把旧 build 的通过移交给 1.3.5 全矩阵 |
| 可信 ingress、逐调用 CID、主/child、项目 | F0 对固定 1.2.16 证据作 No-Go；GF-01 出现反例；当前源码未开放可信 MCP | v1 核心阻塞；1.3.5 升级既不自动解决，也不自动继承旧反例的适用性 |
| 六 MCP 工具与 exact CID show_last | 六 canary 元数据观察、合成 broker 测试存在；无真实产品 adapter 闭环 | 未完成；canary 与模拟不等于业务调用 |
| 配置加载、稀疏所有权、漂移拒绝、恢复 | CLI 独占规则 writer 已实现；MCP 共享配置 C1 仍待冻结及验收 | CLI writer 可复用；不能直接用独占文件策略覆盖共享 MCP 配置 |
| CLI/MCP 各自 30/120/600 秒、取消、EOF/drain、并发单终态 | 旧版本有有界等待结果；最新 transport run 仍 implementation，review pending | 当前完整矩阵未完成；清理进程不等于原生取消通过 |
| 相关回归、最终安装、独立审查 | 新 transport 日志：Vitest 780；四组定向 Rust 3/7/12/17 通过并完成 Clippy；此前全 Rust 1378 通过、1 Cursor 实机依赖失败、3 忽略 | 最新变更有局部验证，但最终安装/真实验收/审查仍缺；不得写“全测试绿” |

最新代码已把 `Accepted.request_id` 接入 `perf::mark_cli_accepted`（`client/mod.rs:776`、`perf.rs:114`），并加入按请求标记 Popup 绘制的观测。`runs/antigravity-cli-135-transport/RECOVERY.md` 仍描述“待 dispatch / request_id 被丢弃”，已落后于源码及 T-003/T-004 报告。恢复时必须以当前源码、manifest 和测试日志交叉核对，不能照旧恢复文案重做。

## review 数量：有累积成本，不能统称超量返工

23 个顶层 run 的 `review.used` 合计 **36**：11 个 run 各 2 轮，10 个各 1 轮，1 个 4 轮，1 个 0 轮。状态为 19 complete、2 planned、1 discovery、1 implementation。所有这批历史 run 的 tier 都是 team；不是每个 run 都有四个实际并发 worker。

| 时间段（UTC） | run 数 | 记录的 review 数 | 含义 |
| --- | --- | --- | --- |
| 10/04–10/05，初始支持至 NixOS-first | 9 | 16 | 家族范围规划、载体、P0 和基础验收 |
| 10/16–10/17 06:18，N0 至 source-design-audit | 7 | 12 | 来源/项目多轮诊断；唯一四轮 run 在这里 |
| 10/17 18:18 v1-scope 起至最新 transport | 7 | 8 | 收窄后仍主要在可行性/夹具/运输准备，无 run 超过 2 轮 |

这些是 manifest 账面完成轮数，不是所有临时反馈、审查中断或修改次数，更不是工作小时。很多 C1/C2 是“执行前安全预检 + 执行后证据审查”，两轮本身合理。

唯一四轮的 `antigravity-n1-profile-probes` 有确切返工链：C2 后发现原 A 与独立控制 A 来自不同 fixture 目录，没有覆盖 FR002 要求的同一绝对目录连续重启；于是追加 C3 预检和 C4 结果审查。见该 run `completion-audit.md:5–11`。用户明确批准额度扩到 4；这不是未经许可超限。但 `review-cycle-4.md:68–74` 记录 validator 不支持这个额度，退出 1。不能把流程兼容性错误说成产品缺陷，也不能称 validator 通过。

因此，**“许多审查”的观感有根据，但“多轮审查无必要”没有得到证据支持**。更准确的问题是微小调查被串成许多完整 run，且局部 approved 无法回答“离本机完整可用还差什么”。

## 根因与影响

### D1 / 高：关键可行性风险未在交付承诺前关闭

早期家族计划对 P0 文件事务、UI、分工非常详细，但 P1 以“来源验证后绑定”“需独立证据后追加”描述可信来源（旧 `runs/antigravity-support/plan.md` 的“P1/P2适配边界”）。这适合探索路线，不足以作为完整可用版本的确定性交付计划。10/17 v1 将可信 MCP 明确列为必需后，它就成为必须先裁决的关键路径。

后续诊断逐步证明 clientInfo、UUID、ancestry 和 pipe 形状不足以授信；GF-01 又给出 opposite helper 匹配候选 predicate 的反例。F0 明列三个缺口：保护原生请求/响应通道、唯一调用因果关系、权威逐调用项目 snapshot。见 `docs/investigations/antigravity-v1-f0-f1.md`“F0”。继续增加同类相关性样本不能代替不变量。

这是“计划详尽，但核心前提仍探索中”，不能仅靠更细任务表补救。也不能据现有材料断言上游永远不可能支持；F0 明确说明 collector 未保留全部未知 metadata 或 JSON-RPC ID。

### D2 / 高：局部完成不断累积，缺少以产品门槛为中心的收敛机制

scope、source-options、F0/F1 等 run 都 complete；一些 `live_validation=true` 实际只覆盖有界文档/诊断范围。例如 F0/F1 没有新 native 执行，仍可按该 run 范围完成；这不是证据造假，但字段本身不足以表达产品 readiness。

最新 transport 计划仍在推进必要的 CLI 验收，它有独立价值；不过无法解除 A1/A2。若只沿“再准备一个探针 → 两轮审查 → 有界完成”的节奏前进，会持续消耗成本，而可信 MCP 的交付日期仍不可估计。v1 收窄有效减少了范围，却没有解决这一技术依赖。

### D3 / 高：验收可观测性设计偏晚，测试系统自身成为开发项目

1.3.5 的三秒 schema 用例成功执行，但 public stream 缺 Cwd/Wait、原始 TaskId 和 finish 信息；原 gate 保留失败。随后单独授权读取一份 journal 才补齐部分观察，又启动新 run 增加 exact AskHuman request UUID 与绘制标记。见 `runs/antigravity-cli-135-journal-intake/final-report.md`、最新 transport `requirements.md` 和 T-003/T-004 报告。

前置观测字段表如果先区分“宿主实际行为”和“某输出流是否公开字段”，可以少做这类先执行、后换证据载体的返工。新 UUID 观测有实用价值，但它是运营关联，不能自动升级为可信 CID 来源。

采集器本身也曾漏算额外 RUNNING/FAILED 命令，直到审查发现；见 `antigravity-cli-1216-acceptance/review-cycle-1.md` 的 C1-F01。这说明部分 review 是在补测试 oracle 的缺陷，不宜直接删减。

### D4 / 中：需求到 case 的覆盖检查晚于审查，且并行接口冻结仍有遗漏

同目录重启遗漏直接增加两轮审查；新 paint 实现一度把实际 `pid-epoch_ms` perf ID 按 UUID 验证，由 root 在审查前纠正（最新 T-004 报告）。共同原因是自然语言合同没有完全落到可检查的输入类型、用例身份及断言上。

改善点是执行前让 owner 对每条必需项写出 case、字段来源、断言和失败处置，并用真实格式的小样本验证 collector；不是让 reviewer 多读几遍长文档。

### D5 / 中：版本漂移、长期未提交改动与状态文档分散放大恢复成本

证据跨 1.2.9、1.2.16、1.3.5 和 AskHuman 0.14.5/0.15.0；恢复曾涉及 main 同步、58 个新增文件恢复及重验证（目标 `docs/PROGRESS.md:174–178`）。最新 manifest、tasks、RECOVERY 与源码细节不同步，又增加定位成本。

当前目录还含 43 个 probe 文件、828 个历史 run 文件，包含 rendered 配置和验证材料；文件数量不能当作无效工作比例，但足以说明人工反复重读的负担。应保留证据，另建一个当前门槛索引，而不是删除失败历史或不断复制旧路线。

材料不足以计算各根因占用的精确工时，也不能把 10/05 到 10/16 的日历间隔全算成开发耗时。没有证据将延误归因于某个个人或模型能力。

## 已完成工作的复用度

不提供“完成了百分之多少”的伪精度：代码体量、测试数与剩余关键风险不成比例。按资产判断如下。

| 资产 | 复用度 | 处理建议 |
| --- | --- | --- |
| AgentKind/target、None/CLI、设置/doctor、提示规则、家族识别与冲突拒绝 | 高 | 保留；可信 adapter 接入时扩展，不重写基础接线 |
| `antigravity_rule.rs` 独占文件事务、ownership、漂移与恢复 | 高，限原合同 | 继续服务 CLI；抽用安全机制，不直接套到共享 MCP JSON |
| Linux Popup 修复、既有 CLI/Popup/daemon 机制 | 高 | 保留并做受影响回归；不用为 Antigravity 再造一套问答生命周期 |
| 新 Accepted UUID / scoped paint 观测 | 中高 | 完成当前变更的最终验证；只作为验收观测，避免扩张为公共来源 API |
| 旧版本原生实机证据、负例与配置调查 | 中 | 复用 case、反例和经验；按 profile 作有针对性的重验，不直接继承认证 |
| 合成 broker、失败状态机及合同 | 中 | 复用测试场景/状态设计；Python 合成权威前提不直接成为 Rust 产品认证实现 |
| FD/clientInfo/UUID 来源启发式 | 诊断可复用，授信不可复用 | 保存反例；没有新保护机制，不继续当 admission 算法完善 |
| MCP adapter、权威项目绑定、MCP 配置接管 | 尚无可交付实现 | 这是剩余核心工作，不是已有大模块推倒重写 |
| IDE、跨平台和高级能力计划 | 后续资料可复用 | 从 v1 完成依赖中排除，不要求删除既有材料 |

推荐的“局部重设计”边界是可信 ingress / 调用关联 / 项目权威，不是整套 AskHuman，也不是已完成 CLI 集成。若所需 native 合同不可得，重写 AskHuman 基础层仍不能创造该合同。

## 以本机完整可用为 P1 的优化顺序

这里 P1 指用户要求的最高交付优先级，区别于历史家族计划的 P1 阶段。以下是审阅建议，不是已获授权执行的新计划。

1. **先形成一张当前 v1 门槛表。** 逐项列 owner、当前 profile、原始证据、缺口、下一动作、明确停止条件；保留 CLI 与 MCP 两条验收线。结束口径严格沿用 v1，自动模式兼容、可信来源、exact CID 恢复和配置恢复条件不降低。
2. **优先裁决 1.3.5 能否满足核心来源合同。** 有界复核新版已知接口与允许的本地材料，针对三个 F0 缺口寻找具体新机制。先出证据需求表和最小反证 case；没有机制就不启动又一组 UUID/pipe 相关性探针。旧 F0 是 1.2.16 结论，不能直接外推新版，但版本变化本身也不是 Go。
3. **给不可行分支一个明确出口。** 若仍缺原生保护/关联/项目合同，提交可审查的上游接口需求，或给出具体受控启动/隔离/项目语义方案让用户选择。这些使用方式变化需另行确认。不得暗改为 CLI-only、固定 cwd 授信、关闭自动模式或改用 SDK 来冒充已完成 v1。缺口未解除前，不承诺完整交付日期。
4. **并行只保留一条有界 CLI 收尾线。** 复用最新 UUID/paint 工作，先完成 helper 与证据采集器的离线验证，再冻结一次完整运输矩阵所需的 native/public stream/journal 可见性及权限范围。用户对未来 journal 的授权不能从旧单文件授权推导。此线可关闭真实 CLI 门槛，但不能遮蔽 MCP 仍 blocked。
5. **来源 Go 后做最小产品纵向闭环。** 先证明真实 main/child 调用进入统一 VerifiedContext、单 canonical project、一次 dispatch、精确响应；同时做同参跨 CID、错来源/错项目/失效 epoch 的零副作用反例。然后六工具共同接入，完成 exact show_last、配置 C1、图片/附件和 CLI/MCP 各自运输矩阵。保持其余家族既有语义。
6. **最后一次以最终代码验收本机交付。** 相关回归、build/lint、隔离 install、实际 Popup、两 transport 的等待/取消/EOF/drain/并发单终态以及独立结果审查。当前 Cursor 实机测试失败须明确归因或隔离环境复核，不用“无关”替代证据；也不因该失败去扩大 Antigravity 功能范围。

建议把探索预算写成可执行规则：一次只验证一个能够改变 Go/No-Go 的机制；每个 case 写清“结果为 A/B 时下一步是什么”；若两种结果都只会产生下一轮相似探针，就不应排进当前关键路径。这是控制方向，不是降低验证标准。

## 怎样减少 review，而不降低正确性

- 对实际改配置、启动 native、接触敏感证据的批次，保留一次具体预检与一次最终结果审查；不要把合理的 C1/C2 当成异常。
- 只读说明、状态更新、已有证据索引可按共同问题打包审查，使用 assisted 或单协调者加一名独立 reviewer；不机械地为每份微小补充启动完整 team 流程。
- 预检前完成“需求 → case → 原始字段 → 断言 → 失败处置”映射，特别检查同目录、同 profile、重启身份、并发同参和真正取消；不要到 C2 后才做覆盖审计。
- 分开标记 implementation-ready、diagnostic-accepted、product-gate-passed；每次 approved 都注明本次关闭哪个 v1 门槛，若没有关闭就直说。
- 共享改动冻结类型与样本，再派发；最终审查只读最终 diff 和相关证据。证据、fixture、命令改变时复核影响范围，不挪用旧批准，也不无差别重跑全部历史调查。
- 不靠反复开新 run 规避 review 额度。明确的新问题可以建立新 scope，但重复发现保留原 ID；达到额度仍未通过时总结具体阻塞与下一决定。

**最终判断：保留实现、重排关键路径、对可信来源做有界局部重设计。** 当前资产复用价值高；当前完整 v1 的可交付性仍受未证明的 native 来源合同制约。解决这项不确定性，比增加人手或减少几轮必要审查更能缩短剩余周期。
