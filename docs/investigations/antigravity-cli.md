# Antigravity CLI 支持调查

调查日期：2026-10-04；对象为独立 Antigravity CLI `agy`，不将 Gemini CLI、IDE launcher 或 SDK 当作可替换接口。
本机 `agy --version` 返回 **1.2.9**。在线文档是当前版本说明，不能据此认定本机每个接口均可用。

## 证据等级

- 官方契约：下列官方文档在线读取成功；实际差异见 [第一波实测](antigravity-cli-probes.md)，不能用文档代替 E2E。
- 本机观察：help/version 成功；第一波已有真实登录会话、CLI 30/120/600 秒、MCP 30/120 秒完成及 600 秒约180秒取消、局部规则/Hook/身份观察。所有自动 gate 保留 blocked（继承 always-proceed），不代表产品支持。
- 未验证：`agy agents` 需要启动 language server，受 sandbox 的用户目录写入和 localhost socket 限制而失败；未得到角色列表。
- 规划阶段首次 `./scripts/install.sh` 缺 Rust 而失败；随后第一波经人类批准使用临时 Nix shell 已成功安装 worktree 二进制，`AskHuman --version` / `--agent-help` / `dev status` 通过。没有产品集成代码变更。
- macOS / Windows 均未实测；本机 Linux 命令结果不替代它们。

## 工具与 Hook 接口

内置工具覆盖文件、搜索、终端、权限、子代理、提问和图像。AskHuman 最相关的是 `run_command`、`ask_question`、`ask_permission`、`invoke_subagent`。完整工具列表以官方 Hooks 参考和真实 headless `init.tools` 为准，MCP 工具是否进入同一 Hook 需实测。[官方 Hooks](https://antigravity.google/docs/hooks)

可用事件为 Pre/PostToolUse、Pre/PostInvocation、Stop，stdin/stdout 使用 JSON；有 `conversationId`、workspace 与 transcript 路径。Hook 默认 30s，允许配置 timeout。PreToolUse 可输出裁决；Stop 可要求 continue。公开事件表未列 PermissionRequest、compaction、SessionStart/End 或 SubagentStart。不能移植 Claude/Codex 的事件名和响应体。[官方 Hooks](https://antigravity.google/docs/hooks)

`ask_question` 的官方说明只有输入，未定义可从 Hook 返回用户答案或改写工具结果的契约。因此提示词引导调用 AskHuman 与原生提问接管分开评估；permissionOverrides 也不是原生提问答案。[官方 Hooks](https://antigravity.google/docs/hooks)

| 工具类别 | 官方列出的工具名 | 对 AskHuman 的作用 |
| --- | --- | --- |
| 文件 | view_file、write_to_file、replace_file_content、multi_replace_file_content、list_dir、find_by_name | 安装产物/附件访问边界 |
| 搜索 | grep_search、search_web、read_url_content | 无专门适配需要 |
| 执行与权限 | run_command、manage_task、schedule、list_permissions、ask_permission | CLI运行、异步等待/取消；权限语义需单独验证 |
| 协作 | invoke_subagent、define_subagent、send_message、manage_subagents | child例外及身份隔离 |
| 交互与媒体 | ask_question、generate_image | 前者不等于可接管答案；后者不属首版适配 |

工具名/参数只作为探针 matcher 的候选，不代表每个模型、权限模式和子代理配置都会提供这些工具。[官方工具参考](https://antigravity.google/docs/hooks)

## MCP

CLI 全局配置 `~/.gemini/config/mcp_config.json`，项目配置 `.agents/mcp_config.json`，顶层 `mcpServers`。stdio 使用 command/args/env/cwd；远程使用 serverUrl。AskHuman 首版只需要 stdio；不新增 HTTP 服务。[官方 MCP](https://www.agy.dev/docs/mcp/)

公开 schema 未列长等待超时设置，不能臆造 timeout 字段。MCP 默认还可能触发原生审批；自动集成不默认添加 `mcp(*)` 或整个服务器放行。需记录原生审批与安全影响。共享 `.gemini/config` 路径可能影响其它 Antigravity surface，实施时必须检测并告知。[官方 MCP](https://www.agy.dev/docs/mcp/)

本机 `agy mcp add` 接受 name、commandOrUrl、args 和 type/env/header；flags 必须位于 name 前。它没有公开 scope 参数。自动集成直接使用既有最小 JSON 编辑器，避免 CLI 重写整个用户配置或隐式选择错误 scope。

## 提示词规范与加载优先级

规则是累加加载，子目录约束优先；并非按最高优先文件替换其它规则。AGENTS.md / GEMINI.md 常驻；模块规则必须有 `trigger: always_on`（或 model_decision/glob/manual），无效 frontmatter 会被忽略。CLI 专属全局位置可选 `~/.gemini/antigravity-cli/rules/`，避免主动写共享 GEMINI.md。[官方 Rules](https://www.agy.dev/docs/rules/)

同 scope 的 AGENTS/GEMINI、插件与同名文件冲突顺序没有完整契约，不声称 AskHuman 可以覆盖它们。单文件 24,000 bytes（含展开引用），常驻规则共享 20k-token 预算，超额的大规则可能降为指针；`@[label](path)` 内联，`@filename` 只规范化路径。受管协议应短小、常驻并带说明。[官方 Rules](https://www.agy.dev/docs/rules/)

Skills 用 SKILL.md 封装工作流，支持按需选择和 slash 调用，不能代替强制交互协议的 always-on 规则。[官方 Skills](https://www.agy.dev/docs/skills/)

自定义 agent 使用 YAML frontmatter 与 Markdown body；全局目录 `~/.gemini/config/agents/`，workspace `.agents/agents/`。mainAgent/subagent、tools、model 等控制角色，子代理有独立 context。AskHuman 协议继续保留子代理唯一例外；仅提示父代理声明例外不等于可靠识别 child hook。[官方 Subagents](https://www.agy.dev/docs/subagents?tab=cli)

## agy 使用与版本差异

本机 help 确认：交互入口 `agy`，初始任务 `agy -i`；headless `agy -p`；输出 text/json/stream-json；继续 `--continue` / `--conversation`；mode accept-edits/plan；model/effort/agent/sandbox。不得默认添加 dangerously-skip-permissions。

官方 headless 使用缓存登录态；stream-json 提供 conversation_id、工具事件和结果，支持恢复会话。但它不是 MCP 每次调用携带会话 UUID 的证据。[官方 Headless](https://www.agy.dev/docs/cli/headless/)

`/fork` 出现在交互参考，但本机 help 没有非交互 fork 参数；`--conversation` 表示 resume，不能把它标成 fork。官方 `agy agents` 已公开，本机也识别该命令；旧 agent-team 文档的“无 listing 接口”记录不应当作当前结论。[官方 CLI reference](https://www.agy.dev/docs/cli/reference/)

## 与现有集成对照

| 能力 | Codex / Claude Code 已有基础 | Antigravity 证据与取舍 |
| --- | --- | --- |
| CLI 人工提问、结束交接、待办 | 共用 AskHuman CLI 与协议 | P0 推荐入口；先验证 run_command 的等待与续取 |
| MCP 六工具与图片 | 现有 stdio server、各家配置 | P0 探针，P1 条件开放；长等待未知 |
| 常驻协议 | 受管 Rules / marker | P0 CLI 专属 always_on 文件；不是按需 Skill |
| 生命周期 | 原生 hooks，registry | P1 PreInvocation/Stop 可候选；无原生会话起止事件 |
| 精确 show_last / 压缩恢复 | 真实 session 或一次性 token | P1 会话绑定；无 UUID 时明确弱键，不冒充精确 |
| Stop 继续 | 各家 continuation | P1 有候选官方返回，但等待/自然停止/子代理必须实测 |
| 插话 | 同步 hook 消费队列 | P2 验证模型调用前注入，保持送达回执语义 |
| 权限接管与权限记忆 | PermissionRequest 和家族专属策略 | P2；缺原生 PermissionRequest，不给所有工具加审批 |
| 原生选择题接管 | Claude updatedInput.answers | P2/后置；agy 未公开答案注入契约 |
| Watch / 全会话 / 标题 | 家族 parser | P2 存储 schema fixtures，不能只靠 transcriptPath |
| 新建任务 / Fork | readiness + native terminal launcher | P2 macOS/Windows 原生验收；Fork 后置 |

## 仓库实施入口

身份与 capability：`src-tauri/src/agents/mod.rs`、`integrations/agent_rules.rs`、`agent_mode.rs`、`src/lib/types.ts`、`src/views/settings/useIntegration.ts`。
规则与 MCP：`integrations/mcp_config.rs`、`src-tauri/src/prompts.rs`；CLI 展示 `cli/agents_cmd.rs` / `doctor.rs`。
高级能力：`integrations/agent_lifecycle.rs`、`agent_stop.rs`、`agent_permission.rs`、`agents/interject.rs` / `registry.rs` / `context_recovery.rs`、`integrations/agent_launch.rs`；会话读取 `agents/session_paths.rs` / `transcript_full.rs` / `title.rs` / `workspaces.rs`。
这些是仓库事实（研究 worker T-003），高级能力只在各自证据 gate 通过后进入范围。
