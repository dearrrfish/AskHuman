# Antigravity 本机客户端与配置库存

2026-10-16；NixOS/minos；AskHuman worktree `antigravity-support`，HEAD `c1c8b46` 加未提交 P0/N0。人类确认 IDE 已安装并授权只读参考 `/dotnix`。本次未启动 IDE、App 或模型，未读取用户扩展清单、会话数据库、认证材料或其它原生日志。

## 安装证据与验证边界

| 宿主 | 实际证据 | 可得结论 / 不能推导 |
| --- | --- | --- |
| agy CLI | [N0 恢复实测](antigravity-cli-n0-resume.md)：自有真实运行报告 1.2.16 | 本机 CLI 可运行；1.2.9 的长等待、规则和权限结论不自动继承 |
| Antigravity IDE | `/etc/profiles/per-user/yjin/bin/antigravity-ide` 指向 `/nix/store/ydv702v69qy0bnhg2jaq81nyf13hsfr3-antigravity-ide-2.5.5/bin/antigravity-ide`；desktop entry 为 `antigravity-ide %F` | IDE 已安装；不能从 PATH 中缺少 `antigravity` 或 `code` 判断未安装 |
| IDE bundle | immutable `resources/app/package.json`、`product.json`：version `1.107.0`，applicationName `antigravity-ide`，quality stable，commit `ecfbad74d93962fc8ca485d93ab9b4f3d4cb6cf8`，date `2026-08-13T08:37:22.547Z` | Nix derivation `2.5.5` 与 bundle `1.107.0` 分别记录；不能猜测两者 API/release 对应关系 |
| IDE bundled extension | `resources/app/extensions/antigravity/package.json`：`google.antigravity` `0.2.0` | 只证明随 IDE 打包；不证明运行时激活或其它编辑器安装了该扩展 |
| VSCodium | 活跃 profile 有 `codium`，store 包 `vscodium-1.126.04524` | 独立编辑器；未检查用户扩展库存，不能宣称 Antigravity 扩展可用或不存在 |
| 独立 App / Antigravity 2.0 | 本次授权的元数据范围未建立安装证据 | `not-tested`；不是全机不存在证明，不能套用 CLI/IDE 结论 |

原生实例 PID 命名空间与 sandbox 可能不同；本表不是其它 agent 进程不存在的证明。禁止据此终止用户实例。

## 声明来源及可写配置

`/dotnix` 当时 HEAD `15e83d517ff56c6dab7b86732639bd8f8f43aed6`，已有未提交修改；本次全部只读，记录针对所读工作树内容，不声称仅 HEAD 即可复原。

- `users/yjin/minos.nix:129–135` 启用 editor.vscode、codium、antigravity；`modules/home-manager/feature/editor/antigravity.nix:10–20` 为 IDE 添加包与 Chrome。证明包由 Home Manager 管理，不证明 IDE 用户配置也受管。
- `modules/home-manager/feature/ai/antigravity-cli/default.nix` 将 CLI 设置经 `mkMergeConfigActivation` 合并到 `~/.gemini/antigravity-cli/settings.json`，diff 为同目录 `last-diff.json`。本次只读模块源码，未读取现场 settings 的内容/权限/identity。
- `modules/home-manager/feature/ai/mcp.nix:36–38` 在相应开关启用时把 `programs.mcp.servers` 作为 CLI `mcpServers` 的声明默认值。该文件只直接定义条件 nixos 服务，未对其它模块做全量求值；不能据此断言最终不存在同名 askhuman 条目。
- `lib/merge-config.nix` 调用 `config-merger`；CLI 调用未传 `managedSettingPaths` / `removeSettingPaths`。`config/merge-settings/merge-settings.py` 的 `merge_settings` 递归合并对象，已有本地值优先；缺失声明字段会重新加入。`docs/config-merger.md` 与源码一致。
- merger 对 symlink 有转可写普通文件行为，现有普通文件保留 mode；语义 no-op 保留普通文件 bytes/identity。替换前检查现场 snapshot，但与 AskHuman 无共享锁，检查到 rename 仍存在竞争。此行为不是 AskHuman 安全 writer 的授权或证明。

因此 N1 必须区分**声明受管**与**运行时自有**。同名声明条目不可自动收养，现场 symlink 不由 AskHuman 解耦；Home Manager activation 与 AskHuman 配置事务不得并发。Nix 回滚不等于可写 settings 回滚，也不能保证 None 后声明条目不再出现。

## 证据生产与后续

命名只读 explorer `n1_local_inventory` 读取 active profile、desktop entry、immutable manifest 和上述声明；协调者补读 merger 文档及实现。所有读取命令成功；中止的宽 store 搜索不作不存在证据。无安装、升级、Home Manager activation 或配置写入。

[N1 来源/绑定合同](../specs/antigravity-n1-source-binding.md)以这些库存和当前公开接口为输入；下一步只在独立 fixture 中验证真实 Hook/MCP 传播及 child 来源，再决定宿主能力 eligibility。
