# Antigravity 1.3.5：可信来源可行性决策

2026-10-18；承接交付设计审阅。用户已选择保持原 v1 范围，优先解决可信来源。本文不修改该范围，不开放 MCP，不实施新的启动/隔离合同。

## 当前可支持的结论

当前资料不足以证明“普通 agy 启动 + 无额外执行隔离”可达到现行来源合同。新版文档和本地包没有提供可以直接落地的认证证明；也不能据此断言上游没有相关私有机制。

下一决策应落在实际信任边界上：继续等待/调查可验证的原生保护合同，或研究带强制执行隔离的受控启动。仅增加 CID、Hook、token、broker 或更多端点形状不能自行解决模型命令冒充原生来源的问题。

## 本轮新核对

### 1. 固定本地构件

只读核对本地包 `/nix/store/dhrxfv34r1ls35217y215qqjjdkmx7ik-antigravity-cli-1.3.5/bin/agy`：单个 ELF，215060568 字节，SHA256 `90d29e145af960cf313ffef98f0f0db8ef6336b2eb04c5b63df55d6042aa11ea`。

包文件枚举只发现该可执行文件，没有可直接审查的附带源码。本轮未启动 agy、未访问生产会话日志或凭据、未读取运行中进程状态。

有界静态字符串检查发现 conversationId、parentConversationId、workspacePaths、roots/list、call_mcp_tool 等字样；toolCallId 命中包含通用 chat protobuf，invocationId 命中包含 build/test protobuf，不能把这些通用字段当作 CLI→MCP 共享调用 ID。未命中特定 syscall 名也不能证明二进制不执行对应 syscall。静态检查只定位候选，不提供调用链和运行安全证明。

### 2. 当前官方接口仍缺完整保证

| 接口 | 官方明确描述 | 对本任务仍缺的证明 |
| --- | --- | --- |
| Hook | conversationId、workspacePaths、toolCall、stepIdx；可 overwrite 参数 | Hook producer 防冒充；与实际 MCP RPC 的唯一关联；完整项目集合的生命周期 |
| Headless stream | conversation_id、step_index、tool_info，部分 child 信息 | 与 Hook/MCP 共用 ID 的合同；普通 TUI 的等价观察；事件/响应通道保护 |
| CLI terminal sandbox | Linux namespaces；命令隔离，可配置绕过 | 是否覆盖所有模型代码执行/文件工具；是否保护 native FD/内存/代码/配置及 AskHuman 状态；bypass 时如何失效 |
| MCP | stdio/remote 配置、服务端认证与权限规则 | 原生逐调用 CID/role/project 的端到端认证合同 |

来源：[Hooks](https://antigravity.google/docs/hooks)、[Headless](https://antigravity.google/docs/cli/headless)、[Sandbox](https://antigravity.google/docs/sandbox/)、[MCP](https://antigravity.google/docs/mcp)，本轮访问日期 2026-10-18。网页描述不绑定本地 1.3.5 build；“没有在这些页面建立保证”不等于所有实现中不存在机制。

特别是 sandbox 文档允许 unsandboxed 规则和请求绕过。把 `--sandbox` 加到启动命令不能单独证明现行威胁边界成立；本轮没有改权限模式，也不将自动批准等同于不安全。

### 3. 发现需要澄清的信任边界差异

目标 worktree `docs/specs/antigravity-n1-source-binding.md:27`：

> 威胁边界沿用现有用户私有 IPC/文件和受管 Hook，并不宣称能防住已能读取同 UID 私有状态的恶意程序。

`docs/specs/antigravity-source-adapter.md`“信任边界与准入”也排除已能任意读取私有凭据、操纵其它进程/OS 的恶意同 UID 程序，同时明确模型公开命令和 helper 必须纳入负例。

后来的 `docs/plans/antigravity-v1-source-options.md:34–40` 则要求：模型公开工具/命令、parent-FD reopen、响应截取、native 内存/配置代码注入路径都须说明实际限制；不能凭 0700 或同 UID 私有状态将其排除。其 run requirements.md 的 FR-002 也明确了更强的假设，不能把这段当成随手写错而直接删去。

这是有实际工程影响的边界张力，不足以证明后续要求未经授权。它意味着“现有用户级集成的可信归属”与“防止有执行能力的同用户 helper 攻击”不能混用同一个完成口径。

源码提供了基线：

- `src-tauri/src/history.rs` 将问答序列化为 JSONL，Unix harden 使用 0600；这是用户间文件权限，不是同 UID 执行隔离。本轮只读源码，没有读取真实历史。
- `src-tauri/src/mcp/ask.rs:995` 起的 `resolve_direct_binding` 对 Codex 的 threadId 依赖原生逐调用覆盖约定；此函数没有实施 Antigravity 计划中的完整 ingress 认证。本观察不等于对 Codex 整体安全性作结论。

**推论：** 若模型命令在相同 UID 下能够直接读取目标历史、改写相关配置或操作 native 通道，仅强化 Antigravity adapter 不能保护整个链路。需要证明这些访问实际受到限制，或者明确采用较窄的威胁边界。不能以其它 adapter 较宽松作为忽略 GF-01 反例的理由。

## 保持现行 v1 的两条路线

### A：维持普通 agy 使用方式，要求原生提供可证明机制

要求 native 或独立可验证机制同时解决：

1. 保护请求与响应通道；普通模型工具不能冒充 producer 或截取响应。
2. native 每次调用生成不可由模型覆盖的 CID/role、唯一 invocation 与参数绑定；主/child、同参并发和重启可区分。
3. 完整 workspace snapshot/generation 与该调用绑定，变化使旧绑定失效。

优点是有机会保留普通启动、TUI 和现有工具能力。缺点是当前没有可审查的现成机制证据，可能依赖上游。下一产物应是具体接口要求和已知缺口，不再泛搜相同文档或追加 UUID/pipe 形状探针。

### B：受控启动 + 强制执行隔离，再研究 native metadata/Hook 关联

仍使用 agy CLI，保持六工具、exact CID 恢复、Popup 和自动模式目标；但可能改变启动方式、文件/网络访问与部分命令能力。这是新运行合同，必须明确接受后才实施。

最小候选的可信侧包含 agy 原生事件、AskHuman broker 与其代码/配置/数据；不可信侧包含模型启动的全部代码执行入口。必须证明两侧实际分离。只把 agy 和 helper 一起包进同一个 bwrap、只隐藏 `/proc`、只包装 run_command 都不够。

第一阶段只研究以下可否实现：

- 固定本机 binary/profile，列清主/child 及所有执行、文件写入、插件/Hook 动态加载入口。
- native sandbox 或外部 supervisor 能强制隔离所有相关入口，且不能通过 bypass、另一个工具或修改配置绕过；保留哪些日常命令、哪些受限必须列明。
- 原生直接文件工具也不能修改可信侧代码/配置/数据；同时考虑宿主随后加载 workspace 中可写文件的路径。
- 在上述边界内，再确认原生 vendor metadata 防覆盖、Hook 覆盖/共同 ID、完整逐调用单项目集合和失效。

这不是承诺 B 已可行。若固定 binary 无法提供可强制覆盖的入口，不应转而做大型通用 sandbox；记录明确失败点，回到 A。

## 推荐的停止条件与验收

推荐先决定是否接受 B 的研究边界；若不接受启动/执行限制，保留 A 的上游依赖状态。不要同时要求“无任何隔离/使用限制”与“防御拥有相同访问能力的恶意 helper”，然后期待 adapter 自行完成。

研究顺序：保护边界 → 唯一调用关联 → 项目权威。每一项在前项无机制时停止。不是立即运行三套新的 native 探针。

只有找到具体机制后，才准备固定 argv/cwd/config/输出范围的 fake MCP 正反例：真实 main/child、同 CID 重复、跨 CID 同参并发、模型伪造 metadata、受控自有通道攻击、重启与项目变化。用例必须同时检查拒绝前零业务副作用与合法调用可用性。诊断输出仍不能自动成为产品认证。

任何新的 native 执行、配置或隔离部署须在具体方案可审查后另行授权。本轮不启动新运行，不恢复旧 transport 任务，不安装、不修改产品或降低安全门槛。

<!-- # VERIFY: Establish a concrete protected execution boundary and native per-call authority on the pinned 1.3.5 build before product admission; this read-only investigation is not runtime proof. -->
