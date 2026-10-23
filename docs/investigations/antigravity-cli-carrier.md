# Antigravity 全局规则载体诊断与替代方案

日期：2026-10-04；当前观察限于 NixOS Linux、agy1.2.9。前序 [前置实测](antigravity-cli-preflight.md) 已证明同字节本地规则加载、CLI-only 全局候选未加载。

## 原因边界

[官方 Rules](https://www.agy.dev/docs/rules/)列出 CLI-only ~/.gemini/antigravity-cli/rules 与共享 ~/.gemini/config/rules，但没有版本下限或确切加载诊断入口。[官方 changelog](https://github.com/google-antigravity/antigravity-cli/blob/main/CHANGELOG.md) 的1.2.15修复涉及代理读取共享 customization 文件，不证明全局 prompt 注入修复，也不能承诺升级即解决。

人类批准只读检查已安装公共程序。Nix store 中版本1.2.9的 ELF SHA256为10b96c5d1bd2149145fdd003ab2a3c3cfbecb8f07e2736ffbc540ac9501b66fe。有限静态检查找到 readGlobalRules/getRules/discoverRulesForDir 等方法；readGlobalRules直接调用fs.ConfigDir。仅字符串缺席不证明路径未实现（可能分段构造），也不能据方法名认定运行时分支。没有读取凭据或用户settings内容。根因目前仍未知。

## 待审查的最小路径对照

准备共享目录中的独占随机always_on文件，不写已有共享GEMINI/AGENTS。规则只响应独立测试请求并输出另一独立标记，其它prompt无影响。使用新项目、零工具的before/present/after控制，与前序CLI-only结果比较。操作要求已存在真实目录、固定目录描述符、独占创建和checksum/stat创建记录；缺目录停止。具体candidate/manifest/helper在本轮run fixtures，10项离线文件安全测试通过。该临时共享文件对及必要的单个 rules 目录创建后来获得人类授权，实测见下节。

## 产品替代方案

| 方案 | 已知证据与收益 | 代价及放行条件 |
| --- | --- | --- |
| 保留CLI-only目录 | 官方列出，物理载体专属CLI | 本机加载失败；查清条件/版本并实测后才能安装 |
| 共享config/rules独占模块 | 官方列出；不编辑共享大文件，易于归属和漂移检查 | 本机原生加载通过；其它Antigravity客户端也可能加载，语义CLI guard不等于技术隔离；需要用户接受表面变化 |
| 项目.agents/rules | 相同内容零工具加载通过 | 每项目设置，且其它读取项目规则的客户端可见；不满足原全局安装目标 |
| CLI插件规则 | 官方列为插件加载源，可能保留CLI目录包装 | 未测；须验证安装/启用/常驻/卸载和跨surface范围，不能直接用lazy Skill替代协议 |
| 共享GEMINI/AGENTS托管块 | 仓库有begin/end块复用先例 | 跨客户端常驻影响更大；不可复用把读取失败当空文件的writer，需要fail-closed、锁、漂移与回滚审查 |

推荐先完成共享模块路径对照，作为诊断；不据此自动选择正式载体。候选共享文件必须明确所有权、读取错误fail-closed、重复安装/卸载和用户漂移拒绝。正式协议应通过CLI解析器强制family提示/冲突规则，提示词不能保证模型执行哪个命令。保持批准的跨家族拒绝意味着继承CODEX_*的嵌套原生环境会被拒绝；这项限制需明确展示，改变合同须另行批准。

当前不实施产品代码，不升级agy，不改共享现有文件，不改变权限或身份合同。macOS/Windows和IDE/其它客户端隔离仍需独立证据。

## 测量修订

独立预检发现只统计DONE工具会遗漏ACTIVE/ERROR尝试。本轮修订为所有tool updates均计数，未知schema不认定零工具；synthetic回归通过。前序四份载体日志已用严格规则复核，均无工具更新或未知格式，故原对照结论保持成立。公开静态方法调用仍不证明实际根目录；初始未限制的静态解析退出137，后续有限读取成功，不把失败当运行时证据。

## 共享模块实测

人类批准唯一临时文件 `/home/yjin/.gemini/config/rules/askhuman-probe-719487dc671cf69badf59c933a6c2a4c.md` 的独占创建/受保护删除，并批准缺失时只创建该 rules 目录（0700），不创建祖先。该目录原先缺失，现保留；临时规则文件已按原stat/checksum记录删除。没有改已有全局文件或权限。候选SHA256：07d38797c2baf5b4580ec5af493f759f385d1a7da5c4c9191f6938019f54a258。

| 控制 | 回答 | 严格零工具判定 | 夹具 UUID |
| --- | --- | --- | --- |
| 创建前 | UNKNOWN | 0 tool updates / 0 opaque schema | d2f1f56c-9785-4db8-932a-e7dc45889dce |
| 文件存在时 | 精确独立标记 | 0 tool updates / 0 opaque schema | 4d2ec375-3e44-4ed8-96fe-33eae4309092 |
| 删除后 | UNKNOWN | 0 tool updates / 0 opaque schema | aee8efb2-399f-4f36-b446-5143ff8c616e |

全部原生SUCCESS且进程退出0；present的carrierLoadedObserved=true。所有自动status仍为blocked（inherited always-proceed），不扩张为权限认证。共享全局模块有可重复的加载/清理行为证据；前序CLI-only候选没有加载，同字节本地对照通过。两组全局探针使用独立标签和不同描述，未声称完成同字节跨全局目录交叉对拍。

已定位可用的替代加载路径，未定位CLI-only目录遗漏的精确实现/版本原因。静态Loader调用ConfigDir与共享实测相符，但无法排除其它加载条件；公开源码/版本对拍或受控追踪仍需后续证据。不能承诺升级后CLI-only目录即可恢复。

## 可供批准的P0方案

若人类接受共享surface范围，可以将正式载体改为共享目录中的独占 `askhuman-antigravity-cli.md`，沿用frontmatter与本家托管标记，保留所有权/checksum漂移拒绝、锁内重复安装/卸载和跨客户端范围告知。具体产品实现前需冻结marker/version规则与回滚测试；不能用现有把读错误视为空文件的writer。

协议可明确仅适用于agy CLI，但这只是语义约束，不承诺其它客户端完全不受影响。CLI解析器应继续强制显式家族提示、跨家族冲突拒绝、无会话绑定不恢复历史及不合流；任何新增原生环境eligibility guard也需独立设计/批准，不能把环境UUID当可信权限。仅接受共享载体并不批准改变身份合同。

若必须保持物理CLI专属载体，继续保留CLI-only安装blocked，调查版本/加载根或CLI插件规则。项目本地规则可作为手动诊断/配置备用，不能自动顶替全局安装目标。正式替代方案尚未采纳；本轮只完成诊断与可审查候选。

## 最终验证

10项文件安全测试通过；新增严格零工具regressions通过，前序四份日志严格复核通过。独立预检两轮，最终approved，范围为具体工具及操作边界；实际native观察由协调者取得。修订后Nix shell内install.sh退出0，新装AskHuman v0.13.2验证通过。私有原始数据和创建receipt保留，UUID仅为本地夹具目录标识。未提交或发布。

## 后续人类决定：家族共享（2026-10-04）

人类明确批准“按家族共享、按客户端验收修订计划”。此前CLI-only隔离是规划默认，已被该决定替代；共享模块正式候选名改为 `askhuman-antigravity.md`，不加仅CLI语义guard。目标是读取共享协议的官方Antigravity本地客户端（CLI、IDE/扩展、2.0）；不宣称当前所有客户端能力已通过。CLI/MCP仍是AskHuman调用方式，身份冲突/无绑定保护不变。此前待采用/保持CLI专属的候选讨论仅为决策历史，当前合同以修订后的 [规格](../specs/antigravity-cli.md) 与 [计划](../plans/antigravity-cli.md) 为准。产品实施和永久安装须另行启动。
