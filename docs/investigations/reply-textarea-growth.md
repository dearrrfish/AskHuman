# 回复输入框循环增高：Wayland / WebKitGTK 调查

日期：2026-10-29。仓库基线：`00480ef2`。初始阶段仅调查；用户随后批准局部修复，实施与验证见文末。

## 结论

已在本机原生 WebKitGTK 2.54.1、Wayland 下复现完整机制：**未换行的短文本每次触发 autosize，输入框也会增长约 1 CSS px；达到 240px 上限后，下一次允许收缩的输入重新测量内容，使输入框突然缩回。** 退格在触顶之前也会让输入框增长，符合用户观察。

同机、同版本、相同 CSS 和 TypeScript 算法在强制 `GDK_BACKEND=x11`（本机 DISPLAY :0，XWayland 对照）下稳定；headless Chromium 155 也稳定。这足以证明本机的异常与 Wayland/WebKitGTK 的尺寸报告组合有关，不能升级成“所有 Linux/Wayland 都必现”或“其他平台绝不可能出现”。用户报告 macOS 人工使用没有此问题，本轮未另行在 macOS/Windows 实测。

## 用户确认

用户通过 AskHuman 补充：不是随文字或换行增加，退格也会改变文本框；原生窗口大小不变；输入框看似随机恢复原大小；macOS 人工体验无此问题。

## 因果链

1. `AnswerComposer.vue:93` 的 textarea input 进入 `usePopupCore.ts:1429`，调用 `autoGrow`，再进入 `textareaAutosize.ts:79`。
2. `textareaAutosize.ts:35` 对 border-box 使用 `scrollHeight + computed 上下 border` 计算所需高度。
3. 正常情况下，空余高度足够时，scrollHeight 应当与当前内高一致，加回边框就等于外高，继续输入同一行不会增高。
4. 本机 Wayland / WebKitGTK 2.54.1 的测量不一致：当前外高 70px 时，scrollHeight=69px，computed borderTopWidth=1px，borderBottomWidth=1px，算法得到 71px。其结果比当前高度多 1px，大于 0.5px 阈值，写回 71px；下一次得到 72px，持续正反馈。
5. `textareaAutosize.ts:100` 的增长分支先执行且提前 return，即使 inputType 是 deleteContentBackward、允许收缩，也不会进入镜像测量。因此退格在到顶前同样使框长高。
6. 上限为 240px（第 1、93–96 行）。触顶后增长分支不再成立：普通 insertText 会停在 240px；delete、undo、replacement、paste 等允许收缩的输入（第 123–132 行），或者其他默认 allowShrink=true 的调用，才会进入镜像重测（第 110–120 行），将高度恢复到内容实际需要的约 70px。
7. 复位依赖输入类型和调用时机，并非随机计时器。输入框复位时原生窗口不会改变。

## 实测

用项目 `tokens.css`、`base.css`、`controls.css`、`popup.css`，将真实 `textareaAutosize.ts` 经 TypeScript 转译后放入最小 HTML。500px 宽、rows=1、无换行、短 ASCII 文本，每轮逐字增加并调用实际函数。另以固定 abc 连续调用 400 次，隔离字符数/换行影响。

| 环境 | 普通带边框 textarea | collapsible 无边框 textarea |
| --- | --- | --- |
| Chromium 155 headless | 69px 持平，40 次输入无增长 | 38px 持平 |
| WebKitGTK 2.50.4，本机默认后端 | 约 68.333 / 69.333px 小抖动；未复现持续累加 | 38px 持平 |
| WebKitGTK 2.54.1，原生 Wayland，应用实际链接的库 | 约 68.333 → 70 → 71 → … → 109px（40 次） | 38px 持平 |
| 同一原生 WebKitGTK 2.54.1，强制 X11 | 69px 持平，40 次输入无增长 | 38px 持平 |

固定 abc、默认 rows=2、allowShrink=true 的 400 次调用中，初始约 89.333px，第 151 次发生 `240 → 70px`，第 322 次再次 `240 → 70px`。allowShrink=false 的对照增长到 240px 后保持，不复位。

原生 Wayland 的真实 WebDriver 键盘输入也记录到：insertText 连续增高；随后五次 deleteContentBackward 依次 `114→115→116→117→118→119px`。因此并非只在程序直接改 value 的同步循环中发生。WebDriver 输入产生了额外事件，本文不将精确事件总数作为断言。

## 环境与证据边界

- 当前 AskHuman PATH 在调查中由旧构建 `581a3ba` 更新到 `66e5db1`（不是本调查执行的安装）。`ldd` 确认新入口实际链接 `/nix/store/mhq9kk703lrphqxjqzphva99f1qa5s0c-webkitgtk-2.54.1+abi=4.1/lib/libwebkit2gtk-4.1.so.0`；最终原生对照直接使用该 store 的 WebKitWebDriver。
- 最初误选的另一份 2.54.1 是经 qemu-aarch64 运行的构建；随后用实际原生库重做 Wayland/X11 对照，结论一致，最终结论不依赖模拟架构。
- 环境 XDG_SESSION_TYPE=wayland，WAYLAND_DISPLAY=wayland-1，DISPLAY=:0，GDK_DPI_SCALE=1.500000。额外设置 GDK_DPI_SCALE=1/GDK_SCALE=1 没有改变 Wayland 最小复现，但未调整合成器输出缩放，不能当作真实 100% 显示缩放对照。
- 2.50.4 WebDriver 后续真实按键/异步尝试发生 page crash or hang；它的同步测量成功，不能据此给旧版完整兼容保证。
- 当前证据确定的是 DOM/CSSOM 几何报告与算法反馈的交互；没有定位 WebKit 上游首次引入该差异的提交，也没有证明具体是 WebKit、GTK 或合成器哪层负责底层尺寸舍入。
- macOS 使用 WKWebView，Windows 使用 WebView2；没有对应实机矩阵，不能仅凭 Chromium 对照宣布 Windows 完整验收。

## 为什么现有测试没发现

执行 `pnpm exec vitest run src/views/popup/textareaAutosize.test.ts` 返回 0；实际配置运行了 7 个文件、49 个测试，全部通过。但 autosize 单元测试 stub 了 scrollHeight/getBoundingClientRect，不包含真实 WebKitGTK 下带边框控件的尺寸不一致，因此不能覆盖本缺陷。

## 修复方向

优先让增长判断也使用独立内容镜像测量，避免用已经被撑大的 live textarea 的 scrollHeight 继续回灌高度。已有镜像机制可复用；需要关注测量成本、窄宽换行、删除/输入法和停靠状态。不要简单靠增加阈值或统一减 1px：误差随引擎和缩放变化，可能掩盖真实增长。

修复验收至少包含真实 WebKitGTK Wayland 与 X11：短文本反复插入/退格、超过 240px、多题无边框、停靠 120px、换行/删行、中文输入法及分数缩放。功能变更后须按 AGENTS.md 运行安装脚本，再使用新安装 AskHuman 验证。本轮未实施修复。

## 修复投入与是否值得修复

建议修复，优先级为普通体验缺陷（P2），适合小补丁，不需要紧急发布。它不会损坏答案或改变窗口，但在受影响环境里每次输入/退格都可能触发，影响的是核心回复动作；根因已可稳定复现，后续定位成本很低。

**估算（非承诺，假设现有 Linux 构建环境可用）：**

- 局部实现和针对性回归测试：约 0.5–1.5 小时；预计主要改 textareaAutosize.ts 及其测试，不需要重构 composer。
- Wayland/X11 真实浏览器验证、安装编译、安装后回复验证：约 1–2 小时，编译等待随机器缓存变化。
- macOS/Windows 针对性回归如有现成机器：约 0.5–1 小时；没有机器时这是验收依赖，不能把模拟通过当作替代。
- 完整交付按约半天工程量安排；若扩展成常驻多引擎 CI、通用布局框架或追查上游提交，投入会显著增加，不建议将其纳入本修复。

**对管线的影响：**

函数只由 Popup 的 usePopupCore 调用，也用于焦点、切题、停靠/回位和语音文本更新。因此不只要测打字，还要保持这些尺寸校正行为。它不参与答案序列化、IPC、daemon、渠道抢答、历史和提交判定；不需要协议升级或数据迁移。风险主要集中在输入手感：每键镜像创建/样式复制/布局读取的开销，以及改高导致滚动位置、焦点与选区跳动。现有设计明确禁止让 live textarea 暂时经过 height:auto，修复必须继续保留这一约束。

**推荐实施范围：**

先保留现有廉价增长探测，在准备写入更大高度时用独立镜像确认真实内容所需高度；已有的收缩测量可复用。这能把修复限制在 helper 内，避免直接把所有输入改成无条件创建镜像。它仍需用真实浏览器验证：不能把这个候选方案当作已经测试通过的补丁。若实现分支变复杂，再选择统一镜像测量并量一下高频输入成本。不要优先加 Wayland 特判、硬编码减1px、调整全局 CSS 或修改窗口布局。

投入判断：值得花半天完成一个有边界的修复；不值得为这一缺陷建立大型跨平台布局改造。若短期无发布窗口，可随下一个常规版本带上。临时强制 X11 只是已验证的最小复现绕过方向，真实应用的拖放、缩放、焦点副作用本轮未验收，不建议为了这个问题默认改变整个应用后端。

## 局部修复实施

用户批准后，在 `fix/reply-textarea-growth` 分支修改 `textareaAutosize.ts`：保留普通插入的无需增长快速返回；疑似增长也先用隐藏镜像确认，再写一次最终高度。允许收缩的输入直接完成镜像重测，不再被虚假增长的提前 return 拦住。未改 IPC、窗口、提交或渠道逻辑，也未加平台特判。

新增回归覆盖 live scrollHeight 与 border 合计比实际高 1px 的重复调用（允许/禁止收缩各一组），以及退格在 live 尺寸误报增长时仍能收缩。真实 WebKitGTK 2.54.1 Wayland/X11 对照：短文本交替 allowShrink 连续400次无累积漂移；带边框/无边框 × 普通/停靠的四组布局均保持短文本稳定，长文240px/120px上限正确，删除恢复短文本高度。真实键盘 insertText/deleteContentBackward 不再逐次增高；默认 rows=2 的初始控件可按既有 allowShrink 语义在删除时缩为单行内容高度，这属于正常内容重测。

一次本机微测量（300次、最小页面）中，受影响的普通带边框框每次镜像确认约2.24ms，无边框框保留快速路径约0.007ms。这不是完整输入延迟或严格性能基线，仅证明未见数量级异常；低性能机器及长文本仍应关注。

针对性4文件43测试通过。第一次完整套件1072通过、1项新测试因并行负载5000ms超时；减少模拟循环次数（浏览器保留400次）并限制并发重跑。初始不加exclude的测试命令还扫描了嵌套工作树及pnpm-store，出现Vue运行时重复相关失败；最终只验证当前检出，显式排除这两类目录。最终完整套件在 `--exclude '.worktrees/**' --exclude '.pnpm-store/**' --maxWorkers=4` 下90文件1073测试全部通过；vue-tsc通过。新增插入不收缩约束后，当前helper 11项测试通过。Node脚本测试50通过、22按平台跳过、0失败。`nix-shell -p cargo rustc pkg-config webkitgtk_4_1 gtk3 openssl librsvg --run './scripts/install.sh'` 成功，local-install编译7分21秒并安装至 `~/.local/bin/AskHuman`；脚本按预算清理构建缓存。使用该新二进制发起真实回复测试，用户明确确认「输入和退格正常，未再出现异常增高/复位」。安装检查提示本地与Nix两份二进制共存，本轮未删除或更改Nix安装；真实验证明确使用本地新构建。macOS/Windows本轮未实测。

## 复现资产

同次 assisted run `.agent-team/runs/reply-textarea-growth/` 内保存最小探针、JSON 结果及 worker 报告。该目录通常被 gitignore，仅用于本机审计；本文中的数据和机制作为持久仓库报告保留。
