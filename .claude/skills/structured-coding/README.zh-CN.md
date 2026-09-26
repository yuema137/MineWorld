<!-- Generated file. Source: docs/content.zh-CN.json in the structured-coding repository, built by scripts/build_human_docs.py. Direct edits here are overwritten by the next build. -->

# Structured Coding

[English source](README.md)

先商量好要改什么，让 agent 放手做，再把结果写回计划。

> **什么时候值得用，什么时候不值得**
>
> 这套东西是为大型代码库的持续开发设计的。当一个改动要跨多个 PR 或 session、代码库大到 agent 必须先审计再动手、以及这活儿之后还得有人接手时，它的开销才划得来。改错别字、修一个单文件的 bug、写个用完就扔的原型，它就是 overkill：写计划文档的成本会超过改动本身。这些情况直接用 agent 就行。计划、测试和 LLM review 仍然可能出错；这套 workflow 加上的是把它们的假设和证据写下来，让你能检查。

[用起来是什么样](#example) · [一步一步使用](#tutorial) · [想了解更多](#further) · [可选 hook：安装、覆盖范围和限制](#hooks)

## 你已经装好了

这份指南随 skill 一起安装，所以你现在是在一个已经装好它的项目里读它。明确调用 skill，描述你想做的功能；先要 planning，不要直接要实现。

Codex 的请求开头加 $structured-coding，Claude Code 加 /structured-coding。说清 feature 和约束，先让它规划，不要直接开始实现。

<a id="example"></a>

## 用起来是什么样

三条消息把一个功能从想法带到可以 review 的 PR。你点两次头，中间的活儿由 agent 干。

### 1. 规划 feature

```text
Use the structured-coding skill for this feature. Read its SKILL.md
entrypoint first and load the complete resources its table lists for the
current phase.
First agree with me on requirements, module-level direction, and overall
step boundaries; then detail the current step. Work on planning for now.
Requirements: ...
```

Agent 会问它推断不出来的东西，检查你真实的代码，然后写出总体计划和 step 边界。这一步不写任何实现。

### 2. 准备下一个 PR

```text
Read the overall and step documents, audit the current code, and prepare the
PR 01a design doc and filled execution contract. Use structured-coding:
start from its SKILL.md entrypoint and load what the PR design row lists.
Follow the original PR requirements for the commit checklist. Separate
implementation, validation, and review, and prepare the design for my
approval.
```

你会拿到一份 PR 设计：经过代码审计的 commit 计划，加一份填好的执行 contract。读它、要求修改，等它确实描述了你想要的东西再批准。

### 3. 批准后，用新 session 执行

```text
Execute PR 01a. The approved DESIGN FROZEN document is
.structured-coding/plans/order-flag/pr-01a.md, and the filled contract is
.structured-coding/plans/order-flag/pr-01a-contract.md.
Use structured-coding: read its SKILL.md entrypoint, then the complete
resources its execute row lists, including Implementation Working Rules and
TEST / CI / GATE in full. Reconcile actual state, and begin.
Continue autonomously to READY FOR OPERATOR REVIEW under the contract.
Do not merge.
```

一个全新 session 负责实现、验证、review 自己的逻辑、提交，停在 review handoff。你读 diff，决定要不要 merge。

整个循环就这些。确认 merge 之后，agent 会把学到的东西写回计划，下一个 PR 从那里开始。

## 为啥要用 Structured Coding？

计划能写清想做什么，但光有计划，还没说清 agent 怎么执行、拿什么证据算完成、什么时候找你，以及 context 丢了以后怎么接着干。这个 skill 把这些决定连成一套能反复使用的 workflow。

## 你在哪些地方参与？

不用每次 commit 都点头。但会改变约定的决定，得由你来做。

design 批准之后，agent 自己调查、实现、验证、review、记录并 commit。如果 PR 和 CI 工作已获授权，它也会一并做完，不需要你一步一步催。contract 里记着你授权到哪些 endpoint——commit、push、开 PR、修 CI——merge 永远不在其中。agent 也不许偷偷把这份清单收窄：一个你没要求过的停止点，是要拿来问你的，不是它可以替你采用的保守默认值。

遇到实质性范围变化，或者现有授权以外的操作，agent 带着证据和方案回来找你。

<a id="roles"></a>

## 你、specification、agent 和 hook，到底各管什么？

Specification 是写下来的要求，不是一个盯着所有操作的程序。agent 读这些要求，再结合你的项目执行。Hook 则是 host 在特定事件发生时运行的小程序，比如 compact 或调用工具的时候。它能检查什么、提醒什么，要看我们实际实现了什么，不能光看 specification 里写了什么。

| 谁来负责 | 具体做什么 | 不能替代什么 |
| --- | --- | --- |
| 你 | 你决定要做出什么行为，批准当前 PR design，对实质变化作出选择，并在 review 后明确授权 merge。 | 已经在约定范围内的普通修复、test 和 commit，不用每次再等你点头。 |
| Specification 和 prompt template | 它们规定 PR design 要写什么、agent 怎么执行、什么算有效的验证，以及哪些地方必须有授权。 | 文字规则不会自动拦住工具。Hook contract 还包括未来要实现的要求，不全是已经提供的功能。 |
| Agent | 它读完整的相关规则，检查 code，写计划，实现、运行 test、review 逻辑，并持续更新 design 和 handoff。 | 打完勾或说得很有把握，都不能替代 test evidence 和你的批准。即使没装 hook，agent 也得遵守工作流。 |
| 可选 hook | Continuity 在手动 compact 前检查已记录的 checkpoint，自动 compact 时尝试保存现场，并在恢复时提供读文档的指令。Checkpoints 提供 commit 准备提醒和 review 提醒。 | 它们不能理解每项设计取舍，不能证明 test 通过，不能强制 design freeze，也不能拦住所有 merge。它们不会让 agent 自动一轮接一轮地继续。 |

比如，specification 要求 PR design 里记录验证证据。agent 得真的运行 test，再把结果写进去。Checkpoints 可以提醒它检查缺失的证据，但不能替它认定 test 通过。最后你看结果，再决定是否批准 merge。多装几个 hook，也不能省掉这些责任。

<a id="sessions"></a>

## 到底要开几个会话？

一个方便的安排是：保留一个 planning 会话，每个 PR 再开一个新的 implementation 会话。需求、overall plan、当前 step 和下一个 PR design，可以在同一个 planning 会话里讨论，不用各开一个。工作流要求的是每个新 PR 用新的 implementation context，不是每次 commit 或 test 都换聊天。 这样分开，是因为 planning 里常有被否决的方案和后来改变的假设。新的 implementation 会话从已批准的文件和当前 code 开始，减少把讨论中的旧想法当成最终要求的机会。

| 会话 | 你在这里做什么 | 什么时候切换 |
| --- | --- | --- |
| Planning 会话 | 讨论需求，让 agent 检查 repo、写计划，review 当前 PR design，并批准 execution contract。 | 当前 PR 批准后，让 agent 准备带真实文档路径的 kickoff，再开新的 implementation 会话。 |
| PR A 的 implementation 会话 | 把已批准的 design 和 contract 交给 agent，让它实现、验证、review、commit，并完成已授权的 PR 和 CI 工作。 | 普通修复、commit、compact 和 resume 都继续处理这个 PR，不在这里启动 PR B。 |
| PR A 的 review | 你看 diff 和 handoff。有问题，就在原 implementation 会话里要求修复；满意以后，再明确授权 merge。 | 可以另开 reviewer 会话，但不是必须。修复后，要更新证据，并确认最终 HEAD 的 CI。 |
| PR B 的规划与实现 | 确认 A 已 merge 之后，implementation 对话记录 A 的 merge 身份和证据；synchronization owner——默认就是这个 planning 对话——回写 A 的 parent step 和 overall plan，再拿这些记录去细化并批准 B。 | 做完 A 不等于做完计划：overall 里列出的每个 step 都交付、或者被你去掉之前，它一直是未完成的。你可以回到 planning 对话，也可以开一个替代对话去读已保存的 plans。B 要在另一个全新的 implementation 对话里开始。 |

比如一个功能拆成两个 PR，通常就是三个工作会话：planning、implementation A、implementation B。这是示例，不是硬性数量限制。Planning 聊得太长可以换会话，implementation 中断了也可以恢复。会话之间传递约定，靠的是项目里保存的文档，不是指望另一个聊天自动记得前面的事。 Handoff 就是 agent 保存的接续说明，告诉恢复后的会话：做到哪儿了、什么还在跑、下一步是什么。

<a id="tutorial"></a>

## 第一次使用，照着走完一个功能

下面按实际操作顺序走一遍。假设你的应用会读取文件，你想加一个可选的字母排序模式，同时保留当前默认行为。示例消息里的路径和 PR 编号都需要替换，不是这个 toolkit 已经替你创建好的文件。先让 planning agent 写出真实文档，再把真实路径填进 execution 消息。下面的英文入口消息在中英文页面里完全相同，也不能代替原始长 prompt。

### 1. 开 planning 会话，先说清楚你要什么结果。

在选好的 agent host 里打开目标项目。Codex 用 $structured-coding，Claude Code 用 /structured-coding 调用 skill，然后发送下面的 planning 请求，把需求换成你自己的。

拿这个例子来说，你要说明：输入是 [c, a, b]，启用字母排序后应按 [a, b, c] 读取；不开这个选项，旧行为必须不变。也要说清楚这次不做什么，比如这个 PR 先不改 resume 机制。Agent 应该先检查 repo，把未确定的产品问题问清楚，再提出整体方向。你说的是做计划，它就不能直接开始 implementation。

```text
Use the structured-coding skill for this feature. Read its SKILL.md
entrypoint first and load the complete resources its table lists for the
current phase.
First agree with me on requirements, module-level direction, and overall
step boundaries; then detail the current step. Work on planning for now.
Requirements: ...
```

**进入下一步前，确认这件事**

进入下一步前，你应该能用自己的话讲清楚目标和主要步骤。哪里看不懂，就让 agent 重写到你能 review 为止，不需要你替它写 design doc。

### 2. 让同一个 planning agent 细化下一个 PR。

Overall plan 讲整个功能和主要步骤，step plan 讲几个 PR 怎么配合，当前 PR design 才深入到具体修改。Agent 先读真实 code 和调用方，再写要改的文件、函数、commit 和检查。后面的 PR 可以先粗一些，因为这次 implementation 可能会带来新发现。 具体格式由 PR requirements 管；你让 agent 按完整 specification 准备，不需要自己重新拼一份模板。

字母排序这个 PR，验收要观察 reader 是否真的按 [a, b, c] 读取，不能只检查配置里存进了一个值。你可以要求：如果某个调用方漏传了选项，这个检查必须失败。每个 commit 的 implementation、validation 和逻辑 review 要分别记录。一个 step 如果只需要一个 PR，就把 step doc 直接展开，不用再维护一份重复计划。

```text
Read the overall and step documents, audit the current code, and prepare the
PR 01a design doc and filled execution contract. Use structured-coding:
start from its SKILL.md entrypoint and load what the PR design row lists.
Follow the original PR requirements for the commit checklist. Separate
implementation, validation, and review, and prepare the design for my
approval.
```

**进入下一步前，确认这件事**

Agent 应该交给你真实的 PR design 路径和填好的 execution contract。一起确定这些记录放在哪儿、哪些需要进 Git。私下的 planning 笔记和原始 log，不会因为产生了就自动成为对外发布的内容。

### 3. Review 这份约定，明确批准当前 PR。

你要看清楚：这次改什么、保留什么、不做什么，以及观察到什么结果才算完成。Execution contract 还得写明，agent 能不能 commit、push branch、创建或更新 PR、修复 CI，以及应该停在哪儿。这些是不同的授权。只让它在本地实现，不代表允许它发布到远程。

Design 符合你的意思后，明确批准这份具体的 design 和 contract。Agent 再记录 DESIGN FROZEN 和批准依据。Freeze 固定的是已约定的 scope、invariants 和验收要求，不是把整份文件锁住。新发现、进度和证据仍然要接着写。批准 implementation，也不等于批准 merge。

已授权的验证，现有 subscription 能覆盖，就不用再问一次 provider、账户或费用。按量计费 API 和其他额外收费，需要适用的费用授权。已有时间限制、配额和明确约束仍然有效；agent 不能为了绕过限制就换账户或启用付费 fallback。任务如果另有真实 training 的授权要求，也仍然要遵守。

**进入下一步前，确认这件事**

让 agent 准备一份 kickoff，写清已批准的 design、填好的 contract、implementation base、下一步和停止条件。不要把还留着占位路径的模板直接粘过去，就当 execution 已经准备好了。

### 4. 为这个 PR 新开一个 implementation 会话。

在同一个目标项目里，真正新开一个会话，不是给 planning 会话改个名字。重新调用 skill，把带真实路径的 kickoff 发过去。新的 agent 不需要整段 planning 聊天记录，但需要已经保存的约定，以及能确认当前状态的源文件。

修改前，agent 必须先重读 skill entrypoint 和它的 execute 行，再读已批准的 design、填好的 contract，以及完整的 execution 和 test rules。它还要检查 branch、HEAD、已有改动、前置 PR 是否 merge，以及相关 job 是否仍在运行。无关改动要保留。如果你明确启用了某个 preset，它还要读对应说明，把当前 session 绑定到这个 PR。安装了 hook，不等于已经替它选好了当前 PR。

```text
Execute PR 01a. The approved DESIGN FROZEN document is
.structured-coding/plans/order-flag/pr-01a.md, and the filled contract is
.structured-coding/plans/order-flag/pr-01a-contract.md.
Use structured-coding: read its SKILL.md entrypoint, then the complete
resources its execute row lists, including Implementation Working Rules and
TEST / CI / GATE in full. Reconcile actual state, and begin.
Continue autonomously to READY FOR OPERATOR REVIEW under the contract.
Do not merge.
```

**进入下一步前，确认这件事**

确认 agent 找对了 PR，也读到了已批准的真实路径。Kickoff 让它在 contract 范围内执行，不会额外授予 host 权限，也不会替你启用 hook。

### 5. 让 agent 把约定范围内的实现循环做完。

Agent 做完一块有意义的修改，运行相关检查，review 逻辑和调用方，记录结果，然后 commit。Unit test 失败，或者在约定范围内发现漏掉的调用方，通常就是查原因、修复、继续。不该每个 commit 都来问你一次。发布 branch、创建 PR 和处理 CI 已经授权的话，它也应该接着做完。

需要你回来决定的是：解决办法要改变已冻结的要求、public interface、重要 scope，或者超出批准预算。Agent 应该带上证据和具体选择，而不是只说一句卡住了。你决定以后，它才能继续受影响的工作。

聊到需要 compact 时，做的还是同一个 PR。手动 compact 前，agent 更新 design 和 handoff。Compact 或 resume 后，它重新读完整规则，核对真实 Git 状态和 process 状态。原来有 test 在跑，就先检查它，别直接再开一份。Continuity 能帮助做机械检查、提供恢复指令，但不会替 agent 写出语义正确的 handoff。

**进入下一步前，确认这件事**

你问当前做到哪一步、有什么证据、接下来做什么，agent 应该能根据已保存的记录回答。收到 hook 提醒，不代表它已经 review，也不代表某个 test 已完成。

### 6. Review 做完的 PR，再决定是否 merge。

对于已授权的 PR 工作流，READY FOR OPERATOR REVIEW 表示约定的 implementation、validation 和逻辑 review 已完成，PR 已创建或更新，而且准确最终 HEAD 的必需 CI 已通过。HEAD 标识当前 commit。旧 commit 的 CI 绿了，不能证明后来又改的内容也通过了。

你把 diff 和原先答应交付的行为对起来，看偏离原因、test evidence 和剩余限制。有问题，就在原 implementation 会话里要求修复。Agent 要更新证据和最终 HEAD 的 CI，再交回来。你可以另找一个 reviewer agent，但工作流不要求必须为 review 多开一个会话。

满意后，明确授权 merge 这个 PR 和你 review 过的版本。没拿到这份授权，agent 就停在可 review 的状态。目前提供的 preset 没有 merge guard，所以这条边界仍靠指令，以及另外配置的 host 或 repo 保护来维持。Contract 如果只授权本地工作，就按本地终点交付，不能假装已经创建或验证了远程 PR。

**进入下一步前，确认这件事**

授权 merge 后，要确认远程确实完成了，并拿到 merge commit。发起了 merge 请求，不等于 merge 已经成功。

### 7. 更新计划，再开始下一个 PR。

确认 merge 后，让 agent 把当前 PR 标为已 merge，再更新所属 step，最后更新 overall plan。不光写完成了什么，还要写这次发现会怎样影响后面的工作。目前 hook 不会验证这些 merge 后的 planning 更新。

比如实现字母排序时，发现 resume 只保存一个文件名。后面的 PR 可能需要更明确地标识：同名文件重复出现时，下次到底从哪一次继续。把这个发现带进下一个 PR design，别沿着已经过时的假设接着做。

你可以回原 planning 会话，也可以新开一个，让它读更新后的文件。细化并批准下一个 PR，再开新的 implementation 会话。上一个 PR 的 agent 可以把记录补完、准备交接，但不能在旧 context 里悄悄开始实现下一个 PR。

**进入下一步前，确认这件事**

一个 PR 的收尾，不只是收到 merge 通知，还要把结果和对后续工作的影响写下来。接着重复这套流程时，你就比上次少了一些没弄清楚的问题。

## agent 到底维护哪些文档？

| 文档 | 作用 |
| --- | --- |
| Overall | 它记录整体目标、需求和主要步骤，让后面的 PR 知道在完成哪件事。 |
| Step | 它说明当前步骤要拆成哪些 PR、谁依赖谁，以及怎样观察到它们配合起来了。 |
| PR design | 它记录检查过的 code、commit plan，以及实施中持续更新的决定、进度和证据。 |
| Execution contract | 它记录当前 PR 允许改什么、允许执行哪些操作、预算和停止条件是什么。 |
| Handoff | 它记录当前 PR、branch 和 HEAD、仍在运行的 job、log 位置，以及恢复后具体接着做什么。 |

DESIGN FROZEN 固定的是你批准的要求，不是整份文档。Agent 仍要记录新发现和进度。Implementation、validation 和 review 分开记录，只有对应工作真的完成了才能打勾。如果一个 step 只有一个 PR，就展开原 step doc，不必复制成两份。

## compact 或 resume 时，怎么接着干？

Compact 是 host 为腾出 context 而压缩聊天历史的过程，不是新建一个 PR。新 PR 要开新的 implementation 会话；compact 或 resume 则继续原来的 PR。可选 continuity preset 检查已记录的机械状态是否仍然匹配、尝试保存 snapshot，并提供恢复指令。Design 和 handoff 写得是否准确、恢复时有没有真正核对清楚，仍由 agent 负责。

恢复时，重新读取当前 design、填好的 contract 和完整 execution rules；改 code 前核对 repo 和已有任务。snapshot 不能编决定或测试结果。自动 compact 不能因为 handoff 不完美就一直卡住。

<a id="hooks"></a>

## 可选 hook：安装、覆盖范围和限制

需要 compact 恢复可选 continuity，需要 commit/review 提醒可选 checkpoints，需要在 commit 后跑你声明的检查可选 standards，也可任意组合。默认都不开。Checkpoints 提供建议，不拦截 commit，也不证明已达到交付条件。Protocol 测试不能证明 native 事件送达或模型遵守了提示。agent 不会动态注册自己的 hook。

```sh
./structured-coding/scripts/install codex --project /path/to/project --hooks checkpoints --dry-run
./structured-coding/scripts/install codex --project /path/to/project --hooks checkpoints
./structured-coding/scripts/install codex --project /path/to/project --hooks continuity checkpoints
./structured-coding/scripts/install codex --project /path/to/project --check-hooks
./structured-coding/scripts/install codex --project /path/to/project --upgrade-registration
./structured-coding/scripts/install codex --project /path/to/project --remove-hooks checkpoints
./structured-coding/scripts/install codex --project /path/to/project --remove-hooks
```

这些命令和上面的普通安装一样，在包含 toolkit 的父目录里运行。/path/to/project 必须换成目标项目准确的 Git root；Claude Code 用户把 codex 换成 claude-code。先用 --dry-run 预览，再运行你选择的安装命令，不需要把每一行都执行一遍。Installation 会添加选中的 preset；只移除一个 preset，另一个仍可继续用。不带名称的 --remove-hooks 会移除本工具拥有的全部 hook。更改后重启 host，并在 /hooks 里检查注册与信任状态，installer 不会替你授予信任。注册的命令不含任何只属于你这台机器的路径，所以提交到共享设置文件里的注册对同事同样有效，而每个人仍然要在自己机器上审阅并信任这些 hook。旧版本装出来的注册保留其绝对路径，仍然可用；--upgrade-registration 会重写它，这会改变每一条命令，因此需要重新信任这些 hook。Agent 还要绑定当前 session，在 commit 前检查 staged diff，并明确准备 review handoff。提醒不能把没运行、没定论或仍在等待的检查变成通过。已有设置、skill 文件和 session 数据会保留。

| 功能 | 候选事件 | 应有的行为 | 已提供的支持 |
| --- | --- | --- | --- |
| H1 · 实现前 | PreToolUse | 核对已批准设计、contract、repo 状态和恢复情况。不匹配就拒绝依赖这些条件的修改；audit 和设计准备仍然允许。 | 尚未实现 |
| H2 · commit 前 | PreToolUse | 检查或提示 diff review、ledger、证据和偏离记录。agent 修好再试，不增加每个 commit 都找人批准的步骤。 | Checkpoints：明确的准备步骤，以及 Bash 直接 git commit 的提示；不强制执行 |
| H3 · merge 前 | PreToolUse + merge 路径覆盖 | 要求来自可信通道、对应具体 PR、目标 branch 和候选 HEAD 的明确授权，并核对完成条件及 CI/Gate 证据。覆盖 CLI、API、auto-merge 和直接操作目标 branch 的绕行路径。 | 尚未实现 |
| H4 · 手动 compact | PreCompact: manual | handoff 过时就先拦住，同步后再允许 compact。 | Continuity：只检查机械 checkpoint 是否仍然对应当前状态 |
| H5 · 自动 compact | PreCompact: auto | 允许 compact；必要时保存机械 snapshot。保存失败也要留警告，并要求恢复。 | Continuity：限时尝试 snapshot，不主动阻断自动 compact |
| H6 · compact/resume | SessionStart + 修改前检查 | 动态确认当前 PR，完整读取规则，核对 repo 和进程状态再继续，别重复启动任务。 | Continuity：提供 session 绑定的 PR 和完整读取指令；没有修改 guard，也不能证明恢复完成 |
| H7 · 准备交付 review | Stop / 完成事件 | 核对真实完成条件、最终 HEAD 的 CI、证据和 handoff。准备好 review 不等于允许 merge。 | Checkpoints：明确 intent 后最多一次 operator 提醒；不自动续跑，也不判定证据通过 |
| H7 · merge 后 | 已观察结果 / 核对远端状态 | 确认 merge，再要求并检查 PR → step → overall 更新。agent 写经验，下一个 PR 用新 session。 | 尚未实现 |

有事件名，不等于一定能拦截。adapter 得处理各 host 的 protocol、可信授权来源和 tool 覆盖缺口。merge 后的检查不能倒过来阻止 merge。实现 adapter 前，先看 contract 的验收场景。

比如：批准 PR 12 的 HEAD A，不等于允许 merge 后来的 HEAD B。实现中的 agent 也不能自己写个“已批准”，就把它当成人的授权。

<a id="further"></a>

## 想了解更多

| 资源 | 内容 |
| --- | --- |
| [工作流参考](references/agent-workflow.zh-CN.md) | Agent 在每个阶段做什么，以及它的权限到哪里为止。 |
| [PR 规范](prompts/pr-design-requirements.md) | 一份 PR 设计在被批准之前必须包含什么。 |
| [可选 hook](references/platforms.zh-CN.md) | 安装方式、覆盖范围，以及 hook 能强制什么、不能强制什么。 |
| [行为契约](references/hook-contract.md) | 完整的目标行为，包括尚未实现的部分。 |

<a id="standards"></a>

## 项目规范（可选）

项目可以把整个 codebase 层面成立的规矩写下来一次，不用每次 planning 都重说一遍。把模板复制到 .structured-coding/standards.md，编辑其中一个块就行。这个文件是可选的：没有它就用下面的 default，行为完全不变。只属于某一个 PR 的要求留在那个 PR 的对话里，这样这份文件才能一直是稳定的仓库资产。

| 你能设置什么 | 写在哪 | Default |
| --- | --- | --- |
| 由 LLM 判断的规范 | review.conventions | 空；你用普通句子写 |
| 这些规范什么时候被提出来 | review.trigger | PR 到达 review 就绪时 |
| 有明确通过/失败的命令 | checks.tools | ruff 和 pyright 只查改动文件；pytest 列出但关闭 |
| 这些检查什么时候适用 | checks.trigger | PR 到达 review 就绪时 |
| 你自己的补充 | .structured-coding/standards.local.md | 只能增加和收紧，不能放松团队的 |

scope 按工具分别设置，因为正确答案本来就因工具而异：只查改动文件适合 ruff 和 pyright，对 pytest 却是误导，因为覆盖这次改动的测试通常在改动没碰过的文件里。工具按「会不会执行你的代码」分组，所以 pytest 和 mypy 是认识的名字，但在真正运行它们之前仍需要一次明确批准。跑检查是一条显式命令：每一项返回 PASS、FAIL、INCONCLUSIVE 或 NOT RUN，而没安装的工具是 INCONCLUSIVE 而不是通过，因为它什么都没检查。目前还没有任何东西会自动运行；没有 hook，所以不会拦截任何 commit 或 merge。

英文是唯一正确源；中文是同步镜像，保留英文专业术语。给人看的解释沿用 DongbeiGPT。specification 和可复用 prompt 全部使用英文。
