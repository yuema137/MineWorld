# Codex 与 Claude Code

[English source](platforms.md) · 英文是唯一权威源，本页是中文镜像。

两份包使用同一份 `SKILL.md`、agent workflow、human guide 和完整 prompt。Codex 包另带 `agents/openai.yaml` 展示信息。Claude Code 包使用通用 skill 入口，不添加 fork、子 agent、模型或权限设置。两边默认都不注册 hook。installer 可以明确选装 [continuity](continuity.md)、[checkpoints](checkpoints.md)，或同时安装两者。已有 hook group 和其他项目设置会保留；全局设置、权限和信任配置不变。

## 放置与调用

把对应包中的整个 `structured-coding/` 文件夹复制到目标位置，保留内部相对路径。目标已存在时，先比较内容，避免覆盖已有定制。

| 平台 | 项目级目录 | 个人级目录 | 显式调用 |
| --- | --- | --- | --- |
| Codex | `.agents/skills/structured-coding/` | `~/.agents/skills/structured-coding/` | `$structured-coding` |
| Claude Code | `.claude/skills/structured-coding/` | `~/.claude/skills/structured-coding/` | `/structured-coding` |

Codex 的目录和调用方式依据 [OpenAI 官方 skills 文档](https://learn.chatgpt.com/docs/build-skills)；Claude Code 依据 [官方 skills 文档](https://code.claude.com/docs/en/skills)。核对日期：2026-09-05。已有定制安装目录的环境应遵循其实际发现规则，避免同时安装重复副本。

开始新 PR 时，先在 planning session 准备完整、已批准的 design 与填好的 contract，再在新的 implementation session 调用 skill 并指向这些文件。compact/resume 则读取当前 PR 的 handoff，继续同一个 PR。

## 选装 preset

在 clone 的仓库内运行，填写项目的准确 Git 根目录（含空格的路径要加引号）：

```sh
./scripts/install codex --project /path/to/project --hooks checkpoints --dry-run
./scripts/install codex --project /path/to/project --hooks checkpoints
./scripts/install codex --project /path/to/project --hooks continuity checkpoints
./scripts/install codex --project /path/to/project --check-hooks
./scripts/install codex --project /path/to/project --remove-hooks checkpoints --dry-run
./scripts/install codex --project /path/to/project --remove-hooks checkpoints
./scripts/install codex --project /path/to/project --remove-hooks
```

Claude Code 用户把 `codex` 换成 `claude-code`。需要 Python 3.9+、Git 和 macOS/Linux/WSL。目前保守的宿主版本下限是 Codex 0.153.4 和 Claude Code 2.1.261；实际事件送达还需要独立的 native 验证。修改后重启 host 并检查 `/hooks`；installer 不授予信任，也不会打开被禁用的 hook。安装后的副本在 `VERSION` 里带着自己的发布版本，`--check-hooks` 会报告它，所以你能知道某个项目装的是哪一版。

Codex 的信任分**两层**，两层都给到之前注册都是空转的。第一层是项目：在项目被信任之前，Codex 根本不加载任何 project-local hook，并会在 stderr 上说明，同时指出它要求写入的文件。那条记录必须写在 `CODEX_HOME` 下的 `config.toml` 里；命令行上的 `-c projects."<path>".trust_level="trusted"` 覆盖**不满足**这一条，而这种失败看起来像是 hook 被送达却被忽略了。第二层是每个 hook 定义按自身内容 hash 单独信任，所以改动命令会重新询问。两层都是 Codex 自己的机制，installer 都不会替你授予。Claude Code 没有对应的第二层。

安装是追加操作。单装 standards 会注册共享 SessionStart 和 PostToolUse，它是唯一会真正运行东西的 preset；它的 PostToolUse hook 预算是 60 秒而不是 12 秒，因为跑一次检查比发一条提示慢。单装 checkpoints 会注册共享 SessionStart、PreToolUse 和 Stop，不注册 compact handler。单装 continuity 会注册共享 SessionStart 和手动/自动 PreCompact。三个一起安装共六个 group，SessionStart 只出现一次。移除一个后另一个仍可用。裸 `--remove-hooks` 移除全部自有 preset，保留 skill 和 session 数据。重复安装同一选择不会改变文件字节；重复或未知名称会被拒绝。

一份 aggregate receipt 保存原始设置备份。旧 continuity receipt 仍可读取，仅在明确改变安装状态时迁移；检查、预览和未改变选择的重装都不改写它。installer 不覆盖定制过的 runtime 依赖：先比较、备份已有 skill，再明确升级。自有 group 缺失、被修改或重复，receipt schema 未知，以及路径被重定向或无法核验，都会导致拒绝。新安装注册的命令不含任何机器相关路径，因此提交到共享设置文件里的注册在同事的机器上同样有效；每台机器仍需自己审阅并信任这些 hook。旧版本装出来的注册保留其绝对路径命令，仍可检查、仍可移除，只有 `--upgrade-registration` 会重写它；重写会改变每一条命令，因此需要重新信任这些 hook。`--check-hooks` 会说明注册是否可移植，并且只对绝对路径的注册报告记录的 interpreter。

配置和 receipt 在共享锁下分别原子替换，使用最多 8 MiB 的私有 transaction journal（配置和 receipt 各最多 1 MiB）。重试只处理与已知前后状态完全一致的组合；不认识的用户修改会保留并报告冲突。`--check-hooks` 和 dry-run 只报告待恢复状态，不写入；明确执行安装或移除重试时才能恢复。应避免同时编辑配置：这不能与外部编辑器或 host 构成统一的原子事务。只有最后一个 preset 被移除，且剩余设置与原始解析结果一致时，才恢复原始字节。不会用旧整份配置覆盖仍在使用的 preset。

## Hook 的平台映射

下表对应完整目标 contract，不是配置文件。Continuity 提供 compact 同步检查、snapshot 尝试和恢复指令。Checkpoints 增加直接 commit 的提示，以及明确准备 review 后的一次提醒，不自动续跑，也不强制核验 H2/H7 证据。freeze/merge guard 和强制恢复仍是后续工作。[Continuity interface](continuity.md) 和 [checkpoints interface](checkpoints.md) 说明了设置、测试、输出对象与时机，以及覆盖限制。Host protocol 测试不能证明实际生命周期事件已经送达；安装后还要在 `/hooks` 中核对。

| 工作流行为 | Codex 候选事件 | Claude Code 候选事件 |
| --- | --- | --- |
| 实现前的 freeze 检查、merge 前的授权检查 | `PreToolUse` | `PreToolUse` |
| 手动/自动 compact 前检查 | `PreCompact`，区分 `manual` / `auto` | `PreCompact`，区分 `manual` / `auto` |
| compact/resume 恢复当前 PR | `SessionStart`，匹配 `compact` / `resume` | `SessionStart`，匹配 `compact` / `resume` |
| 结束时核对 handoff 与状态 | `Stop` 等结束事件，需适配循环语义 | `Stop` 等结束事件，需适配循环语义 |

事件映射来自 [OpenAI Hooks 文档](https://learn.chatgpt.com/docs/hooks)和 [Claude Code Hooks reference](https://code.claude.com/docs/en/hooks)。两边事件同名不意味着返回协议完全相同；尤其不能直接复制一边的阻断 JSON 到另一边。

Codex 官方文档说明部分工具路径可能不经过 hook，持续 exec 会话的后续输入也不会重新触发同一次命令的前置检查。因此后续实现必须核查真正可用的工具路径与绕行方式，不能只凭存在 `PreToolUse` 就宣称所有写入和 merge 已被拦截。[OpenAI Hooks：工具覆盖](https://learn.chatgpt.com/docs/hooks#tool-coverage)

两个平台都需要分别实现手动 compact 的阻断和自动 compact 的放行恢复。恢复时优先注入当前 PR 身份、文档路径和恢复要求；长模板若无法完整注入，要求 agent 从文件完整读取。不要把 compact 后平台自动保留的摘要当成已加载完整 execution 模板的保证。

本包没有设置全局自动批准、绕过权限或持续执行开关，也不依赖原文提到的某个 `/goal` 命令。持续执行与 hook enforcement 是宿主集成问题；本次交付的任务语义与停止条件已在 skill 和行为约定中完整定义。
