# Antigravity CLI 原始任务与权限验收

运行 `antigravity-native-acceptance`，2026-10-04，本机 NixOS/agy1.2.9。
范围为已交付 P0 的 native 等待/取消/精确输出 gate；不是其它平台或客户端认证。

本轮按人类修订范围完成：原始TaskId严格诊断、RequestReview准确正负对照、同任务实际取消
（人类接受外部终态确认），以及真实短回答的官方structured_output.output逐字精确。
17Node/8Python及install通过。自由文本result.response仍聚合等待提示；真实30/120/600
在本轮曾另约，现已完成[真实长等待矩阵](antigravity-cli-real-waits.md)；完整native/platform gate未放行。原始settings完整bytes/0600已恢复，
临时permissions移除，正式规则仍None，产品高级能力未更改。


## 初始发现与预检（历史状态）

原 P0 自有测试 init 注册的完整原生 `transcript_full.jsonl` 保留了启动命令对应的
GENERIC/RUNNING 后台回执，包括原始 TaskId。其 source 字段实际为 MODEL，不能要求
TOOL 后跳过检查，也不能把任意 MODEL 文本当作回执。原始 run_command 的 stream-json
DONE output 和 steps/output.txt 均已变成最终 child stdout，未保留最初 TaskId。
对应 completion message sender 与原始回执一致，只是交叉观察，不用于反推来源。

人类批准本次诊断使用严格关联的原生记录：唯一测试 init、agy1.2.9、唯一准确命令/Cwd、
紧随启动的固定原生回执格式、TaskId/step/log 路径一致。未知版本/格式、多个启动、
伪造文本或模糊对应均拒绝。仅读取本次已注册 fixture 的路径，不扫描最近会话；
它不提供产品 session binding，也不开放历史/生命周期/Stop。

限定字段只读检查确认全局 toolPermission 为 always-proceed、allowNonWorkspaceAccess为true。
命令没有危险跳过权限参数，所以原 blocked 来自继承设置，不能把删除命令flag等同于修复。
人类批准临时切到RequestReview/非workspace访问false，仅放行固定Node+测试helper命令；
其它字段保留，测试后按完整原始字节和mode恢复，检测其它修改则保留并报告。
该预检阶段尚未应用；后续实测与最终范围见下文。

## 实测合同

- 权限：init request-review；准确未放行命令必须被原生headless策略拒绝且未执行。
  不能把模型解释、OS EACCES 或 stderr单词permission当作认证。
- 真实30/120/600：一次helper、一张真实Popup卡，实际等待时长达到门限并收到真实回答。
- 输出：人类回答后才生成随机nonce；同一原始TaskId status读到完整childstdout；最终response
  与该完整block精确一致（只规范CRLF/外围空白），不能预知prompt标记后直接回显。
- 取消：原始TaskId kill、同任务终态status、helper信号/child退出确认，以及专用闲置
  popup-only Dev Instance唯一请求的caller disconnect取消和finalized记录；活动请求归零。
  异步helper退出不代替daemon请求关闭。

17项Node来源/判定测试与5项Python事务测试通过；install.sh exit0，最终P0二进制字节未变化。
配置恢复验证覆盖restrictive umask、精确字节/mode、外来修改与identity替换，以及恢复时同字节换inode/更严格mode拒绝。
独立C1预检发现四项漏洞已修复；C2预检approved（只审查工具，不是native能力通过）。该预检阶段未运行live等待或取消，不能据预检宣称gate通过。

## 来源与限制

[官方headless](https://www.agy.dev/docs/cli/headless/)说明继承设置、soft denial及全局permissions.allow。
[官方permissions](https://www.agy.dev/docs/permissions/)说明allow/deny/ask与匹配语义。
最新在线文档不等于本机1.2.9实现。公共二进制同时包含较旧的“不支持headless settings allow-rules”
提示，须实际负例和固定helper正例验证；失败恢复，不用危险skip或未经批准升级掩盖。
原始私有日志/设置备份不导出；仓库仅记录脱敏证据。

Windows严格writer、macOS/其它客户端、MCP约180秒取消、可信绑定、child/IM/无hint ancestry
均仍为独立未验收项。现有P0已接收本地交付不受本次诊断结论伪扩展。

## 首次 Request Review 实测及恢复

人类选择先权限与取消，真实30/120/600另约，已记待办。两次init均request-review。
未放行负例与固定helper正例分别出现原生run_command ERROR/TOOL_ERROR权限拒绝；
各自完整原生记录确认CommandLine/Cwd准确、唯一调用。没有helper启动、Popup、TaskId或kill，
所以取消未执行。自动判定继续blocked：实际ERROR事件及stream仅暴露CommandLine，与预检
假设的DONE/Cwd/notice形式不同；没有为了通过而放宽判定。脱敏记录在run/native-policy-observation.json。

测试期间settings被重写为仅permissions.allow（本次固定entry），两个根字段消失；无关字段
没有变化，mode0600，inode/hash变化。严格restore拒绝覆盖，人类另行批准具体恢复，
恢复工具验证review后当前hash/inode/mode并恢复完整原始bytes/0600，exit0。
8项隔离事务/恢复测试通过（另含17Node），最终恢复安装exit0，产品二进制不变。
恢复工具是人类批准的特定异常恢复，不代表正常restore可忽略任何漂移；
已批准C2预检覆盖原工具，后来恢复代码仅有root测试及人类明确审核，未虚称第三轮独立审查。

[CLI官方文档](https://www.agy.dev/docs/cli/reference/)记录旧toolPermission模式；
[官方仓库issue548](https://github.com/google-antigravity/antigravity-cli/issues/548)有headless忽略allow的
第一方用户报告（Windows，不能直接认定是本机相同原因）。本机正例拒绝原因仍需确认。
原设置已恢复，当前无临时全局allow。后续选择更正精确规则、TUI逐条原生审批或版本迁移，
不得用dangerousskip替代真实权限验证。

## 并行实例干扰控制

人类随后告知首次测试期间有其它agy实例。本次只隔离AskHuman实例与项目，没有隔离agy
用户级settings。其它实例可能读入临时权限或写回缓存；第一次正例失败和配置重写归因
均不充分，不能据此认定1.2.9缺陷。人类退出其它实例并要求ps验证；
`ps -eo pid,ppid,comm` 筛选agy/antigravity/language_server无匹配，原始settings bytes/0600
仍完全一致。取得新恢复基线后重新应用同一已批准差异，复测negative/cancel并以50ms
监测settings hash/inode/已知字段存在性。未读取其它会话日志或终止其它实例。结果待回填。

## 无其它实例的命令字面形式对照

隔离复测负例/固定helper（带单引号）仍被拒绝，第一次negative时50ms监测再次捕获
根字段消失/相同规范化hash；不是并行其它agy的充分解释，但不能排除其它非agy写入者。
恢复后用相同原始基线/同一approvedpermissions差异，固定helper命令仅去掉单引号
（所有路径均限定无空格/Unicode/shell元字符，只改变字面形式、不改argv/allow范围），
成功启动真实取消卡。再用fresh同配置带引号对照，准确命令/Cwd仍被headless拒绝，helper0。
本组证据支持本机1.2.9该literalallow与引号形式匹配不同；不推广所有引号场景。

无引号取消：原生完整回执严格parser通过；一launch/一helper/一kill，kill/status原始同TaskId；
kill输出cancelled，status明确CANCELED；daemon请求accepted06:20:07、callerdisconnect及
finalized06:20:10，原生status/final响应06:20:14。因此取消早于native最终回复及runner
退出后的processgroupcleanup，不是rootcleanup伪造。helper/AskHuman进程随后均消失，
没有expectedresponse或真实回答，native最终输出NATIVE_CANCEL_CONFIRMED精确。

helper没有写入自身signal/closed/cancelled回调，不能声称优雅信号处理通过；
严格自动gate因该ack形式仍未满足，待人类决定是否采用原生CANCELED+daemon提前finalized
作为实际取消确认。此处为root手工严格关联证据，不是C2自动工具gate通过。
每次观察到完全相同已审核规范化bytes，沿用人类已批准同一恢复差异、重新登记当前identity
后恢复原始bytes/0600；所有其它字段相同，否则拒绝。当前再次恢复完成。

## 已接受的取消与结构化短回答范围

人类明确接受取消的外部确认依据（原始同TaskId/CANCELED、daemon先finalized、进程结束），
不要求在被结束的helper内伪造信号回调。随后真实短回答完成：自由文本result.response将
81-byte初始等待提示与127-byteblock拼接，整段精确比对失败。stream text_delta是分块，
不能取最后DONE的一段当整个回复。保留失败记录。

使用本机支持的官方--json-schema对照，一张真实Popup、一次helper、一个原始TaskId
续取、helperexit0，实际等待9360ms；完成后随机nonce的127bytes完整block与
result.structured_output.output逐字相等且仅output键。result.response仍不能直接JSON.parse，
认证消费者应读官方structured_output字段，不宣称自由文本字段精确。人类明确接受这一范围。
Popup历史唯一条目familyantigravity/session未绑定；这不是P1可信binding。

同样无引号形式的另一个Node私有负例准确CommandLine/Cwd得到原生TOOL_ERROR权限拒绝，
未写sentinel，因此本组grant没有泛化到所有Node命令。所有测试末尾恢复完整原始bytes/0600、
临时permissions移除、ps无agy；恢复方案只在先前人类已审核的完全相同规范化bytes及
无其它字段变化时复用，新的漂移拒绝。正式共享模块仍None，产品高级capabilities未更改。

30/120/600真实等待人类另约，待办e613e6c3-b580-4646-b961-fbe0c4d42f3f；完整native与
平台认证仍未完成。旧探针automaticgate继续blocked，这些新结果是明确范围的手工/结构化
验收，人类接受，不冒充第三轮独立审查或整族放行。

## 后续真实长等待完成（2026-10-05）

[独立后续运行](antigravity-cli-real-waits.md)完成真实30/120/600秒，实际时长61.930/138.807/627.596秒。三例原任务续取、结构化字段精确、唯一Popup及进程清理均通过，临时设置完整恢复。上方另约描述保留为历史。
