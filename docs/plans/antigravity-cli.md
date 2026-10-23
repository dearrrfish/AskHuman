# Antigravity 家族支持实施计划

> 2026-10-17 v1 范围调整：以[本机 NixOS CLI v1 边界](../specs/antigravity-v1.md)的已确认决定为准。仅 agy CLI、基础 CLI + 可信 MCP、Popup 必验；IDE、其它平台、IM验收与N2–N4高级能力不阻塞v1。本文完整路线/历史证据保留，不再整体作为v1收敛依赖；来源、配置、运输安全门槛保留。V1已确认单canonical项目范围（多项目歧义拒绝），exact CID show_last必需，自动compact恢复延后。

初始run `antigravity-support`；修订run `antigravity-family-plan`；tier=team。
2026-10-04人类批准按家族共享协议、按客户端验收。[需求](../specs/antigravity-cli.md)、[调查](../investigations/antigravity-cli.md)。
2026-10-04 人类授权执行 P0；本地实现与当前验收边界见下方执行记录。

## 阶段与顺位（2026-10-05 修订）

人类决定以完整支持本机 NixOS 为优先，macOS/Windows 的专用实施与真机验证后移。
当前执行顺位以[本机优先计划](antigravity-nixos-first.md)为准；下方 P0 DAG 和执行记录保留为已完成实施的合同与历史，不作为新一轮待办。

| 顺位 | 本机主线 | Gate |
| --- | --- | --- |
| 已完成 P0 | 家族、共享规则、None/CLI、事务、基础交互与真实长等待 | 保留已有证据；当前正式模式 None |
| N0 | ancestry/child、IM、桌面与路径基础收尾 | 自有来源、隔离频道、真实端到端 |
| N1 | 可信来源与 binding、MCP 六工具/图片/恢复/长等待 | 每项原生传播和600秒/取消/并发 gate；180秒取消问题不得绕过 |
| N2 | 生命周期与 Stop | 可信主child、自然结束、同UUID继续 |
| N3 | transcript/标题/Watch、插话与新任务 | 来源、消费回执和固定argv各自 gate |
| N4 | 本机权限/原生选择题/Fork接口与总体验收 | 独立契约先行；无接口保持blocked并回报，不虚构支持 |
| X 后移/可介入 | macOS/Windows writer与真机、非本机客户端独立验收 | 独立平台adapter/worktree/证据tuple；不默认阻塞本机主线 |

高级capability当前仍false，只有对应宿主/OS/能力通过后分项开放。共享接口与平台adapter边界预留，已有平台待办保留；真机或贡献者就绪可独立提前介入。N0为下一默认开发入口，本次仅改顺位，不启动产品实施、升级、永久安装或发布。

## 当前证据与执行边界

稳定probe已交付；agy1.2.9/NixOS延迟夹具30/120/600秒完成，当时原始TaskId来源与取消闭环未证明；后续范围见下方native追加验收。
真实未绑定Popup约9秒通过；非产品家族归属/长等待验收。共享临时模块严格零工具before/present/after通过，
满足已批准载体发现选择；CLI-only精确遗漏原因未知，不再阻塞共享方案。早期native自动gate仍always-proceed blocked；不是后来RequestReview实测的结论。
这些观察不等同产品放行，也不等同最终文件已安装/通过。

现worktree `.worktrees/antigravity-support`，popup-only；不抢IM租约。
功能变更后install.sh并使用新装worktree AskHuman。已授权临时Nix环境可复用；不全局装包/改系统。
原生登录由操作者提供，不读取/复制凭据。P0 实施已授权；正式规则仅临时安装验收并按所有权清理，不保留永久安装。

## 家族、模式与身份接口

新增Rust AgentKind/AgentTarget、TS AgentKind/AgentId、UI表及所有exhaustive matches，标识antigravity、标签Antigravity，
不加gemini别名。agy仅用于CLI宿主可执行检查/未来启动；IDE-only用户可装共享协议，不能因缺agy禁用全家安装。
launcher/fork仍unsupported。安装状态与宿主/version验证范围分别说明；保留现有list/status/doctor成功JSON shape，
读取错误走现有Result错误通道，UI显示无法读取而非新装/未安装。CLI-host缺binary是单独diagnostic。

P0 `AskHuman agents mode antigravity cli|none`；MCP gate后才mcp，初次模式由用户选择。
`AskHuman --caller-agent antigravity <原有参数>`主dispatch前消费/校验，unknown/重复/冲突hint报错。
每条受管CLI调用包括帮助/恢复/交接/todo都带hint；hint仅家族，不是session或权限。
继承其它家族env冲突拒绝，不借外层UUID、不让原生标记静默优先。

caller_context保留已知Antigravity且未绑定；show_last拒绝Project回退并要求重新确认，无可信binding不称恢复成功。
恢复metadata的无hint直接agy入口需正向process-tree家族识别，ancestry不提供session权威。
IDE仅在hint/env/经验证宿主识别时确认家族，通用Code.exe/extensionHost不认作Antigravity；
无法识别的IDE不宣称安全恢复已支持。普通非Agent既有Project查询不全局禁用。
ask_dedup无session/无MCPinstance不合流，两个同项目同题请求独立，不造每项目UUID。
P1精确恢复需可信binding，弱instance+project键须标限制，不用环境UUID或最近日志猜绑定。

## 共享协议及工具段

固定 `~/.gemini/config/rules/askhuman-antigravity.md`，有效always_on、description与本家托管标记；
无CLI-only语义guard、不改共享AGENTS/GEMINI、不用lazy Skill。
prompts.rs生成共同ask/show_last/whats_next/todo纪律、等待回答、child例外及结束标记；只描述所选transport。
CLI借宿主可用命令执行工具，未来MCP借MCP工具；不能将Shell/Bash24h要求用于Antigravity。

agy1.2.9的条件指导：run_command启动一次；后台manage_task status续取同一原TaskId；完整stdout；不重启/伪答/提前交接。
不得把CLI工具名/TaskId/参数/超时当IDE/扩展/2.0已验证契约。缺所选调用工具时报告集成不可用，不编造接口。
WaitMsBeforeAsync不是答复期限，RunPersistent效果待测。模块<24000bytes，不内联外部文件。
同scope顺序未知，doctor只报告可读确证、不称规则最高权威。

## 独占文件事务：冻结的worker接口

新增 `integrations/antigravity_rule.rs`，仅处理固定模块和本家ownership；不决定family/transport/能力。
全部现有Rust及mod glue归I-002，新增模块归I-003；以下合同在开工前冻结，不用任务中间未完成里程碑作DAG依赖。

- `RuleContext` 包含固定target_path、integration_state_file路径、generation_version和完整rendered_bytes/variant；生产由paths/prompts组装，不接受CLI任意目标路径。测试注入隔离home。
- `probe(&RuleContext) -> anyhow::Result<RuleSnapshot>`：Missing、OwnedCurrent、OwnedOutdated、Conflict/NeedsRepair状态；只NotFound可认作Missing，I/O/解析错误返回Err。snapshot包含已核对hash/identity/variant。旧合法版本与ownership一致才能OwnedOutdated，用户漂移不等于可更新。
- `install_unlocked(&RuleContext) -> anyhow::Result<RuleSnapshot>` 与 `uninstall_unlocked(&RuleContext) -> anyhow::Result<()>`：调用者持有现有IntegrationMutationLock，底层不再次取锁。旧公共入口均经持锁wrapper或agent_mode路径调用，避免裸写/重入锁。
- 公共status/doctor先用严格probe并传播Err；兼容bool辅助值不能将该错误转为未安装。current mode依据已核对artifact variant推导，不能仅凭文件名/标记/未知正文猜MCP。
- ownership沿用integration-state.json，新增 `antigravity.rule`：固定path、generation_version、variant、sha256、platform_file_identity；保留codex与未知字段。解析失败中止，不把账本读成空。
- 完整模板格式：YAML always_on/description + 独占Antigravity managed-file标记(version=1) + 现有BLOCK_BEGIN/END包裹所选协议；标记只是格式检查，完整hash+identity+本家账本共同证明归属。

### 预检、写入和回滚

现有mutation_lock覆盖路径预检、文件/ownership变更及模式结果；它只串行共享该锁位置的AskHuman写者，不能约束外部/独立Dev实例。
逐级真实祖先目录检查和句柄绑定，拒绝symlink/Windows reparse/逃逸；目标本用户普通文件。
未知同名文件即使相同bytes也不认领，read/stat/类型/账本错误均fail-closed。
更新/删除前核对原identity/hash/格式；任何用户追加、frontmatter编辑或同字节换inode均漂移拒绝。

缺目录只沿固定 `.gemini/config/rules` 路径逐级创建并记录本轮identity；Unix0700，Windows需真实ACL/访问验证，不能把mode当ACL证明。
新装独占创建；更新同目录临时文件+原子替换，替换前重新检查原文件/父目录。平台实现验证句柄/路径替换保护。
最后stat至rename/unlink的同用户竞争窗口仍存在，明确实现限制，不宣称hash等同条件原子替换。

持久化本家pending事务journal到AskHuman私有状态目录；每次操作使用唯一transaction_id，记录固定目标、操作、
前后ownership/byteshash/identity、variant、本轮目录，以及prepared/file-applied/ledger-applied/committed阶段。
在任何变更前，独占保存旧受管文件与完整旧账本的durable preimage（或明确原先不存在），journal引用其路径、hash与identity；
Unix文件0600、目录0700，Windows核对私有ACL；先同步备份和journal再改目标，保证崩溃后仍可恢复实际旧bytes，不能以hash代替备份。
新文件在原子替换前记录其临时文件identity/hash，替换后核对并记录最终identity；阶段落后时按固定目标实际identity/hash与已记录临时文件identity核对，
只接受明确的旧态或本事务新态。无法区分或存在外部漂移时保留证据、报NeedsRepair，不猜测或自动覆盖。
模式由验证后的产物推导；文件和账本提交并同步后才清pending与本事务私有备份。
异常/崩溃恢复仅在实际目标与预期旧态/本事务新态一致时，使用已核验preimage恢复文件及完整账本；
恢复后重核identity并同步更新所有权，不能沿用已替换文件的旧inode。恢复自身再次中断须可重入，漂移保留证据，不称ready。
更新/卸载分别在备份、prepared、目标替换或删除、账本写入、commit及清理之间注入失败/终止；验证恢复旧bytes/其它家族账本、
幂等重入、新identity、阶段落后和漂移拒绝。备份只含已核验受管模块与集成账本，不记录凭据或会话。
只回滚本轮创建且identity未变、仍为空的目录；不递归删，不处理其它实例journal。
成功安装后的共享目录（含产品创建目录）在None时保留；原有与诊断保留目录也不删。

重复同版本CLI安装no-op；合法老版本更新；None对Missing幂等清本家过期记录，对漂移文件拒绝删除，保留偏好/其它家族。
无已发布CLI-only产物需迁移；旧诊断目录/旧路径同名文件不自动收养/搬移/删，报告未迁移。
P0不装MCP/Hook。未来MCP最小托管mcpServers.askhuman，非本家同名拒绝，不加未知timeout或默认权限；切换只清已验证本家产物。

## P1/P2适配边界

独立antigravity_hook.rs按实际宿主/version normalize，不改其它家族parser。
Hook路径、命名key、工具事件、timeout、跨客户端范围逐项实测；共享规则不意味着Hook也共享安装。
conversationId来源验证后才内部绑定，workspacePaths多根沿现有归属；未知pid/child不造，日志root防逃逸。
MCP一次性binding需证明事件/参数传播，不让模型提供可猜session参数。无可靠compact触发/幂等依据，不每轮注入问答。
Stop默认关；主来源可信、自然停止且fullyIdle才发卡，continue原生同UUID，不能以外部resume代替。
child不可靠不开放Stop/interject/permission；错误/基础设施失败保留原生行为，不发通用allow。
插话ephemeralMessage须真实消费回执；PreToolUse allow不等PermissionRequest；无原生答案接口不接管ask_question。
权限/fork独立规格；后续host eligibility未知时高级adapter拒绝，不从CLI单测放行全家。

## P0 实施DAG与文件责任（历史合同，已执行）

| ID / wave | 唯一owner | Depends | 验收/报告 |
| --- | --- | --- | --- |
| I-001 / 1 | ops；现有probe模块/fixtures与证据 | 实施授权与本文冻结 | 剩余原TaskId/取消/真实等待诊断，保持blocked；I-001-ops |
| I-003 / 1 | implementer；仅新antigravity_rule.rs及其文件内测试 | 本文冻结的RuleContext/函数合同 | 文件事务/账本/journal/ownership；不改mod.rs，先交付供I-002链接；I-003-implementer |
| I-004 / 1 | implementer；TS types/settings/UI/i18n/tests | 本文冻结的既有status shape/家族/mode合同 | Antigravity标签、None/CLI、无MCP/高级、读取错误可见、缺agy不禁家族安装；I-004-implementer |
| I-002 / 2 | implementer；全部现有Rust，含paths/commands/agents/cli/prompts/agent_mode/mcp_config/mod glue/launcher/doctor/show_last及必要dedup | I-003完成 | 链接newmodule并运行其测试，身份/模式/状态/兼容；I-002-implementer |
| I-005 / 3 | ops；新装产品的真实fixtures与记录 | I-001/I-002/I-003/I-004完成，产品预检通过 | P0完整ask/交接/todo/恢复失败/长等/取消/并发/安装更新None；I-005-ops |
| I-006 / 4 | ops；其它host/platform记录 | 对应产品实现与可用真机 | 各tuple独立验收，缺条件明确未验证；I-006-ops |
| I-007 / 4 | implementer；wiki与局部specs | I-005，已取得I-006结果 | 证据对应发布范围；I-007-implementer |

I-003新模块完成后才由I-002接mod并统一cargo验收，未链接文件不宣称测试通过。
I-001/I-003/I-004文件不交叉可并行；全部现有Rust单一owner，避免exhaustive match互踩。
任务仅在deps完整完成后ready，不把“另一任务中途冻结”作隐式依赖。总并发≤4含协调者，reviewer生命周期不进DAG。
P1/P2需独立证据/规格后追加bounded tasks，不先算P0实现。

## 验证矩阵

每tuple记录宿主/hostversion、OS、AskHumanversion、调用方式/能力、命令退出、回答/取消时序、日期与证据。
每项分别passed/failed/blocked/not-tested；不导出真实session/凭据，准备命令非live证明。

| 宿主 | 本机 NixOS 主线 | macOS 后置 X | Windows 后置 X |
| --- | --- | --- | --- |
| agy CLI 1.2.9 | P0、原任务、RequestReview、实际取消及真实30/120/600秒结构化输出已验证；N0–N4按gate补齐，旧自动gate仍blocked | 真机逐能力未验证；可独立提前介入 | 同左；严格writer也未实现 |
| Antigravity IDE | 本机可运行时纳入独立宿主任务，加载/调用/等待/取消/身份未验证；缺客户端blocked | 独立adapter/真机未验证 | 同左 |
| IDE/VSCode扩展 | 同左，不能以通用extensionHost确认来源 | 独立adapter/真机未验证 | 同左 |
| Antigravity 2.0 本地客户端 | 先明确本机客户端/version，再纳入独立任务；当前未验证 | 独立adapter/真机未验证 | 同左 |


| 能力/需求 | 确切检查与标准 | Responsible |
| --- | --- | --- |
| 载体发现 | 独立标记before/present/after，所有tool attempts及opaque schema均阻断；最终产品模块另测 | 既有carrier证据；I-005/I-006 |
| FR-003/NFR-001事务 | 新装/重复/更新/None、未知同名、read/stat/账本错误、symlink/reparse、祖先替换、hash/identity漂移、缺目录、journal/失败/崩溃/回滚漂移、共享目录保留 | I-003；I-002链接后cargo验证 |
| FR-003/NFR-003身份 | hint解析/冲突、UUID非权威、已知未绑定show_last拒绝、无hint识别边界、普通非Agent兼容 | I-002；I-005/I-006实测 |
| NFR-003并发 | 同项目两个未绑定同题各收自己回答，不合流/重放/读其它历史 | I-005/I-006 |
| FR-003基础 | 真实ask、show_last安全失败再确认、whats_next下一任务/批准结束、todo、附件/stdout完整性 | I-005/I-006 |
| NFR-002等待取消 | 真实30/120/600秒、一次启动、可证明原任务续取、人类答复、取消ack/本请求进程结束 | I-001/I-005/I-006 |
| FR-004 MCP | 六工具/图片感知/600秒/交互取消/EOF/drain/并发binding；180秒问题解决前不放行 | 后续P1 |
| FR-005生命周期/Stop | 来源/child/原因/后台未完/不重复卡/自然continue同UUID | 后续P1 |
| FR-006高级 | unsupported一致，未测host不进adapter，不造permission/fork | I-002/I-004 |
| NFR-005回归 | Rust tests/fmt/clippy、pnpm test/build、旧记录与原家族、status错误/IDE-only无agy | I-002/I-004 |

产品命令：cargo test --manifest-path src-tauri/Cargo.toml；cargo fmt --manifest-path src-tauri/Cargo.toml --check；
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings；pnpm test；pnpm build。
事务/身份自动测试隔离home/ledger，不碰真实全局文件。CLI probe `node scripts/antigravity-probe.mjs --suite cli,mcp --delay-seconds 30,120,600`；
hooks/sessions/rules suite仅对应诊断，不当产品passed。
Linux/macOS install.sh，Windows scripts/install-windows.cmd，然后使用新装AskHuman做GUI/CLI、空格/Unicode/quoting/focus。
IM用独占测试preset，不用生产bot；缺条件保留未验证。NixOS sandbox限制与宿主缺陷分别记录。

## 文档与发布

P0 实施更新 spec/plan/PROGRESS、overview 中新增严格模块的地图行、双语 wiki 与 README 集成入口；五家高级会话/lifecycle 范围未变。
完成durable工作删PROGRESS对应项，历史留git。当前不提交/发布/永久安装/升级。
发布仅宣称实际通过tuple；inherited always-proceed blocked不能由载体观察/计划修订解除。

## P0 执行记录（2026-10-04）

执行 run `antigravity-p0`，同一 popup-only worktree；原生线程额度不足时由协调者接手非 writer 任务，独立 reviewer 保留。
已实现家族/None/CLI、共享模板、strict ownership/journal、设置读取错误、CLI/doctor 和高级能力关闭。
Windows 严格 writer 仍是明确 unsupported 的 ACL/identity gate；聚合 doctor/show/update 和协作风格更新排除该不可用 adapter，专用 Antigravity 查询保持报错。
Unix 聚合更新在过滤目标前严格读取 Antigravity snapshot；冲突、pending 和账本错误不会被吞成 None。
聚合 cleanup 复用同一平台清单，包含受支持的 Antigravity 并沿用严格 None 与错误汇总；未执行会删除其它个人集成的真实全局 cleanup。

自动验证：1190 Rust tests passed / 2 ignored / 1 unrelated personal Cursor DB smoke filtered；fmt、all-targets Clippy、184 Vitest + 2 Node tests、前端 build、最终 install.sh 通过。
最终新装 AskHuman 验证：未绑定恢复拒绝、跨家族冲突、12 项严格错误路径、共享规则 CLI/no-op update/None 幂等与目录保留。
原有 Codex 提示正文 SHA256 与安装前相同。待办采用经过校验的显式家族提示，附件/列表/删除验证通过。
两个同题未绑定真实 Popup 请求分别得到 A/B，两个独立历史 ID，无 session；下一任务选择、实际合成任务执行及结束测试通过。
这些是已安装 CLI 接口证据，不冒充 agy 异步任务认证。

agy 1.2.9/NixOS 真实附件提问、恢复失败后重新确认、测试交接均返回实际答案；两条历史归属 Antigravity，无借用外层 UUID。
原生最终文本含实际回复标记，但未满足严格精确输出；原始结构化 TaskId 未证明，same-task status/cancel gate 不放行。
正式规则 before/present/after 字段观察为 UNKNOWN/antigravity/UNKNOWN，各零工具调用；这不是独立随机 nonce 的认证。
三个原生加载观察及真实调用都因 inherited always-proceed 保留 blocked。
按原批准前提，缺原始 TaskId 时未追加真实30/120/600秒与取消测试；旧延迟夹具不代替实际产品验收。

人类批准临时正式规则；安装 bytes 与审批附件完全一致（4390 bytes）。重复安装和更新 bytes/inode/账本均未变化。
验收后 None 两次均成功，固定文件与本家 rule 记录/pending 均移除，共享目录 identity/mode 保留；当前模式 None。
当前未提交、发布或升级。MCP、可信绑定、生命周期/Stop、插话/Watch、权限、新建/Fork 继续关闭。
macOS、Windows及 IDE/扩展/2.0 无实机证据；Windows 还需严格 writer 实现与 native 校验。
完整 native 长等待、取消、输出纪律、无 hint ancestry、child 和 IM 仍为未完成验收。

执行证据：`.agent-team/runs/antigravity-p0/` 的 reports、installed-cli-checks、installed-state-regressions、
production-idempotence/cleanup、carrier-*、native-product、cli-functional、todo-hint-final-checks 和 final-install。
本记录陈述已执行检查与未通过 gate，不能据本地代码交付宣称所有 P0 客户端已认证。
独立 C2 审查 approved，全部代码 finding 已关闭；人类接收实验性本地交付，明确将 agy gate、Windows writer 和跨客户端实机验收延期，并已记录为三项项目待办。

## native追加验收（2026-10-04至05）

Run `antigravity-native-acceptance`，同一popup-only worktree；17Node/8Python与install通过。
人类接受自有fixture完整原生记录的严格原始TaskId依据，非产品binding。RequestReview下
固定helper无引号形式获准、另一同Node准确负例拒绝；带单引号形式在本组literalrule下被拒绝。
其它agy实例已由人类退出并ps核对；用户级settings没有worktree隔离，须避免其它实例写回干扰。

真实取消一启动/一kill/同TaskIdstatusCANCELED；daemon先于native最终回复finalized，进程结束。
helper无自身优雅signal回调，人类接受原生/daemon外部取消依据，不伪造callback。
真实短回答等待9360ms、唯一popup历史familyantigravity/session未绑定；回答后随机nonce的
127-byte完整stdout经原TaskIdstatus读取，与官方result.structured_output.output逐字一致。
人类接受结构化字段认证，自由文本result.response聚合背景提示仍不精确。

原始完整settings字节与0600已恢复，临时allow移除；模式仍None，不永久安装/升级/发布/提交。
配置规范化导致普通restore拒绝漂移；人类明确审核特定恢复，只有相同已审核bytes及其它字段
未变化时复用恢复差异，新的漂移拒绝。C2是预检approved，不是后来手工对照第三轮独立审查。
真实30/120/600后续已完成，实际61.930/138.807/627.596秒，原任务续取与结构化字段精确通过，见[长等待验收](../investigations/antigravity-cli-real-waits.md)；完整native/P1/其它host/platform仍未完成。
具体证据与限制见[追加调查](../investigations/antigravity-cli-native-acceptance.md)。

## 历史回填：被当前共享方案替代的设计状态

以下保留当时测量、批准与诊断。“共享未批准”“CLI-only gate阻塞”“askhuman-antigravity-cli.md候选”仅为当时状态；
活跃要求以上述家族共享合同为准，负面观察与精确根因未知仍有效。

## 第一波实测回填（2026-10-04）

> 历史记录：当时载体批准状态已由上方家族共享合同替代；测量结果保留。

环境与稳定探针已交付，详见 [实测报告](../investigations/antigravity-cli-probes.md)。CLI 600秒完成；MCP 600秒请求约180秒后被客户端取消。`--new-project` 对照实际加载局部 MCP/Hook/根规则，而 init.cwd 本身不能证明配置发现。CLI 原生会话 env UUID 与 init/Hook 一致，同时存在外层 Codex env。

这些观察不改变已批准的 CLI-first 顺位、跨家族冲突拒绝与无绑定历史 fail-closed 合同。实施前需确认配置发现/全局规则载体；P1 MCP 长等待保持关闭，不臆造 timeout 字段。当前 inherited always-proceed 导致所有自动 gate blocked；观察不构成权限或产品放行。

## CLI-first 前置验证回填（2026-10-04）

> 历史记录：当时载体批准状态已由上方家族共享合同替代；测量结果保留。

详见 [前置实测](../investigations/antigravity-cli-preflight.md)。真实未绑定 CLI Popup 问答通过，等待约9秒，不能计作30秒或长等待验收。测试隔离继承的外层 Codex/MCP 标记，不改变产品冲突裁决。

CLI-only 全局 always_on 候选存在时未加载；同字节本地规则零工具对照通过。P0 载体 gate 仍 blocked：先定位发现条件或确认替代载体，不能直接按文件存在开启安装。共享规则替代方案尚未批准。获授权创建的 rules 目录保留，唯一临时规则文件已按所有权校验删除；未修改产品实现或全局权限。

## 全局载体诊断候选（2026-10-04）

> 历史记录：当时载体批准状态已由上方家族共享合同替代；测量结果保留。

[替代载体报告](../investigations/antigravity-cli-carrier.md)：共享config/rules独占临时模块before/present/after严格零工具对照通过，CLI-only目录遗漏的精确原因仍未知。可审查方案是共享 `askhuman-antigravity-cli.md`，须明确其它客户端也可能加载、语义guard不提供技术隔离、读取失败fail-closed及所有权/重复安装/卸载/漂移保护。

此方案尚未采纳，不替换本计划批准的CLI-only载体，也不自动解除I-003入口gate。先由人类决定是否接受表面变化并修订计划，或继续调查CLI专属路径/插件；身份冲突拒绝合同保持不变。
