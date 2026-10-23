# Antigravity CLI-first 前置验证

日期：2026-10-04；NixOS Linux，agy 1.2.9，worktree AskHuman 0.13.2。
这是诊断验收，不是产品集成放行。前序见 [第一波实测](antigravity-cli-probes.md)，合同见
[规格](../specs/antigravity-cli.md) 与 [计划](../plans/antigravity-cli.md)。

## 结论

真实未绑定 CLI 弹窗传输已通过；现有 CLI-only 全局规则候选没有自动加载，仍阻塞 P0 安装载体。
相同字节的本地规则零工具对照通过，候选内容能被该版本解析；尚不能确定全局发现失败的原因。
已批准的跨家族冲突拒绝及无绑定历史 fail-closed 保持不变。

## 全局载体

人类明确授权一个独占临时规则文件的创建与受保护删除，并在发现缺目录后另行授权仅创建
`~/.gemini/antigravity-cli/rules`，权限 0700。目录保留；临时文件已删除，创建及失败记录保留。
没有修改已有全局规则、MCP 或权限配置。

候选是短小 `trigger: always_on` 模块，只在独立随机请求标签出现时回答另一个独立随机标记，
其它请求不受影响。SHA-256 为 `45e87008dd644aee4f4377b305c04545a99ab13cf672f15386f01d567497df61`。
所有会话使用新项目控制，禁止工具与文件检查；最终均 SUCCESS，工具数均为 0。

| 场景 | 回答 | 本地证据目录（夹具 UUID） |
| --- | --- | --- |
| 创建前 | UNKNOWN | f9c9fbb6-4836-45fb-9f2c-d6c3006239d7 |
| 全局文件存在时 | UNKNOWN | 68a1a036-578e-4c70-88ff-a727600d9ede |
| 删除后 | UNKNOWN | a2a54a9f-48c7-44c3-afa7-3fc9a076d9fa |
| 相同内容放入项目 `.agents/rules/` | 正确独立标记 | ad384b98-0d42-404e-b8f2-3e9a892bd6fe |

[官方 Rules](https://www.agy.dev/docs/rules/)仍列出 CLI-only 全局目录。本机观测与该契约不符，
不能将文件存在当作安装成功，也不能据此断言所有环境/版本均不支持。
下一步应定位加载条件、版本差异或经确认的替代载体；本轮没有试写共享 GEMINI.md/config rules。

文件操作要求真实祖先目录、固定目录描述符、独占创建、原创建 stat 与 checksum 匹配。
10 项离线测试覆盖缺目录、symlink、替换和漂移，协调者复跑通过。
POSIX 最后 stat 至 unlink 的同用户竞争窗口仍存在；中断和漂移保留证据，不扩大自动清理范围。

## 身份冲突

当前 `agents/detect.rs` 没有 Antigravity 分支，任意 CODEX_* 标记可能触发 Codex 检测。
第一波已观察到真实 ANTIGRAVITY_CONVERSATION_ID 与 init/Hook 一致，但同时继承外层 Codex 标记。
不能借外层 UUID 绑定原生 agy，也不能通过最近项目历史或日志猜测会话。

产品计划的 `--caller-agent antigravity` 仍未实现。本轮保持显式提示仅指家族、不提供会话权限的合同，
遇外来家族标记仍拒绝；不改为原生变量静默优先。
诊断只在 AskHuman 测试子进程移除继承 CODEX_* 与 MCP 绑定标记，并拒绝其余家族标记；这不代替产品修复。

## 真实 CLI 传输

原生 agy 只启动一次测试 helper，调用已安装的本 worktree AskHuman binary，固定 Dev Instance home，
保留桌面环境并开启唯一选项弹窗。人类实际选择后，CLI stdout 与最终原生回复都精确匹配选项。
AskHuman 退出 0；真实等待约 8963ms，故不计作 30 秒/长等待验收。原生会话总耗时约 17761ms。
仅读取本 Dev Instance 历史中唯一 source 对应的记录：恰好 1 条，channel=popup，family/session 均为空。
没有调用 show_last、伪造 caller 标记、读取外层会话历史或重启请求。
证据目录：7c8a61a7-ca44-475d-a5fd-4b6b1c2a3057。

这证明未绑定 ask/wait/reply 传输；不证明 Antigravity 归属、恢复、并发隔离、取消、其它 AskHuman 工具或附件。
所有原生自动 gate 仍因 inherited always-proceed 为 blocked，局部传输成功不能解除权限限制。
macOS/Windows、长等待和高级生命周期仍需独立实机验收。

## 验证与留存

经批准的 Nix 环境内 `./scripts/install.sh` 成功；新装 AskHuman version 成功。
两个诊断 MJS 语法通过；三个 Python AST/compile 检查通过；10 项文件安全测试通过。
独立预检两轮，最终 approved，范围仅为具体诊断工具。运行证据由协调者取得。
私有原始数据留在忽略的 `.askhuman-dev/probes`；上表 UUID 是夹具目录名，不是原生会话身份。
没有提交、发布或加入产品功能。主 overview 的架构地图与不变量未变化。

后续共享模块对照与正式替代候选见 [载体诊断](antigravity-cli-carrier.md)；未自动改变本轮批准的CLI-only安装合同。
