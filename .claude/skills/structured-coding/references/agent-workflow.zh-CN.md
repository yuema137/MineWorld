# Agent workflow

[English source](agent-workflow.md) · 英文是唯一权威源，本页是中文镜像。

本文详细解释操作流程。完整 planning 和 execution 要求以 prompt 为准。按 `SKILL.md` 的指引读取当前阶段所需资源；不要用本文替代完整 execution prompt。

## 1. 确定当前工作

检查用户请求和仓库已有的 planning 约定。查找已有的 overall、step 和当前 PR 文档。在判断旧 checkpoint 是否仍有效之前，检查 branch、HEAD、working tree、相关历史和正在进行的工作。

确定用户要求的阶段：overall planning、step planning、PR design/freeze、execution、同一 PR 的恢复、operator review 或 merge 后更新。简要说明所选阶段。利用已有可用产物继续，避免重新生成整套层级。

每个 implementation context 只保留一个 active PR 身份。记录 repository/worktree、PR 标识、branch/base、primary design、parent 文档及填写好的 execution contract。

把这些文档统一放在 `.structured-coding/plans/` 下面，每项工作一个目录，这样并行的工作不会互相冲撞，也不会污染项目自己的 `docs/` 目录：

```text
.structured-coding/plans/infra-exp-p0/overall.md
.structured-coding/plans/infra-exp-p0/step-01-user-map.md
.structured-coding/plans/infra-exp-p0/pr-01a-proposer.md
.structured-coding/plans/infra-exp-p0/pr-01a-contract.md
.structured-coding/plans/infra-exp-p0/handoff.md
```

这是默认做法，不是硬性要求。项目如果已经把规划文档放在别处，就继续放在那儿；真正要紧的是只有一个位置是权威，而且后续 session 找得到它。

规划文档属于开发产物，默认**不进版本库**，和个人的 standards 覆盖文件一起：

```text
.structured-coding/plans/
.structured-coding/standards.local.md
```

共享的 standards 文件是刻意的例外：`.structured-coding/standards.md` 是项目配置，而"git 有没有 track 它"恰恰就是它成为团队标准而不是某个人标准的判据，所以它应该进版本库。把 plans 留在本地的代价是队友从全新 clone 里读不到它们，而同一台机器上的后续 session 仍然找得到。如果对你的项目来说这个取舍反过来更合适，就提交它们。

不要创建与 primary PR ledger 争夺权威的额外跟踪文档。contract 可以是 PR design 的一节，也可以是单独链接的文档。handoff 用于续接工作，不是另一份 design authority。

## 2. Overall planning：确定最终目标

与 operator 一起明确最终需求、主要使用场景、可观察的项目结果、大方向实现、scope/non-goals、主要风险和 step 边界。通过检查仓库为架构判断提供依据。

描述 module 和主要能力，把文件与函数细节留给实际实现它们的 PR。解释有意义的取舍；现有证据无法确定的产品或方向问题，需要询问 operator。等待决定时，继续独立的 audit 和起草工作。

operator 认可 overall 之前、第一个 PR freeze 之前，这份文档必须覆盖整个选定的 effort：

1. 完整的 effort 及其可观察的产出，不要把 operator 还没批准的后续阶段拉进来。
2. 当前能识别出来的每一个必要的高层 step，各自写清预期产出、大致依赖或顺序、验收检查点。低层实现细节可以先不定。
3. operator 要求的每一项产出，都要对上一个 step，或者对上一个明确"尚未决定"的条目。某项需求要退出这个 effort，只能通过明确的 scope 决定——绝不能因为它装不进第一个 PR 就消失。
4. 依赖后续发现才能定的工作，要写成条件 step 或决策点，而不是省略掉。audit 有可能证明某个设想的改动其实不必要；那就把这个证据、以及它需要的 scope 共识记下来。
5. 当前这个 PR 覆盖其中哪一部分，做完之后还剩什么。

这是对路线完整性的检查，不是要你去编还没做过 audit 的架构、也不是规定 step 的最小数量。真的只需要一步的 effort，就还是一步。

采用适合项目的文档形式。operator 理解并同意交付内容和大方向后，overall doc 才算准备好。不能把沉默当成同意，也不能因为已有 overall doc 就开始改代码。

## 3. Step planning：选择有意义的 PR checkpoint

读取已商定的 overall 方向，检查相关 subsystem，确定当前 step 需要一个还是多个 PR。

如果需要多个 PR，描述各自的 medium scope：可能涉及的文件或文件组、彼此关系、依赖和不包含的后续工作。为每个 PR 定义有意义的 integration checkpoint、可观察的通过条件和 adversarial criteria。不要只按文件数或任意大小拆 PR。

如果只需要一个 PR，先确定 step scope，再将同一文档原地扩充到 PR 级细节。标明其兼任 step/PR 的角色，并直接链接 overall doc。不要维护同一计划的两个独立副本。这是关于单个 step 的文档排布规则，它跟 overall 有几个 step 无关。

后续 PR 保持 medium scope，直到前面实现提供的证据足以支撑详细设计。已知依赖如果推翻后续 step，应立即指出，即使当前只会完整细化下一个 PR。

## 4. PR design：先 audit，再制定具体步骤

完整读取 [PR requirements](../prompts/pr-design-requirements.md)、[test rules](../prompts/test-ci-gate-rules.md)，以及 [working rules](../prompts/implementation-working-rules.md) 中的项目专属 contract。

按需 audit 相关代码、caller、下游 consumer、test、schema、保留证据和历史。design 应说明：

- PR 的 goal、已批准 scope、non-goals、invariants 和 parent 依赖。
- Source audit：实际检查的文件和 symbol、发现及未解决的假设。
- 最终可观察的 acceptance，以及 integration/adversarial checkpoint。
- 按保留要求中全部八项内容编写的有序 commit plan。
- Test ownership：各 failure class 由 static checks、Unit、真实 LLM Gate、真实 lifecycle Gate 或最终 CI 中哪一层负责；明确标注不适用的层。
- Execution base、validation budget、发布权限、stop condition 和 merge 边界。

Implementation item 应是具体 atomic operation；audit 支持时细化到文件和函数。不要编造只有实现过程中才能确定的底层步骤。

每个 commit 分别跟踪可勾选的 implementation、validation 和 LLM review。核对实际仓库路径后，可以参考：

```markdown
- [ ] Implementation: thread the new option through the audited caller chain.
- [ ] Validation: observe the selected item sequence under a fixed input.
- [ ] Review: inspect every caller for a silently dropped option or stale default.
```

Validation 为指定 claim 提供确定性或实证 execution evidence。Review 通过 LLM 分析逻辑、contract、consumer 和遗漏场景。完成其中一个不代表另一个也完成。实现工作的 agent 可以承担 review；是否使用独立 agent review 取决于用户工作流和已有授权。

只有对应工作完成且记录证据后，才能使用 `[x]`。如果某项不适用，记录 `N/A` 和 audit 得到的原因，不要声称执行过 test 或 review。

## 5. Freeze 已批准的 design

与 operator 迭代 PR design。依据原始模板准备填写好的 implementation contract，并保留 working rules 正文。必填信息如果未知，就保持未解决，直到 audit 或决策明确；`N/A` 仅用于确实不适用的内容。

contract 必须区分每次 Gate 的限制和总 runtime/cost 范围，并明确 commit、branch 发布、PR 更新和 validation 是否已授权。沿用已有 session 授权，不要重复询问已经作出的决定。

freeze 之前，把这个 endpoint 块跟 operator 实际说过的话对一遍。整件事约定的流程是做到 PR review-ready，而块里却停在本地文档和测试——这是需要提出来的不一致，不是一个可以直接 freeze 的保守默认值。反过来同样是不一致：planning-only 或明确只在本地做的请求，不会因为出厂默认值有发布权限就获得发布权限。每一处收紧都需要一个 operator 认得出来的 source；没有 source 的那一行是 unresolved，不是已决定。

operator 批准具体 design 进入 implementation 后，记录清晰的 header，例如：

```markdown
# PR 01a — <title>

## DESIGN FROZEN

Design revision: <stable revision or fingerprint of the approved design sections>
Approved by / evidence: <operator approval reference>
Implementation base: <branch and exact commit>
Execution contract: <section or relative path>
Lifecycle: FROZEN
```

这是文档约定，不是密码学授权机制。不能编造 approval，也不能仅因 agent 完成草稿就把 design 标成 frozen。

Freeze 语义需求和 acceptance，同时允许 live ledger 持续写入。整份文件的 hash 不适合作为永久 freeze 身份，因为证据和进度会变化。未来的 hook 必须区分 frozen 部分和 live 部分。

## 6. 准备 fresh execution session

用填写好的项目 contract，加上完整原始 working rules 和 test rules 准备 kickoff。kickoff 可以引用文件并要求完整读取；不能用简短转述替代规则。每个新 PR 由 operator 开启 fresh implementation session。

开始 implementation 编辑前，新 session 必须：

1. 检查 branch、HEAD、status、近期历史和相关运行中的 job。
2. 完整读取 PR design、填写好的 contract，以及所需的 binding parent 文档。
3. 核对已批准的 design 身份、implementation 授权、base 和已 merge 的 prerequisites。读 contract 里的 endpoint authority 并照它执行；source 是 unresolved 的那一行要提给 operator，不要自己再收紧。
4. 完整读取两个 execution prompt 文件，并检查第一个 milestone 的 source/test。
5. 按原始 contract 要求的字段，为当前 PR 初始化 handoff。

仅在指定的 workflow handoff 中归档或替换旧 PR 状态。保留无关笔记和工作。现有编辑发生重叠时，可以使用独立 worktree；不要丢弃这些编辑。

如果 agent 不能开启 fresh session，应产出完整 kickoff 并说明 execution 尚未开始。给 planning 对话换个状态标签，不会让它变成 fresh session。

## 7. 执行每个 semantic milestone

编辑前重新读取当前文件的相关部分。在 frozen contract 内实现，检查相关 consumer，运行成本最低且充分的 validation，完成计划中的逻辑 review，并在每个 checkpoint 完成时记录证据。

整个过程中保持 primary design 最新。发现新情况时立即记录，包括失败假设和失败尝试，而不只记录最终成功的实现。记录足以解释选择及原因的决策依据；私有内部推理不是交付物。

每个 semantic commit 前，检查准确的 diff 和 staged files，确认 scope，记录 test 和 deviation，并同步 ledger。随后自主 commit 并继续。为了便于 review，一个计划中的 commit 可以拆成多个连贯的 commit；记录它们与原计划的对应关系。

明确启用 checkpoints 时，在选择 commit 命令前运行 `checkpoints.py inspect`，按 [preset interface](checkpoints.md) 完成语义同步，再刷新机械 checkpoint。

在 semantic milestone 和重要 checkpoint 变化时刷新 handoff。通常最多落后一个 milestone。记录后台 job 标识、log/artifact 路径、预计 runtime 和下一步，防止恢复后的 session 重复启动工作。

### 不确定性与 deviation

先检查 design、实际 source、consumer、test/evidence 和相关历史。外部技术事实仍不确定时，查阅权威外部来源。在 PR ledger 中记录对工作有影响的外部结论及其来源。

如果答案仍处于已批准意图内，自主推进，并记录发现、可选方案、决定、理由、影响和 validation。普通 bug、额外 caller、小型 helper、更合适的 test seam，不自动构成 operator 决策点。

如果正确解决方案需要改变 frozen invariant、public contract、ownership 边界、重要 dependency/architecture 或 scope，准备证据及最小可 review 的修订建议。暂停依赖该决定的动作并取得实质决策；有价值时继续不受影响的已授权工作。

不要为修复无关仓库 bug 无限扩大 PR。确认这些问题与当前改动的关系，记录重要 follow-up；当前 acceptance 不受影响时继续。

### Validation 与证据

使用完整 test rules。在昂贵运行之前明确 claim 及其 owner。真实 Gate 应保持真实且有界；根据实际证据将每次结果分类为 `PASS`、`FAIL` 或 `INCONCLUSIVE`，不能只看 exit code。

按需记录 command、test count、wall time、environment、artifact 和决定性观察。未运行的 test 不是 pass。不能削弱 assertion 或伪造 expected value 来得到绿色结果。

将证据关联到实际测试状态。对于 commit 前在 working tree 中运行的验证，记录 base HEAD、diff/untracked-content fingerprint 和相关配置。确认被测试的可执行内容一致后，才能把证据关联到后续 commit。HEAD 本身无法描述未 commit 的改动。

保留测量证据和 frozen parity reference。把持久证据存到项目约定的位置，并从 design 链接；临时 log 本身不能充当 handoff。

## 8. Compaction 后恢复同一个 PR

Compaction 不重置进度，也不初始化新 PR。通过 handoff 恢复 active PR，对照 repository 身份和 branch/base 核验，并按来源权威顺序处理：repository/git/process 事实 → primary design → binding parent 文档 → handoff → emergency snapshot → 对话记忆。

重新完整读取当前 PR design、填写好的 contract 和两个 execution prompt。只出现在 handoff 里、没有 source、contract 里也找不到依据的限制，要报告而不是遵守；operator 新的明确指示优先于它。启动替代任务前先检查 active process 和 CI run。重新打开下一步所需的 source seam，在编辑前核对过时的 checkbox 或 evidence 声明。

计划中的 manual compaction 应先同步 design、handoff、HEAD 和 working-tree fingerprint。不可避免的 automatic compaction 遇到过时的语义 handoff 时，应产生机械恢复 snapshot 和警告，不能造成 compaction deadlock。hook 不得编造 semantic summary。

事件行为与验收场景详见 [hook contract](hook-contract.md)。未明确设置 hook 时，这些仍是流程要求。可选的 [continuity preset](continuity.md) 只提供机械 compact 同步检查、snapshot 尝试和恢复指令，不强制语义恢复，也不拦截 implementation/merge。

## 9. 达到 operator review 条件

完成 implementation、计划中的 review、ledger 更新和必需 validation。commit 预期最终内容，执行已授权的发布与 PR 工作，检查 PR diff/body，并跟踪准确最终 PR HEAD 的 canonical CI。修复普通失败，push 新 head，再核对新 CI 证据。

启用 checkpoints 时，在撰写 handoff 前调用 `prepare-review`，取得检查清单并设置一次面向 operator 的提醒，不自动续跑。它不能替代实际完成条件的证据；用 `cancel-review` 取消待发送的提醒。

operator 可以在 CI 运行时 review。不能只因 PR 已存在或 CI 已启动就结束自主 execution。不能把旧 head 的绿色 CI 当成当前 head 的证据。

准备 working rules §22 列出的完整 handoff：PR/base/head、语义改动、发现与 deviation、文档更新、各层真实证据、限制和 working-tree 状态。final executable HEAD 与 final PR HEAD 不同时应分别记录。

只有满足 contract 要求时，才将 lifecycle 设为 `READY FOR OPERATOR REVIEW`。将 implementation context 标记为 `CLOSED / AWAITING OPERATOR ACTION`；这表示自主运行结束，不表示 PR 本身关闭。没有明确 operator 授权不得 merge。

避免 final SHA 的自引用循环：commit 不可能包含自己的 hash。适当情况下，将最终 CI/head 指针放在指定 handoff 或 PR metadata 中；所有影响 acceptance 的 design 内容应在最终 CI 前纳入版本。不能新增影响 acceptance 的 commit 后，声称上一个 head 的 CI 对它有效。

review 要求修复时，明确恢复同一个 PR 的工作、核对状态并重验受影响的 claim。它仍属于同一个 PR context；fresh context 的要求适用于下一个 PR。

## 10. Merge 与 backward update

取得明确 merge 授权后，确认授权目标，核验当前 head 和必需检查，再执行已授权的 merge。不能假设旧 candidate 的批准涵盖实质变化后的 candidate。确认真实 merge 结果；不能凭计划或队列中的操作就标记 merged。

确认 merge 后：

1. 将 PR 文档更新为 `MERGED`，记录真实 merge commit/result。
2. 向 parent step 回写已完成能力、实现事实、已达成 checkpoint 和剩余工作。
3. 向 overall doc 回写进度，以及重要方向或依赖影响。
4. 根据 merged code 和新发现，重新 audit 并细化紧接着的下一个 PR。在相应层级标注更广的影响，不必详细重写所有后续 PR。

PR merge 了，只说明这个 PR 完成了。它不代表对应的 step 或 overall 完成了：把剩余工作记下来；只有当所有列出的 step 都交付、或者被 operator 明确的 scope 决定去掉之后，overall 才算完成。如果已合并的工作暴露出 overall 里没有列的必要 step，就在它自己的层级上补上——已经 freeze 且有边界的 PR 保持原边界，剩下的工作单独规划，不要塞进正在进行的 PR。

同一文档兼任 step/PR 时，只更新一次，然后更新 overall doc；不要虚构独立 parent 或产生自引用。

这四件事各有归属，而归属是记下来的，不是靠推断。默认：action 1 由 implementation session 负责，因为证据在它手里；action 2 和 3 由 contract 里写明的 synchronization owner 负责——默认是 planning session，因为共识在它手里，而且它要拿这些记录去设计下一个 PR；action 4 属于 planning session。一个对话全包也可以；希望由 implementation 侧回写的项目，在 contract 里写清楚就行。记下来的意义在于：不要两个 session 都以为对方做了。

两者之间的交接是明确的。implementation session 在 PR 文档里把 parent 同步标成 pending 并写明 owner；owner 完成 parent 更新后，在同一处确认自己记了什么。只有 owner 写 parent 文档，另一方向 owner 汇报。parent 里已经往前走了的内容要对账，不能覆盖。负责回写不等于 merge 权限，也不会带来 merge 权限。

implementation 对话可以在满足以下条件后关闭：它的 PR 文档里有 merge 身份、验证证据、偏离、遗留问题和一份可持续的 handoff，并且剩余同步工作的责任已被明确接手。可持续不等于发布出去：把 plans 留在版本库之外的项目，就继续留在外面。

按照仓库既有文档流程更新状态。implementation branch 已关闭或受保护时，准备或使用合适的文档改动途径，不要静默 push 到受保护 branch。未在权威位置记录前，不能声称 parent 同步已经完成。

下一个 PR design 必须取得自己的 freeze/implementation 授权，并在 fresh execution session 开始。其知识来自 merged state 和 binding 文档，不来自上一个 PR 尚未结束的记忆。
