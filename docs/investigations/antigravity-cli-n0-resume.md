# Antigravity rebase 与 N0 恢复记录

2026-10-16；本机 NixOS/Hyprland Wayland，agy 1.2.16，AskHuman 0.14.5。

## rebase 与恢复

现有 antigravity-support worktree 已 rebase 到本地 main `c1c8b46e6d7768138f46e79ab2431e3658411d38`。原分支没有新增提交，原功能与文档在未提交工作区；先保存 tracked patch、30个新增文件、旧 agent-team 配置和 ignored 冲突快照，再 stash/rebase/apply。七处冲突已解决，保留 main 的 Codex --env/Cursor Desktop 和 Antigravity None/CLI、严格错误传播、关闭的高级能力。新增标题查询补齐 Antigravity unsupported 分支。

恢复 stash `60781de7e85e3b9ffc5ee7d89fdb8003bd86fc7f` 与 `.askhuman-dev/rebase-backup-20261016` 均保留；新增文件完整 hash 复核通过（两份Node测试按现有 *.node-test.mjs 约定改名）。未创建功能提交、推送或操作主 checkout 的 CI 工作区改动。

## 当前 N0 证据

- **无 hint ancestry**：真实 agy run_command 启动一次 helper，移除全部已知家族/MCP环境标记、不带 caller hint；自有 /proc 祖先 comm 链最近 Agent 为 agy，`--show-last` 返回退出1并明确拒绝无可信 session 的项目级恢复。它证明该入口安全拒绝，不证明可信会话恢复。
- **原生 child 观察**：一次实际 invoke_subagent ACTIVE/tool，完成事件改为 DONE/subagent，带 subagent_info.subagents。唯一child提供 type_name/role/initial_prompt/conversation_id/log_uri/workspace_uris；child UUID与parent不同，workspace相同，返回固定标记。记录仅作自有诊断，不把CID当产品binding；未读取child内部工具轨迹或认证child生命周期过滤。
- **空格/Unicode/桌面**：agy在`workspace 空格`执行安装后的AskHuman，发送`附件 空格.txt`，操作者确认正常并真实返回 N0_DESKTOP_CONFIRMED。唯一历史为准确source/project、popup/antigravity、无session，helper退出后消失。
- **四 IM**：无测试preset；人类明确选择后移，待办 `4c1d4a68-18f4-4642-a3ff-e5a61deee0fc`。不连接生产bot或抢租约。

本机 antigravity/code 未在PATH发现，这不是证明GUI客户端未安装。旧agy1.2.9的长等待/权限/回执证据保留为旧版本结果；本次不继承这些认证到1.2.16。当前继承用户已有 always-proceed，未主动更改权限或安装allow，所有本轮结果是受控功能观察，不是权限认证；也未宣称1.2.16原始TaskId严格认证。

## 实际发现并修复的 Linux 空白弹窗

第一次Unicode测试窗口出现但内容空白，人类报告失败，已取消该自有CLI并保留负面证据。通过临时loopback WebKit inspector观察：Vue和请求DOM存在，原生window可见/1070×638，但WebView隐藏、viewport/body为0×0。定位Linux front路径只调用gtk_widget_show显示顶层，没有递归显示隐藏的WebKit子控件。

`app/popup_transition.rs` 改用 gtk_widget_show_all，保留 focus-on-map=false 与原有raise策略。GTK说明该API会递归显示子控件（[官方文档](https://docs.gtk.org/gtk3/method.Widget.show_all.html)）。安装后真实卡正常确认；回答后再次测量WebView仍具有818×593.33布局尺寸（这是回答后快照，不冒充活动卡尺寸）。没有改GPU环境或全局桌面配置。

热窗口另一次perfAutodismiss检查返回唯一cancel历史与正常status块/退出0；未采到fe.painted打点，因此只记录自动取消观察，不认证绘制时序。取消是测试自动行为，不表示人类取消当前开发任务。

所有调试CLI均结束，临时inspector已关闭；以真实host PID namespace确认helper消失、Dev Instance 0 active / IM none、正式Antigravity模式off。普通沙箱的/proc看不到host进程，曾得到的早期进程不可见判断已由host复核替代；不能据它声称机器没有其它agy实例。未停止其它实例或读取其它会话日志。

## 验证

- 最终安装 `./scripts/install.sh`、cargo fmt --check：exit0。
- 隔离 ASKHuman配置的 cargo test：1332 passed，3 ignored，1 filtered（已知依赖真实Cursor数据库的real_vscdb_roundtrip_when_available）。首次未隔离运行还受用户collaboration配置影响；对应测试源是main已有，未修改用户配置来使测试通过。
- 修复后的 cargo clippy --all-targets -- -D warnings：exit0。
- `pnpm test`：394 Vitest +49 Node passed；包括17项原生探针解析/取消测试。8项Python事务测试通过。Node Unix socket测试需host权限，普通沙箱listen失败不算产品失败。
- 文档链接、scoped diff与run render检查另存本run证据。当前只认证已实际执行的本机范围；IM、child产品归属、可信binding/MCP/生命周期/Stop和其它平台仍独立gate。

本轮恢复后可进入N1来源/绑定契约设计；N0四IM待条件就绪再验收，child诊断不自动开放高级能力。当前不发布或永久安装规则。
