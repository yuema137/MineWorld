<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/images/readme/hero-zh-dark.svg">
    <img src="docs/images/readme/hero-zh-light.svg" alt="MineWorld：搭一个活的世界，再走进去。用模块把人、地方和规则拼起来。一台服务器让世界一直运转，2D 和 3D 是看它的两种方式。" width="100%">
  </picture>
</p>

<p align="center">
  <img src="docs/images/readme/3d-cafe-front.jpg" alt="主街上的咖啡馆，3D 视图" width="100%">
</p>

<p align="center">
  <a href="LICENSE"><img alt="许可证：MIT" src="https://img.shields.io/badge/license-MIT-b8792b?style=flat-square"></a>
  <a href="https://github.com/yuema137/MineWorld/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/yuema137/MineWorld/actions/workflows/ci.yml/badge.svg?branch=main"></a>
  <img alt="支持 macOS、Linux、Windows" src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-5b6470?style=flat-square">
  <img alt="Rust" src="https://img.shields.io/badge/server-Rust-5b6470?style=flat-square&logo=rust">
  <img alt="Godot 4.7" src="https://img.shields.io/badge/views-Godot%204.7-5b6470?style=flat-square&logo=godotengine&logoColor=white">
  <img alt="状态：早期预览版" src="https://img.shields.io/badge/status-pre--alpha-a46a00?style=flat-square">
</p>

<p align="center">
  <a href="README.md">English</a> · <b>简体中文</b>
</p>

<p align="center">
  <a href="#有什么不一样">为什么选 MineWorld</a> ·
  <a href="#一个世界是怎么搭起来的">怎么搭</a> ·
  <a href="#一个世界两种看法">两种视图</a> ·
  <a href="#真人和-ai-智能体玩的是同一个游戏">智能体</a> ·
  <a href="#有深度的人物可插拔的人物生成现已支持-matraix">人物</a> ·
  <a href="#看看实际画面">实际画面</a> ·
  <a href="#现在能用什么">现状</a> ·
  <a href="#快速上手">快速上手</a> ·
  <a href="#接下来往哪走">路线图</a>
</p>

> 本文是英文 [`README.md`](README.md) 的中文翻译。两者如有出入，以英文版为准。

**搭一个活的世界，再走进去。** MineWorld 是一个开源框架，用模块拼出可以长久存在的游戏世界：小镇里的人上班、买东西、聊八卦、记事情，没人登录也照样过日子。你可以用 2D 或 3D 去玩，和朋友一起，也可以和 AI 智能体一起，它们守的规则和你完全一样。

图里的咖啡馆小镇是 MineWorld 自带的示例世界，下载就能玩。它只是能搭出来的世界之一，并不是产品本身。

> MineWorld 目前是早期预览版。下面每一项都标了 ✅（已在 `main` 上可用，有测试）或 🚧（已设计，正在做）。标 🚧 的东西现在还用不了。

## 有什么不一样

| | |
| --- | --- |
| **是框架，不是一款游戏** | 你是在拼世界：装模块、写几个文本文件、配置、运行。不需要 fork 一款游戏再去改它的代码。✅ |
| **规则就是模块** | 钱、工作、交情、身体、天气，各自是一个模块，只管世界里自己那一摊。关掉一个，它的动作就没了，小镇其余部分照常运转。小小的核心永远不知道什么是工作、什么是钱。✅ |
| **真人和 AI 智能体都是玩家** | 智能体和真人从同一扇门入座，只看得到那个人看得到的东西，被拒绝时理由也一样。服务器分不出谁是谁。✅ Python 库 · 🚧 给 Claude Code 等智能体用的 MCP 适配器 |
| **会记事的人** | 每个人都有自己的记忆，记着自己看到、听到的事，会像人一样慢慢遗忘，记忆占用有上限。驱动他们的可以是语言模型、简单脚本，或者完全不用 AI 的决策模型。✅ · 🚧 共享记忆 |
| **有深度的人物** | 每个人可以带一组丰富的属性（作息、饮食、性格、技能……），按真实的人口分布抽取。可以用 MineWorld 自己的属性表，也可以换成 MatrAIx 的 1,290 维人物设定。🚧 |
| **真正的经济** | 钱、商店、工资和库存现在就有。✅ 接下来：货币本身是数据，每个卖家自己定价，可以讨价还价，还有一位市场播报员，大家可以订阅他发布的价格公告。🚧 |
| **持久的世界** | 每一次变化连同起因都会记下来。世界扛得住程序崩溃，从同一个种子重放结果分毫不差，在 macOS、Linux 和 Windows 上跑出的结果也完全一致。✅ |
| **一个世界，两种视图** | 2D 和 3D 是看同一台服务器的两扇窗，两边都不会自己另定规则。✅ |

## 一个世界是怎么搭起来的

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/readme/how-it-works-zh-dark.svg">
  <img src="docs/images/readme/how-it-works-zh-light.svg" alt="一个世界是怎么搭起来的。01 世界：人、地方和物品。02 规则：聊天、买东西、上班、吃饭。03 物品：多个世界可以共用的物品种类。它们装进同一台服务器，服务器里的核心只知道有哪些东西、在哪、几点了、发生过什么。04 大脑（真人玩家、简单脚本、AI 模型）只发请求。05 外观：一套 2D 画风和一套 3D 画风。服务器告诉 2D 和 3D 视图你能看到什么、能做什么，并保存每一次变化。" width="100%">
</picture>

一个世界由五种"包"拼成。包就是你往项目里加的一个文件夹（[`docs/MODULE_SPEC.md`](docs/MODULE_SPEC.md)）：

1. **世界**：里面有谁。人、地方和物品，用简短的文本文件写出来。
2. **规则模块**：能发生什么。每个模块只管自己那一摊。工作模块要给 Alice 发工资时，只会说一句"Alice 的工资该发了"，真正动钱的只有钱的模块。
3. **物品种类**：咖啡、雨伞、书。多个世界可以共用。
4. **大脑**：角色由谁来做决定。真人玩家、脚本、AI 模型，或者外部的智能体。
5. **外观**：世界画成什么样。一套 2D 画风，一套 3D 画风。

<details>
<summary><b>现有的规则模块</b></summary>

```text
presence        人在哪里，每个人能看到什么
movement        走路，以及哪些地方是连通的
conversation    聊天，并记住谁说过什么
group-activity  邀请别人、一起做事
relationships   谁认识谁，关系有多近
naming          每个人叫什么
schedule        每个人一天的安排
item            有哪些种类的东西
inventory       谁身上带着什么
item-transfer   互相送东西
employment      工作和班次
economy         钱、商店和工资
consumption     吃和喝
bodies          身体不会重叠；推、踢、扔、撞开；绕开家具走路
calendar        日期和太阳
weather         逐小时的天气，或者重放真实气象站的记录
fishing         第一个由本项目以外的人写的模块，放在它自己的代码仓库里
```

示例世界：[`worlds/social-cafe`](worlds/social-cafe)（咖啡馆小镇）、[`worlds/market-town`](worlds/market-town)（同一个小镇，加上个人物品、工作、钱和食物）、[`worlds/bodies-yard`](worlds/bodies-yard)（两个房间，人和箱子互相挡路）。

一个世界在 `world.yaml` 里列出要用的模块。集市小镇就是咖啡馆小镇多加了六行：

```yaml
systems:
  - presence
  - movement
  - conversation
  # … 咖啡馆小镇的其余模块
  - item
  - inventory
  - item-transfer
  - economy
  - employment
  - consumption
```

要写一个新的规则模块，就在 `systems/` 下加一个文件夹，在 `systems/installed/` 里加两行，然后重新编译（[`systems/README.md`](systems/README.md) 的 "Adding a pack" 一节）。

</details>

## 一个世界两种看法

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/readme/two-views-zh-dark.svg">
  <img src="docs/images/readme/two-views-zh-light.svg" alt="一个世界，两种看法。在 2D 里你点 Alice 再选“说话”；在 3D 里你走到 Alice 跟前按 E。两边发给服务器的是同一个请求。服务器检查你是不是在 3 米以内，把这次说话记下来，再把同样的回答发回两个视图。" width="100%">
</picture>

服务器对每个请求，要么告诉你发生了什么，要么告诉你为什么不行（"离得太远"、"这里不允许"）。**集市小镇的一个早晨：**

1. 早上 5:30，Alice 开始上班，走到咖啡馆柜台后面，因为她的每日安排里是这么写的。
2. 你走进店里，打开自己的菜单，上面有"买咖啡"。这是钱的模块给每个在店里的人提供的选项。
3. 你选了它。服务器查一下你的钱包，扣掉价钱，给你一杯咖啡，并把发生了什么、为什么发生记下来。
4. 咖啡馆的咖啡少了一杯。其他玩家都看得到，服务器重启以后也还是这样。
5. 下午 2:00，工作模块报告 Alice 的工资该发了，钱的模块付钱给她。然后她去公园散步。

目前买东西要在 2D 视图里做，3D 里的购物还在开发中。把钱的模块关掉，第 2 步就不会出现，小镇其余部分照常运转。

## 真人和 AI 智能体玩的是同一个游戏

外部的智能体（一个智能体框架、一段脚本，很快还有 Claude Code 或 Codex）是**玩家**，不是拥有管理员权限的提线木偶。它凭邀请码入座，看到的是那个人自己的视角，从世界给出的选项里挑，服务器对它的每个请求的检查和对真人一模一样。真人可以接管智能体的座位，反过来也行。世界不会停下来等谁思考。

| | 状态 |
| --- | --- |
| Python `Player` 库（[`clients/agent`](clients/agent/README.md)）：入座、观察、行动、断线后接着玩，视图信息丰富（钱、物品、工作、日程、熟人、邀请、天气、价格） | ✅ [#204](https://github.com/yuema137/MineWorld/pull/204) |
| 座位授权（房主只把指定的座位交给智能体）和每个座位的请求额度，真人和智能体一视同仁 | ✅ [#205](https://github.com/yuema137/MineWorld/pull/205) |
| MCP 适配器：Claude Code 等 MCP 智能体通过 stdio 来玩；在三个系统上跑的脚本化 MCP 试玩测试 | 🚧 已设计并冻结（[step 27](.structured-coding/plans/mvp0/step-27-agent-players.md)） |
| 通过 Streamable HTTP 接入的远程智能体；教程章节"以智能体身份来玩" | 🚧 已设计 |

## 有深度的人物：可插拔的人物生成（现已支持 MatrAIx）

一个小镇之所以显得活，是因为里面的人有看得出来的差别：谁天一亮就起床，谁点燕麦奶，谁一开口就停不下来，谁会烤面包。MineWorld 给每个人一组**属性**，属性就是普通的数据，按真实的人口分布抽取；每个世界还可以自己选择**这些人从哪里来**。

> 🚧 **状态：方案已由项目负责人于 2026-10-15 批准，正在实现。** 下面每一部分，包括 MatrAIx 后端和它的演示世界，都是 MVP-0 发布前必须完成的。设计文档：[step 30](.structured-coding/plans/mvp0/step-30-person-attributes.md)。

- **自带一份精选属性表。** 最多 100 个属性，分六组（外貌、背景、心理、能力、行为、生活方式）。每个属性都要有游戏里的某样东西真正用到它才保留：一个决定、一段对话、一个价格、一个人的长相。
- **自定义属性不用写代码。** 往世界的 `attributes/` 文件夹里放一个 YAML 文件，`freckles`（雀斑）就有了，会被抽取，也会出现在别人看到的信息里。
- **真实的人口分布。** 世界指定一份注明出处的人口分布。同一个确定性的抽样器负责填充所有角色，并且绝不改写一个人自己文件里写明的内容。
- **可插拔的后端和掩码。** 世界选一个人物生成后端，再选一个掩码，决定这个世界里有哪些属性。掩码没选上的属性在这个世界里根本不存在：任何提示词、视图或面板都带不出它。
- **大家看到的是同一份。** 一个人身上能被注意到的东西，会出现在 2D 和 3D 的查看面板里、AI 角色的"我看到的人"里，以及智能体看到的那个人的信息里。

### MatrAIx：可选的人物生成后端

[MatrAIx](https://github.com/MatrAIx-ai/MatrAIx-Persona-8B) 是一个大规模人物模型：它的设定用**五组共 1,290 个维度**来描述一个人（背景、心理、能力、行为与互动、生活方式），每个取值都配有一句自然语言描述，标签有六种语言，包括简体中文。MineWorld 将允许任何世界打开它：

```yaml
# worlds/<your-world>/configure/attributes.yaml
backend: matraix          # 默认：mineworld
mask: matraix-core-100    # 或者：all
```

- **原样导入。** 他们以 MIT 许可证发布的设定文件锁定到上游的一个确切提交，并逐字节校验；维度名、取值、描述和多语言标签都保持原样。
- **掩码。** `matraix-core-100`（默认）挑出大约 100 个会影响游戏的维度，每个都写明为什么选它。`all` 打开所有导入的维度。其中有 50 个维度自带比 MIT 更窄的许可声明，在确认条款之前暂不导入，所以目前 `all` 是 1,240 个。
- **一个演示世界** `worlds/matraix-demo`，展示 MatrAIx 生成的人物出现在选角、AI 角色的"我是谁"和"我看到的人"、智能体的视图，以及两种视图的查看面板里，也包括中文界面。还有一章教程，带你把这个后端打开，再关回去。
- **绝不是硬依赖。** 它是一个单独的包，默认不安装。没有它，MineWorld 照样能编译，完整测试照样全部通过，这个包以外没有任何东西依赖它。

*可选的人物生成后端，基于 MatrAIx 人物设定（Li, Hao et al., 2026，[arXiv 2608.04205](https://arxiv.org/abs/2608.04205)；MIT），原样导入。* 感谢 MatrAIx 的作者公开发布他们的设定：请看他们的论文 *MatrAIx: Simulating the World with 8.3 Billion Persona Agents* 和他们的[代码仓库](https://github.com/MatrAIx-ai/MatrAIx-Persona-8B)。

## 活生生的人

| | 状态 |
| --- | --- |
| 每个人自己的记忆：记住自己感知到的事，随时间压缩，按相关程度回想 | ✅ [#143](https://github.com/yuema137/MineWorld/pull/143) |
| 像人一样的遗忘：熟人慢慢淡忘、可替换的遗忘策略、可选的模型摘要；默认每人最多 20 MiB | ✅ [#186](https://github.com/yuema137/MineWorld/pull/186), [#188](https://github.com/yuema137/MineWorld/pull/188), [#201](https://github.com/yuema137/MineWorld/pull/201), [#213](https://github.com/yuema137/MineWorld/pull/213) |
| 每个人可以换用不同的决策模型，自带一个完全不用 AI 的 | ✅ [#191](https://github.com/yuema137/MineWorld/pull/191), [#210](https://github.com/yuema137/MineWorld/pull/210) |
| 由语言模型驱动的 AI 角色：默认用你自己电脑上的模型，也可以用你自己的密钥接 11 家在线服务之一；测试只重放录好的回答，从不调用真实模型 | ✅ [#120](https://github.com/yuema137/MineWorld/pull/120), [#126](https://github.com/yuema137/MineWorld/pull/126), [#207](https://github.com/yuema137/MineWorld/pull/207) |
| 全镇共享、只存一份的记忆，唯一的访问权限来自亲身感知 | 🚧 已写好规格（[step 28](.structured-coding/plans/mvp0/step-28-shared-memory.md)） |
| 每个人都按主角的标准来做，人物来源可插拔 | 🚧 评审中（[step 26](.structured-coding/plans/mvp0/step-26-cast.md)） |
| 换衣服、换发型，初始服装，买衣服 | 🚧 已设计（[step 32](.structured-coding/plans/mvp0/step-32-wardrobe.md)） |

## 看看实际画面

所有截图都是在本项目自己的 2D 和 3D 视图里截的。

| | |
| --- | --- |
| ![3D 的主街](docs/images/readme/3d-street.jpg) | ![在 3D 里和 Alice 聊天](docs/images/readme/3d-talk-to-alice.jpg) |
| **主街，3D。** 用第一人称或第三人称走。 | **和 Alice 聊天，3D。** 连着一台正在运行的服务器，她记得你说过的话。 |
| ![3D 的咖啡馆里面](docs/images/readme/3d-cafe-interior.jpg) | ![3D 里的角色特写](docs/images/readme/3d-character.jpg) |
| **咖啡馆里面，3D。** 从正门走进去，四处看看。 | **你的角色，3D。** 默认的玩家角色站在咖啡馆里。 |
| ![2D 的主街](docs/images/readme/2d-street.jpg) | ![2D 里 Alice 的菜单](docs/images/readme/2d-menu.jpg) |
| **同一个小镇，2D。** 从上往下看，还有住在这里的人。 | **服务器给的菜单，2D。** 此刻你能和 Alice 做什么。 |
| ![2D 里在咖啡馆买东西](docs/images/readme/2d-shop.jpg) | ![同一家咖啡馆的 2D 和 3D](docs/images/readme/one-world-two-views.jpg) |
| **买东西，2D。** 你的钱、身上带的东西，以及这里在卖什么。 | **一家咖啡馆，两种视图。** 同一个世界，画了两遍。 |

## 现在能用什么

✅ 已在 `main` 上可用，有测试 · 🚧 已设计，正在做 · 🗺 下一阶段的计划

| 方面 | 状态 |
| --- | --- |
| **核心** | ✅ 世界时钟、需要持续一段时间的活动，以及每一次变化和它的起因的完整记录，压缩存储。程序崩溃后世界不会丢，从同一个种子重放结果分毫不差 |
| **用积木搭世界** | ✅ 有测试证明：集市小镇 = 咖啡馆小镇 + 六个规则模块 + 一些设置，核心一行没改。去掉其中任何一个，小镇照样能运转 |
| **日常生活** | ✅ 人们走路、聊天、认识、交朋友、一起做事、拥有和赠送东西、上班、领工资、买东西、吃喝，能连续过 300 个游戏日 |
| **身体** | ✅ 人和物体不会互相穿过去；可以推、踢、扔、撞开；规划绕开墙和家具的路线 · 🚧 给小镇加上墙 |
| **时间和天气** | ✅ 日期、星期几、日出日落；逐小时的天气，集市小镇重放的是圣地亚哥一个真实气象站的记录 · 🚧 在 2D 和 3D 视图里把天气画出来 |
| **活生生的人** | ✅ 记忆、遗忘、每人 20 MiB 的上限、可换的决策模型、用本地或在线模型的 AI 角色 · 🚧 共享记忆，以及你在 2D 里告诉 Alice 的话，她在 3D 里还记得 |
| **智能体当玩家** | ✅ Python `Player` 库、座位授权、每个座位的请求额度 · 🚧 MCP 适配器 |
| **人物属性** | 🚧 精选属性表、自定义属性、人口抽样、可选的 MatrAIx 后端及其演示世界 |
| **经济** | ✅ 钱、商店、库存和工资 · 🚧 货币即数据、卖家自定价格、讨价还价、价格播报员和公共公告（[step 31](.structured-coding/plans/mvp0/step-31-wealth-prices.md)） |
| **你自己的规则** | ✅ 第一批可调的规则（比如一个人隔多久才能再开口）· 🚧 其他模块的规则、世界事件，以及两个示例世界：一座讲规矩的庄园，一座溜冰场 |
| **扩展包** | ✅ 每个模块都有名字、版本号和许可证；世界会写明需要哪些版本；不用重新编译就能添加新种类的物品；可以用来自其他代码仓库的模块，并锁定到确切的版本 |
| **多人一起玩** | ✅ 邀请码、昵称、断线 30 秒内可以回来、真人接管 AI 角色、房主可以暂停或请人离开；没人在线时，托管的小镇照样过日子；每个人只会被告知自己看得到、听得到的事 · 🚧 四个人同时在线的测试 |
| **2D 视图** | ✅（预览版）走路、进出门、聊天、邀请、购买、赠送、吃喝，全部通过服务器给出的菜单 |
| **3D 视图** | ✅ 走、跑、跳，走进咖啡馆，在运行中的服务器上和 Alice 聊天 · 🚧 整条街都按服务器的地图来搭、购物、在普通电脑上也流畅、每个世界都有远景 |
| **小镇** | 🚧 每个小镇都有一座火车站，同时也是它的标志性建筑；集市小镇的车站以圣地亚哥的 Santa Fe Depot 为原型（[step 29](.structured-coding/plans/mvp0/step-29-station-landmark.md)） |
| **设置** | ✅ 在任一视图里按 Esc：英文或简体中文、12 或 24 小时制、窗口和显示选项、鼠标灵敏度和反转 Y 轴。两种视图共用设置 |
| **创建世界** | ✅ 自动生成的 AI 角色配置参考；在 CI 里逐步执行每一步教程的运行器 · 🚧 教程章节本身 |
| **电脑系统** | ✅ 同一个世界在 macOS、Linux（Intel 和 ARM）和 Windows 上跑出的结果完全一致，每次改动都会检查 · 🚧 完整测试在 Windows 上运行 |
| **很多小镇** | 🗺 下一阶段：一个世界里有好几个小镇，火车、公交、出租车，在车上走动，小镇地图和世界地图 |
| **看起来像真的** | 🗺 下一阶段：用真实高度数据做的山，能蹚的水，随服务器的风摆动的树 |

更多细节：[`docs/MVP.md`](docs/MVP.md)（§9，验收标准）和 [`docs/MVP_STATUS.md`](docs/MVP_STATUS.md)。

## 快速上手

你需要装 [Rust](https://rustup.rs)（会自动装上合适的版本）和 [Godot 4.7](https://godotengine.org)，并且能在命令行里直接运行它们。

```sh
# 1. 拿到代码
git clone https://github.com/yuema137/MineWorld && cd MineWorld

# 2. 用 2D 玩集市小镇（点地面走路，点人弹出菜单，按 Q 打开自己的菜单，
#    按 Esc 打开设置，可以切换到简体中文）
./mineworld-2d

# 3. 用 3D 走进咖啡馆，在柜台前按 E 和 Alice 说话
./mineworld-slice --world
```

完全不要画面，让集市小镇跑 30 个游戏日，看看发生了什么：

```sh
cargo run -p mineworld-cli -- run worlds/market-town --headless --seed 7 --days 30
```

搭一个你自己的世界，然后用文本文件添加人、地方和物品，在它的 `world.yaml` 里打开需要的模块：

```sh
cargo run -p mineworld-cli -- create my-world
cargo run -p mineworld-cli -- validate my-world
cargo run -p mineworld-cli -- run my-world --headless --seed 1 --days 7
```

想用程序来玩，看 [`clients/agent`](clients/agent/README.md)。想让角色接上语言模型，看 [`cognition/lm-controller`](cognition/lm-controller/README.md) 和自动生成的[配置参考](docs/reference/cognition-toml.md)。

**支持哪些电脑。** MineWorld 面向 macOS、Linux 和 Windows；目前在 macOS 上开发和游玩，三个系统上结果一致这一点有自动检查。想用 Docker 开一个咖啡馆小镇：先运行 `docker build --target runtime -t mineworld .`，再运行 `docker run -p 7878:7878 -v mineworld:/var/lib/mineworld mineworld`。

## 接下来往哪走

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/readme/roadmap-zh-dark.svg">
  <img src="docs/images/readme/roadmap-zh-light.svg" alt="路线图。01 现在（MVP-0，正在做）：一个会过日子的小镇，用 2D 或 3D 玩，一个人或和朋友一起。02 下一步（MVP-1，已设计）：很多小镇用火车、公交和出租车连起来，有地图，每个小镇有自己的天气。03 再往后（计划中）：真实的圣地亚哥山地和海岸、能蹚的水、自驾旅行和可视化的世界编辑器。" width="100%">
</picture>

**第一次公开发布（MVP-0）之前**，要完成上面所有标 🚧 的内容：通过 MCP 接入的智能体、共享记忆、带 MatrAIx 后端的人物属性、卖家自定价格和讨价还价、衣柜、每个小镇的车站和地标、每条地平线上的远景，以及每个人都按同一个标准来做。

**再往后：**

- **很多小镇连成一个大世界。** 路上要花世界里的时间；在火车上你可以走动，跟别的乘客聊天。
- **贴近真实世界。** 物理数值都取自真实测量并写明出处（一个人肩宽大约 50 厘米）。太阳按小镇的真实位置升起落下。
- **规则、画风、角色都由你定。** 世界的作者选画风、选模块，也定下谁能对谁做什么。就算完全不用 AI，世界照样能运转。

完整的设想见 [`docs/VISION.md`](docs/VISION.md)。

## 延伸阅读

以下文档均为英文：

- [`docs/VISION.md`](docs/VISION.md)：为什么做这个项目，要往哪里走
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)：各部分怎么配合
- [`docs/CORE_CONCEPTS.md`](docs/CORE_CONCEPTS.md)：本项目用到的词语及其定义
- [`docs/MODULE_SPEC.md`](docs/MODULE_SPEC.md) 和 [`docs/PACKAGE_FORMAT.md`](docs/PACKAGE_FORMAT.md)：怎么写包
- [`CLAUDE.md`](CLAUDE.md) 和 [`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md)：每一份贡献都要遵守的规则；在自己电脑上运行 `python3 scripts/ci_layer.py fast` 可以跑较快的那部分检查

## 许可证

MIT。例外是 `clients/3d-spike/tools/blender/` 里的 Blender 脚本：它们用到了 Blender 的 GPL 接口，所以采用 GPL-2.0-or-later。来自他人的美术素材是 CC0。每一份美术素材的来源，包括用 AI 工具做的部分，都列在 [`NOTICE`](NOTICE) 里。`docs/images/readme/` 里的截图和示意图都是为本项目制作的。
