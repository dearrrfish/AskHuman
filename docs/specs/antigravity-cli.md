# Antigravity 家族支持需求

> 2026-10-17 v1 范围调整：以[本机 NixOS CLI v1 边界](antigravity-v1.md)的已确认决定为准。仅 agy CLI、基础 CLI + 可信 MCP、Popup 必验；IDE、其它平台、IM验收与N2–N4高级能力不阻塞v1。本文完整路线/历史证据保留，不再整体作为v1收敛依赖；来源、配置、运输安全门槛保留。V1已确认单canonical项目范围（多项目歧义拒绝），exact CID show_last必需，自动compact恢复延后。

初始 run：`antigravity-support`；家族范围修订：`antigravity-family-plan`。
2026-10-04 人类批准按家族共享协议、按客户端分别验收能力。2026-10-04 人类随后授权执行 P0；本地实现与证据见实施计划的 P0 执行记录，未通过的客户端能力仍单独保留 gate。

## 用户与场景

目标是使用本地官方 Antigravity 客户端的开发者：agy CLI、Antigravity IDE、IDE/VSCode 扩展与官方 Antigravity 2.0。
希望复用 Popup/IM、等待回答、结束交接和待办；每个客户端能力分别验证。
None/CLI/MCP 指 AskHuman 调用方式，不指宿主 TUI/IDE。P0 优先 CLI 调用方式及 agy 验证。
当前 native 证据来自 agy 1.2.9/NixOS及[1.2.16有限恢复实测](../investigations/antigravity-cli-n0-resume.md)，两版本结论不自动互认；[本机库存](../investigations/antigravity-local-inventory.md)已确认IDE安装，其原生能力及真实macOS/Windows单列。
2026-10-05 人类批准[本机 NixOS 优先顺位](../plans/antigravity-nixos-first.md)：先补完整本机能力，其它平台专用实施与真机验证后移，并保留独立介入入口。已批准功能与安全 gate 不因顺位变化而缩减；本轮只改计划。

## 功能需求

| ID | 需求 | 验收证据 |
| --- | --- | --- |
| FR-001 | 官方工具、MCP、agy、规则与优先级调查 | 来源、日期、版本、宿主与已知/未知分级 |
| FR-002 | 对照 Codex/Claude 排顺位 | P0/P1/P2、逐项 gate 与未开放原因 |
| FR-003 | antigravity 家族与 None/CLI | 配置/CLI/UI round-trip；共享模块幂等安装更新卸载；实际调用 |
| FR-004 | MCP 六工具、图片和长等待分项开放 | tools/list/call、取消、并发、附件、daemon drain |
| FR-005 | 生命周期、绑定、恢复与 Stop 分项开放 | 合法事件、主/child归属、同会话 continuation |
| FR-006 | 高级能力边界明确 | unsupported一致，无虚构permission/fork/恢复 |

## 非功能需求

| ID | 约束 | 验收证据 |
| --- | --- | --- |
| NFR-001 | 不覆写用户或其它家族文件、不放宽权限 | 读取错误、所有权、symlink/reparse、漂移、事务回滚 |
| NFR-002 | 等待不能当成已答；取消有闭环 | 30/120/600秒、真实回答、取消确认、本请求进程清理 |
| NFR-003 | 不串会话、不把child当主代理 | 同项目并发、缺绑定、外来标记与child |
| NFR-004 | 宿主×OS×调用方式×能力单独记录 | 本地NixOS；其它客户端/macOS/Windows真机 |
| NFR-005 | 现有 Codex/Claude/普通非Agent兼容 | Rust/TS回归、旧记录、普通Project查询不变 |
| NFR-006 | 本机优先且保留平台介入空间 | 共享schema/平台adapter边界、独立ownership/tuple；外部平台不作本机默认依赖，未测宿主不开放 |

## 家族与调用合同

持久化名称 `antigravity`，显示名 `Antigravity`，不以 Gemini CLI 为别名。
`agy` 只用于 CLI 宿主检查/未来启动；其缺失不能阻止 IDE 用户安装共享协议。
“已安装”仅表示受管文件状态，不代表所有宿主已加载或已验收。P0提供None/CLI；
MCP/lifecycle/stop/permission/interject/watch/transcript/launch/fork在自己gate通过前保持不支持。

CLI 每条受管调用携带 `AskHuman --caller-agent antigravity <原有参数>`，包括帮助、恢复、交接和todo。
全局parser在dispatch前校验unknown/重复/冲突hint；hint只声明家族，不提供session、权限或信任。
与继承其它家族env冲突时拒绝，不能借外层UUID，也不静默改为原生Antigravity标记优先。
既有无hint和其它家族调用兼容。

已知Antigravity但无可信session不得降为Project scope；show_last明确拒绝恢复，不展示同项目其它历史，
协议引导重新向人类确认。无session/MCPinstance的请求不按project/同题合流或重放，不造UUID。
无hint直接agy调用的恢复入口需验证process-tree家族识别；ancestry不能提供session权威。
IDE也仅在有hint/env或经验证宿主识别时确认家族，不能将通用Code.exe/extensionHost认作Antigravity。
没有任何识别依据时不能声称IDE安全恢复已支持；普通非Agent CLI的既有Project查询保持。
P1可信binding后才精确恢复；弱MCPinstance+project键须标明限制，不通过环境UUID/最近日志猜绑定。

## 共享协议

正式独占模块 `~/.gemini/config/rules/askhuman-antigravity.md`，有效 `trigger: always_on`、
简短description与本家托管标记。不加CLI-only语义guard、不改共享AGENTS/GEMINI、不用lazy Skill替代。
共享ask/show_last/whats_next/todo、等待回答、subagent例外与结束标记；渲染仅描述所选CLI或MCP调用方式。
命令执行/续取/取消须符合宿主实际工具契约，不硬编码Shell/Bash或24h工具timeout。
agy1.2.9可用run_command/manage_task指导：只启动一次、续取原TaskId、完整读取stdout、不重发/提前结束。
该指导是条件性的，不假定IDE/扩展/2.0的工具名/TaskId/参数相同；缺所选工具时报告不可用，不造结果。
WaitMsBeforeAsync不是人类答复期限；RunPersistent效果仍待验证。规则少于24000bytes、不展开外部文件；
doctor不宣称规则最高优先级。同scope冲突顺序未知。

## 文件事务与失败行为

独立 `integrations/antigravity_rule.rs`，不复用把读取失败当空文件的writer。
操作受现有mutation_lock保护，区分NotFound与其它错误，验证真实祖先/普通文件，拒绝symlink/reparse/逃逸。
标记不是整文件所有权证明：需要持久化本家记录、完整byteshash与文件identity；未知同名文件即使内容相同也不收养。
更新/卸载遇hash、格式、identity漂移保留并报错；status/doctor读取错误不能显示成未安装。
缺目录只沿固定路径逐级建立并记录本轮身份；失败回滚仅处理本轮且未漂移的文件/空目录。
成功安装后的共享目录以及既有/诊断目录在None时保留，绝不递归清理。
变更前持久化私有旧文件/完整账本preimage与transaction_id/阶段journal；hash不能代替可恢复bytes。
崩溃后按旧态/本事务新态identity/hash核对，阶段落后、材料缺失或漂移不能猜测覆盖；保留证据并报NeedsRepair。
恢复旧内容后重新登记文件identity；恢复可重入，提交同步后才清本事务备份。更新/卸载需覆盖各阶段中断测试。
规则、ownership及模式事务幂等；崩溃/回滚漂移保留证据并报需修复，不伪称ready。
无已发布CLI-only产物需迁移；旧路径/同名诊断文件不自动移动、删除或认领。
MCP后续只改已确认本家最小条目，不添未知timeout或默认permission allow；P0不装MCP/Hook。

## 证据与非目标

共享临时模块在agy1.2.9/NixOS的严格零工具before/present/after加载/清理已观察通过，足以选择共享路径。
它不是最终产品文件；CLI-only遗漏精确原因未知，保留调查但不再阻塞当前载体选择。
真实未绑定Popup约9秒问答通过，不证明产品归属/长等待/取消/恢复/并发或其它功能。
早期native自动gate因inherited always-proceed为blocked，不能据此产品放行。
追加[本机验收](../investigations/antigravity-cli-native-acceptance.md)已取得agy1.2.9/NixOS的
严格原始TaskId诊断、RequestReview准确正负对照、同任务实际取消（原生CANCELED与daemon提前finalized）
及真实短回答的structured_output.output精确证据，人类已接受对应范围。
原生记录仅作自有fixture诊断，不赋予产品binding；helper无优雅信号回调，不能造回执。
自由文本result.response聚合背景提示，不认证其整段精确；[真实30/120/600秒矩阵](../investigations/antigravity-cli-real-waits.md)已通过，完整native仍未放行。
首版推荐CLI但用户选择；未验证tuple明确未验证，不能从安装或另一宿主结果推导通过。
本轮不永久安装/升级/发布；临时正式规则已获授权验收并清理。首版不承诺permission memory、原生ask_question接管、Fork、Remote Control、browser/media/schedule专用适配。
身份冲突合同改变需另行决定。

## 来源

[原始调查](../investigations/antigravity-cli.md)、[第一波](../investigations/antigravity-cli-probes.md)、
[前置验证](../investigations/antigravity-cli-preflight.md)、[共享诊断](../investigations/antigravity-cli-carrier.md)、
[实施计划](../plans/antigravity-cli.md)。

## N1 子合同

[来源/绑定与MCP合同](antigravity-n1-source-binding.md)及[执行子计划](../plans/antigravity-n1.md)冻结接口、所有权和前置探针；实际1.2.16/IDE的Hook传播、child来源、MCP配置路径/优先级仍blocked，不代表产品实施就绪或能力放行。
