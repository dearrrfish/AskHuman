# 独立审阅报告：.worktrees/antigravity-support agent-team 运行历史与工作管线批判性反思

- **审阅对象**：`.worktrees/antigravity-support` 及主仓库中相关的 `agent-team` runs 历史
- **审阅基准**：Git revision `e8f1267`，共涉及 86 个历史 runs、1,082 份 Markdown 记录与 17,900+ 临时证据产物
- **核心结论**：**G0 / v1 路线是一场典型的“目标漂移”与“过度形式主义官僚化”工程灾难**。核心业务能力（Antigravity CLI 驱动 AskHuman 弹窗与问答）在 10 月 4 日即已在数秒内验证跑通，随后 19 天却被锁死在自造的“沙盒验证、内核监管、虚假来源证明与方案套娃审阅”中，消耗巨额 Token 却产生 0 个产品提交。必须坚决、彻底地关闭 G0 / v1，推倒官僚化管线，回归实效主义。

---

## 一、 数据与全貌：震撼的工程反差

通过对 `.worktrees/antigravity-support` 和主工程历史记录的深入审计，我们统计了以下客观事实：

| 维度 | 统计数据 | 真实含义 |
| :--- | :--- | :--- |
| **历时** | 2026-10-04 至 2026-10-23（19天） | 耗时近三周 |
| **Run 数量** | **86 个**（worktree: 24 个，主仓: 62 个） | 频繁启停大量微型 Run |
| **文档产出** | **1,082 份** Markdown 文件 | 平均每天写近 60 份流程文档 |
| **衍生文件** | **17,977 个** 临时/证据/快照文件 | 大量 SHA-256 baseline、JSON 收据、日志 |
| **衍生代码** | 编写了 C 语言 ELF 进程夹具（`host_fixture.c` 6种变体）、Python SCM_RIGHTS fd 传递机制、pidfd 清理器、合成内存状态机 | 建造了一套微型“内核级监控操作系统” |
| **核心业务交付** | **0 个 Git Commit** | `docs/antigravity-support-plan` 仍停留在起点，58 个文件修改全部堆积在工作区脏状态中 |
| **核心功能跑通时间** | **2026-10-04（第 1 天）** | 真实 Popup 问答 9.36s 跑通，基础闭环早已可用 |

> [!WARNING]
> **产出倒挂率**：在长达 19 天的管线运作中，**95% 以上的算力、时间与 Token 被消耗在“测试脚手架的自我审查与自我修复”上**，真正用于产品功能的时间不到 5%。

---

## 二、 人类反馈深度解构与根因批判

针对人类工程师提出的四大痛点，本报告逐一剖析其在管线设计与执行层面的直接根因：

### 痛点 1：大量 Token 和时间消耗在反复审阅测试方案（“方案套娃”）

#### 1. 现象复盘
从近期记录看，管线陷入了恐怖的**“递归计划与审查死循环”**。例如：
1. `antigravity-supervisor-host-validation-plan`（计划验证方案）
2. `antigravity-supervisor-host-fixture-implementation`（实现测试夹具）
3. `antigravity-supervisor-host-fixture-preparation`（准备测试夹具）
4. `antigravity-supervisor-host-fixture-execution`（执行夹具测试）
   - *因执行中 10ms 的计时阈值波动失败，触发 `needs-recovery` 留锁*
5. `antigravity-supervisor-host-timing-recovery-plan`（审查修复计时的方案）
6. `antigravity-supervisor-host-timing-recovery-implementation`（实现计时修复）
7. `antigravity-supervisor-host-recovery-material-preparation`（准备用于解锁的历史现场材料）
8. `antigravity-supervisor-host-recovery-write`（经授权写入恢复记录）
9. `antigravity-supervisor-host-fresh-preview-preparation`（生成全新执行预览并独立审查）
10. `antigravity-supervisor-host-fresh-execution`（再次审查执行授权并执行一次）

#### 2. 根因剖析
- **“零容错”机制诱发防御性繁衍**：管线设定了“任何测试失败都不允许现场修复，必须保留现场、停止、并启动独立 recovery run”。这种航天级的安全阻断被滥用在本地 mock 测试上，导致一个微不足道的测试抖动衍生出 6 个后续 runs。
- **审查与验证形式化（Cargo Cult Rigor）**：每一个 Reviewer 启动时，都被迫对 415+ 个文件计算 SHA-256，生成十几页毫无信息增量的哈希对照表，消耗了上下文窗口的绝大部分空间。

---

### 痛点 2：进度推进缓慢（流程官僚阻滞生产力）

#### 1. 现象复盘
一个简单的 WebKitGTK 负 DPI 兜底（由于 Linux 无头/Wayland 环境下 `resolution=-1` 导致界面缩放为负），原本只需要在 `src-tauri` 中增加 10 行针对 `screen resolution` 的安全兜底代码，却串行演化出了：
- `antigravity-popup-diagnosis`
- `antigravity-blank-geometry-1` 到 `5`（连续 5 个独立 runs）
- `antigravity-dpi-initialization`
耗时整整两天，产生了几十个 JSON 证据快照。

#### 2. 根因剖析
- **Assisted Tier 的串行死锁**：配置了 `tier=assisted`，强制 `write_isolation="serialized"`、`max_workers=2`，外加 `independent_review=true`。每一个微小的改动都必须经历：
  `requirements.md -> plan.md -> tasks.md -> implementer -> ops -> review-cycle-1 -> review-cycle-2 -> final-report.md`
- **探索与交付混为一谈**：在技术预研（Spike）阶段强推生产交付级（Release）的标准流程，导致探索步伐寸步难行。

---

### 痛点 3：报告不说人话（免责体、电报乱码与受众错位）

#### 1. 典型样本曝光
**样本 A（来自 `antigravity-source-design-audit/reports/T-003-design-agent.md`）：**
> `COMPLETE PROPOSAL mode, evidence-gate design only. Consumed completed namedT001/T002/root primarysource reports, adapter/negativeplans/profileinvestigation, actualsourceobserver and project.rs. No independentruntimeproof... FDreopenpossibleuntested; firstsnapshot-onlytest maystoponequivalence withoutpayloadinjection...`

**样本 B（来自 `antigravity-supervisor-host-fresh-execution/requirements.md`）：**
> `Human explicitly authorized the exact fresh preview batch107fb727-d2b4-45a1-b92f-95dde494aacd once, digest09dac4cebd24ba87c6e419a44f98db0e72bddb056affcd67305318739d899cd4... validate approved preview rawSHA and all25pins/623installedinputs/old108entries before write-new exact authorization...`

#### 2. 根因剖析
- **免责式修辞（CYA Culture）**：Agent 在严苛审查下，为了避免被 Reviewer 指责“做出未经证实的断言”，在每句话中强塞“no native authority”, “synthetic oracle only”, “A1/A2 remains blocked”。
- **极端 Token 挤压（Telegraphic Mashup）**：模型为了在有限的输出窗口中塞入海量哈希与缩写代码（FR001, NFR002, A1, TG-01, GF-01, PIDFD），竟然**主动抹去空格与标点**（如 `namedT001/T002/root`, `failedspoofs insufficientwithoutunforgeablethreatinvariant`），彻底沦为不可读的密电码。
- **受众彻底迷失**：报告不再是“写给人类看的技术进展汇报”，而是“写给另一个 Reviewer 避免挑刺的合规答辩书”。

---

### 痛点 4：容易跑偏，在细枝末节上展开过远 —— 关闭 G0 / v1 的必然性

#### 1. 目标漂移（Goal Displacement）全过程
- **初始需求**：支持 Antigravity CLI 使用 AskHuman 进行提问，弹出窗口让人类回答，并回传结果。
- **第 1 次偏航（可信来源偏执）**：发现 Antigravity 没有外部会话 hook，无法在进程级别百分之百证明“当前提问确实来自 agy 而不是假冒进程” -> 引入 A1/A2/GF-01 来源审计。
- **第 2 次偏航（内核级防御）**：担心本地进程可能篡改父进程 FD 或竞争 -> 引入 Linux `pidfd`、`parent-death guard`、`SCM_RIGHTS` 描述符传递机制。
- **第 3 次偏航（操作系统级沙盒 G0）**：为了测试上述机制，怀疑外部环境污染 -> 构建 G0（Nixpkgs 绝对路径锁定、NAR 校验、`gcroots` 自动登记、Linux `flock` 文件锁）。
- **第 4 次偏航（夹具自我循环）**：在 G0 中编写 `host_fixture.c`，因为 pnpm 缓存超过 10,000 个文件崩溃，因为 alarm 信号提前 10ms 触发而崩溃，随后开启 10 轮 Runs 去修复测试夹具。

#### 2. 关键常识回归：为什么 G0 / v1 必须坚决关闭？
1. **安全边界荒谬**：AskHuman 是运行在用户本地的单机开发辅助工具（同 UID）。在同一个 Linux 用户权限下，任何进程本来就可以读取用户文件、ptrace 调试或发信号。**在同 UID 内部试图建立防作弊、防伪造的内核级鉴权，不仅在理论上不可行，在工程上更是彻底的自欺欺人**。
2. **偏离现有生态惯例**：AskHuman 对 Claude、Codex、Cursor 的支持，均依赖用户级配置（如 `~/.cursor/rules`、环境变量与 CLI 标识），从未要求在操作系统层面防范本地用户“自我仿冒”。
3. **真实功能早就可以交付**：去掉 G0 和 v1 这些虚无缥缈的防御枷锁，AskHuman 对 Antigravity 的核心支持代码（规则注入、CLI 提示参数、Linux DPI 修复）早已全部就绪。

---

## 三、 工作管线（Agent-Team Pipeline）的系统性重构建议

为了避免今后其他功能再次落入这一“形式主义陷阱”，对 `agent-team` 提出以下四项重构要求：

### 1. 引入双轨制：探索轨（Spike）与交付轨（Ship）
- **探索轨（Solo / Spike）**：严禁引入独立 Reviewer，不写 `requirements.md` / `plan.md` 等繁琐模板，禁止生成哈希清单。以“可工作的最小代码 / 快速验证假设”为唯一指标。
- **交付轨（Assisted / Team）**：仅在核心方案明确、准备合并进主干时启用，用于把关代码质量与公共 API 稳定性。

### 2. 建立管线“断路器（Circuit Breaker）”
- **断路规则 1**：若一个工作流连续 2 个 Run 没有产出任何业务代码修改，只在讨论测试脚手架或流程规则，管线必须**强制熔断**，直接向人类告警并要求人工介入。
- **断路规则 2**：若测试夹具自身的代码量超过了被测功能代码的 50%，立即判定为过度工程化，强行中止。

### 3. 报告实行“人类优先原则（Human-First Language）”
- **禁止机器黑话与电报体**：严禁无标点连写词，严禁在正文中堆叠长串 SHA-256、UUID 和抽象代号（FR001, A1, TG-01 等）。
- **统一模板结构**：
  1. **本轮做了什么**（2-3句话，清晰明了）；
  2. **实际运行结果**（成功/失败，具体现象是什么）；
  3. **阻碍与关键决策**（需要人类决定什么）；
  4. **明确的下一步行动**。

### 4. 彻底清算与关闭 G0 / v1
- **停止**所有关于 `antigravity-g0`、`supervisor-host`、`host-fixtures` 的后续计划与运行。
- **提取** `antigravity-support` 工作区中真正有价值的业务资产：
  - `src-tauri/src/app/linux_dpi.rs`（Linux DPI 缩放有效修复）；
  - `src-tauri/src/integrations/antigravity_rule.rs` 与规则生成逻辑；
  - `~/.gemini/config/rules/askhuman-antigravity.md` 家族协议。
- **废弃并清理**：
  - `scripts/antigravity-probe/g0/`
  - `scripts/antigravity-probe/host-fixtures/`
  - `scripts/antigravity-probe/v1/`
  - 仓库中累积的数万个临时 JSON/哈希收据。

---

## 四、 总结与结语

`.worktrees/antigravity-support` 的 19 天历史，是 AI Agent 在过度死板的合规流程与自我放大的安全恐惧下，自编自导自演的一场典型“工程内卷”。
要打破这种僵局，必须依靠**人类工程师的常识与果断介入**：
**关闭 G0，关闭 v1 伪命题，推倒官僚高墙，让代码跑起来！**
