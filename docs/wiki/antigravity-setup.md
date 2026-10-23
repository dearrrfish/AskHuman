# Antigravity 集成（实验性）

在设置的 Agents 面板选择 Antigravity → CLI，或执行：

```bash
AskHuman agents mode antigravity cli
```

AskHuman 在 `~/.gemini/config/rules/askhuman-antigravity.md` 安装家族共享的 always_on 协议。
CLI 表示调用 AskHuman 命令行，不表示仅支持宿主的 TUI。缺少 `agy` 不阻止 IDE 用户安装共享规则；每个客户端是否读取规则、执行命令和正确等待，都需单独验证。
当前 P0 为实验性集成。NixOS 上已安装 CLI 的提问、附件、同题未绑定并发、待办和下一任务交接经过实际测试；agy 1.2.9 的基础调用已观察，但完整原生认证尚未通过。

协议中的调用均携带家族提示：

```bash
AskHuman --caller-agent antigravity "请确认下一步"
```

该提示只声明家族。原生会话 UUID 不作为可信绑定；继承其它 Agent 家族标记时会拒绝冲突。
未绑定的 `--show-last` 会拒绝项目级历史回退，请重新向人类确认。不要从其它会话或最近日志猜测历史。
使用客户端实际可用的命令工具等待真实回答；不要把转后台的等待参数当作人类回答期限。
协议只对具备对应工具契约的 agy 宿主说明原 TaskId 的续取方式，不假设 IDE 或扩展工具相同。

查看与清理：

```bash
AskHuman agents mode antigravity
AskHuman agents show antigravity
AskHuman agents update antigravity
AskHuman agents mode antigravity none
```

重复安装/当前版本更新为 no-op。None 只删除当前实例能核验所有权的独占文件，保留共享目录和其它规则。
未知同名文件、用户修改、文件替换、账本或 pending 事务错误都会拒绝覆盖；读取错误不会显示为“未安装”。
Dev Instance 的账本与主实例隔离，不会互相认领同一全局文件。不要把临时验收安装当成主环境永久配置。

MCP、可信会话恢复、生命周期/Stop、权限接管、插话/Watch、新建任务和 Fork 均未开放。
agy 1.2.9/NixOS 已取得原始同 TaskId、临时 Request Review 正/负对照、实际取消和结构化短回答证据。精确内容只认证 `result.structured_output.output`；自由文本 `result.response` 会聚合等待提示。[真实 30/120/600 秒等待](../investigations/antigravity-cli-real-waits.md)及同任务精确输出已通过，完整原生认证未通过。AskHuman 不自动安装权限 allow，验收临时设置已恢复；其它 agy 实例共享用户级设置。
Antigravity IDE、扩展、2.0 及 macOS 未完成实机验收。Windows 严格 adapter 的 identity/reparse/ACL 实现与实机验证仍待完成，专用查询/写入会明确报不可用；它不会阻断其它家族的聚合诊断或协作风格更新。

详细证据与剩余项目见 [实施计划](../plans/antigravity-cli.md#p0-执行记录2026-10-04)。

2026-10-16 [恢复验收](../investigations/antigravity-cli-n0-resume.md)：本机agy1.2.16/AskHuman0.14.5已观察无hint恢复安全拒绝、原生child字段和Unicode/空格附件问答；修复Linux冷弹窗空白。旧1.2.9长等待认证不自动继承至新版本，当前权限认证/可信绑定仍未开放。
