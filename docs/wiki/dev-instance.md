# 开发实例：日常系统，验证显式 dev

普通 `AskHuman`、生产 MCP 和 Hook 默认使用系统安装及系统数据。进入已启用开发实例的目录不会改变日常会话的渠道、历史或待办来源。

```bash
AskHuman dev enable
./scripts/install.sh
AskHuman "日常讨论"
AskHuman dev exec -- "验证本树改动"
AskHuman dev exec -- daemon status
AskHuman dev exec -- --settings
```

enabled 标记控制安装目录：默认安装仅更新本树 `.askhuman-dev/bin`。`dev exec` 仅接收 AskHuman 参数；缺少或无法执行实例 bin 时失败，不回退系统。相对附件路径仍按原调用目录解析。

```bash
AskHuman dev target          # 当前普通调用目标
AskHuman dev target --dev    # 本树显式开发目标
AskHuman daemon status       # 系统 daemon
AskHuman dev status          # 本树实例状态
AskHuman dev exec -- mcp     # 独立测试 MCP server
```

`dev target` 显示目标、binary、home、endpoint 和选择原因，不连接 daemon。直接调用 `.askhuman-dev/bin/AskHuman` 也始终使用该实例 home，即使 cwd 位于其他目录；显式 `ASKHUMAN_HOME` 与实例身份冲突时报错。普通系统入口保留显式非空 `ASKHUMAN_HOME` 的高级隔离用途。

开发实例默认 popup-only，不继承生产 bot 或钥匙串。测试 IM 必须在该实例配置独立测试 bot；不要将生产 MCP/Hook 注册改成开发路径。验证集成写入须使用假 HOME 加非空 ASKHUMAN_HOME，详见 [工作树准备](../agent-worktree-setup.md)。

从旧自动路由版本迁移时，还须更新系统安装入口：Unix 使用 `./scripts/install.sh --global`，Windows 使用 `.\scripts\install-windows.cmd -Global`。仅更新 dev bin 不足以覆盖旧系统 dispatcher。检查 `AskHuman dev target` 与 PATH，确保实际执行的是更新后的系统入口。既有在途请求继续留在原实例，更新不迁移请求。
