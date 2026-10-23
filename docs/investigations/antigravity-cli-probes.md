# Antigravity CLI 第一波实测

日期：2026-10-04；NixOS Linux x64，agy 1.2.9，Node 24.21.0，worktree AskHuman 0.13.2。
这是协议探针结果，不是产品支持验收。调查与优先级见 [调查](antigravity-cli.md)、
[规格](../specs/antigravity-cli.md)、[计划](../plans/antigravity-cli.md)。

## 环境、授权与证据边界

人类批准临时 Nix 构建环境、worktree 安装，以及真实 agy 使用现有登录、模型额度和必要用户运行状态。
实际会话继承 `always-proceed`，单次 `--sandbox` 未改变报告的权限模式。人类随后明确批准在该模式下
继续协议观察，保留所有自动 `blocked`，不得用于权限安全或产品集成放行。

没有修改全局 MCP、规则或权限配置，没有加入放行规则，没有使用危险跳过权限参数。
`--new-project` 对照会创建原生项目运行状态；这些临时项目和私有输出保留，没有自动清理。
原始 stdout/stderr 留在忽略的 `.askhuman-dev/probes/<fixture UUID>/raw`，不提交、不作为附件。
以下 UUID 均为本地夹具目录名；不包含原生会话 UUID 或凭据。身份值仅在内存比较或加盐散列。

## 实测结论

| 项目 | 观察 | 仍未放行的原因 |
| --- | --- | --- |
| 基本会话 | 登录有效，预期标记返回，init 提供 UUID 和 57 个工具 | 权限模式；不证明 AskHuman 会话绑定 |
| CLI 30/120/600 秒 | 每次仅启动一个夹具，分别真实等待 30000/120000/600001ms 后完成；600 秒会话正常退出 | 可见 run_command DONE 输出为最终 stdout，没有原始结构化 TaskId；无法自动证明任务续取绑定 |
| CLI 续取 | 30 秒有 13 次 manage_task status，使用一个 ID，状态输出和最终回复含完整标记 | 原始任务 ID 不在捕获的 run_command 输出，不能补造其来源 |
| CLI RunPersistent 对照 | 30 秒完成 | 请求的 RunPersistent、WaitMsBeforeAsync、Cwd 未出现在可见参数记录；不证明其生效 |
| CLI 取消 | 启动一次，未完成；原生会话退出 | 没有夹具 SIGINT/SIGTERM 取消确认，不能据退出推断完整取消闭环 |
| 局部 MCP | 初始仅 cwd 的测试未启动服务器；`--new-project` 对照启动了服务器并完成 30/120 秒调用 | init.cwd 相同不足以证明原生项目配置已加载 |
| MCP 600 秒 | 发起一次，179999ms 后收到 notifications/cancelled，未完成；原生最终 result 仍为 SUCCESS | 真实客户端约 180 秒取消，长等待失败；不是夹具或 runner 超时 |
| MCP 断开 | 确认请求启动后终止仅本次 agy 进程组；服务器记录 SIGTERM 取消，没有完成 | 仅进程断开证据，不等同原生交互取消 |
| MCP 图片 | 30 秒后真实发送 text 和有效 PNG image | 原生展示/视觉理解仍需独立验收 |
| 根/子目录规则 | 新项目 root 返回 ROOT_PROBE_MARKER；child cwd 且匹配子规则条件返回 CHILD_PROBE_MARKER；均零工具调用 | 文件读取时动态 scope、同 scope 顺序及 CLI 专属全局载体尚未验证 |
| Hook | 新项目触发 Pre/PostInvocation、PostToolUse、Stop；非继续 Stop 可正常结束 | continue、长超时、错误/后台任务等语义未测 |
| child Hook | stream 提供一个 child UUID；Hook 中两个会话散列与 parent/child 对应，无未知第三会话 | 这是相关性观察；没有可信注册、权限或 AskHuman child 归属验收 |
| CLI 调用身份 | ANTIGRAVITY_CONVERSATION_ID 是 UUID，等于 init 和 Hook 会话；夹具 cwd 正确 | 同时继承外层 Codex 环境；来源裁决与冲突策略仍需实施/审查 |

原生工具清单见 [nixos-1.2.9-tools.json](../fixtures/antigravity/nixos-1.2.9-tools.json)。
工具出现不代表特定参数、角色权限或能力已经验收。

## 配置发现与目录规则

默认 harness 为每次测试创建真实嵌套 Git 仓库和 `.agents` 配置。
初始 cwd-only MCP 测试没有服务器启动或调用；`agy mcp list` 也未列出夹具。
新项目对照使用同类局部配置，加 `--new-project` 后完成 MCP 调用并触发 Hook。
因此只观察 init.cwd 不足以判断项目选择/配置发现；不能把初始未发现配置当成 stdio 不兼容。
因果仍限于这台机器与其已有项目状态，不能推断所有版本均必须加该参数。

root 对照禁止工具调用和读取规则文件，排除了模型自行搜索标记的解释。
初始 root 请求虽然最终标记正确，却发生了配置搜索，不能作为自动加载证明。
child 文件读取对照返回 root 标记，工具参数未直接匹配预期路径；不能据此认定目录优先级失效。
另一次 child cwd 对照使用了与子规则条件不一致的请求，不纳入有效证据。纠正触发条件后，在 child cwd、新项目、零工具调用的对照中得到 CHILD_PROBE_MARKER，确认该路径的目录优先级观察。

## 身份与生命周期

CLI 夹具只记录环境变量名。明确观察到 ANTIGRAVITY_CONVERSATION_ID、ANTIGRAVITY_PROJECT_ID、
ANTIGRAVITY_AGENT 等原生变量，同时存在继承的 CODEX_SESSION_ID / CODEX_THREAD_ID。
额外身份对照只读取已知会话 UUID并做加盐比较，确认它与 init/Hook 一致；没有读取或导出凭据值。
这为下一波提供了实际输入，不能直接将任何环境变量当成可信权限来源。
原批准计划的跨家族冲突拒绝策略保持不变，改变该行为须另行确认。

MCP tools/call 的 `_meta` 实际包含 progressToken 和两个匿名字符串字段；夹具不保留其名称或值，
尚不能判断其是否包含会话身份。不能声称 MCP 没有 UUID，也不能声称已有可信绑定。

child 对照中父会话 Stop 曾 fullyIdle=false，最终为 true；child Stop 为 true。
terminationReason 未落入夹具已知枚举，所以该字段仍未知；不把 null 解释为没有停止原因。
没有原生 parent/child 权威归属验证，也未调用真实 AskHuman 的 child 忽略逻辑。

## 复现与留存

公共 harness / 离线测试见 [README](../fixtures/antigravity/README.md)。
原始 native 命令清单位于各夹具目录的 commands.json；新项目对照在已审查 live() runner 上加
`--new-project`。MCP 对照禁止配置搜索；root 对照禁止工具；child 对照仅请求一个不调用工具、
不使用 AskHuman、不再委派的子代理。控制脚本保存在忽略的 `.askhuman-dev`，不是产品入口。

| 证据 | 夹具目录 UUID |
| --- | --- |
| expanded offline selftest | eecae084-6831-4c67-808c-ab4fbb700c79 |
| CLI 30 / 120 / 600 | 62ffce3e-4df9-47cb-ad38-691f58cfb68a / 31f37bc3-860c-4b29-9567-55574f0adb84 / d2c46e6f-27e9-4321-bf95-caa5b71f2510 |
| MCP new-project 30 / 120 / 600 | 82f7c09d-2f76-4269-bc62-6d8fd782ea58 / 6cedb7c8-be6d-4b78-a56e-43012e8c2f66 / 461b072c-f002-4004-9a3a-176d28ae549f |
| MCP disconnect / image | 74dd9b2c-551b-476c-b020-f50525d820d2 / a096b277-d472-460f-885b-b3dce669e3b8 |
| root rules / child Hook / identity | 17a49ebe-7cc3-4b59-b675-eb16331ab674 / 883afe64-7f04-40f4-a0e4-47c348b47d47 / fefdc742-e711-4fb2-bdf3-fe61f2547736 |

有效 child cwd 规则对照：e6f8b8cf-1e07-4bcc-8878-4725dfb1ad53。

## 对后续实施的影响

CLI-first 顺位维持。CLI 已观察到 600 秒等待，但仍须用真实 AskHuman 验证交互、取消和任务身份。
MCP 不宜承诺无限等待；先查清约 180 秒客户端取消的可配置性或有正式契约的延长方式，
不得添加文档未定义的 timeout 字段。实际配置发现必须验证，不能只检查文件存在。
全局 CLI 专属规则载体、同 scope 顺序、同项目并发与精确 resume、六个真实 AskHuman MCP 工具、
附件/daemon drain、Stop continue/长 Hook、原生 macOS/Windows 仍为独立 gate。

构建与验证：经批准的 Nix shell 内 `./scripts/install.sh` 成功；新安装 AskHuman 的 version、
agent-help、dev status 成功。六项 expanded offline selftest 通过；独立预检两轮，最终通过，
范围为稳定 harness。上述 native 观察由协调者执行，不把预检批准扩张成产品集成批准。

官方基线：[Headless](https://www.agy.dev/docs/cli/headless/)、
[MCP](https://www.agy.dev/docs/mcp/)、[Hooks](https://antigravity.google/docs/hooks)、
[Rules](https://www.agy.dev/docs/rules/)。实测差异以上述本机捕获为准。

后续 CLI-first 全局载体、身份冲突与真实 Popup 验证见 [前置实测](antigravity-cli-preflight.md)。
