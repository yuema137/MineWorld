# 从零搭一个自己的世界

这份教程带你从一个空文件夹开始，做出一个自己的 MineWorld 世界。
你会一步一步搭出一个小世界，叫 **Corner Shop**（街角小店）：两个房间，三个人，一家店。

每一步分两段：先是**照着做**，再是**你应该看到**。教程里的每条命令、每个文件，
项目的测试都会在 Linux、macOS 和 Windows 上真的跑一遍。所以你看到的结果应该和教程里写的一样。

这一页翻译自英文原文 [README.md](README.md)。两边不一致时，以英文为准。

| 章 | 你要做什么 | 大约 |
| --- | --- | --- |
| [01 安装 MineWorld](01-get-mineworld.zh-CN.md) | 拿到 `mineworld` 命令，跑起第一个世界 | 10 分钟 |
| [02 第一个世界](02-your-first-world.zh-CN.md) | 写一个世界：一个房间，两个人 | 15 分钟 |
| [03 两个房间](03-two-rooms.zh-CN.md) | 加上小店，在两个房间之间开一扇门 | 10 分钟 |
| [04 三个人](04-three-people.zh-CN.md) | 给每个人起名字、排一天的安排；看一个人这一天发生了什么 | 15 分钟 |
| [05 一家店](05-one-shop.zh-CN.md) | 商品、公司、小店、钱和工作 | 20 分钟 |
| [06 进去玩](06-play-it.zh-CN.md) | 把世界开起来，走进去逛 | 10 分钟 |
| [07 世界的规则](07-rules-of-the-world.zh-CN.md) | 改一改谁能做什么、在哪里能做 | 15 分钟 |
| [08 时间和天气](08-time-and-weather.zh-CN.md) | 给世界一个日期、一个地点和天气 | 10 分钟 |
| 09 谁来做决定 | 每个人怎么做选择 | 以后的教程更新会补上 |
| 10 遗忘 | 不再见面的人，会慢慢忘掉对方 | 以后的教程更新会补上 |
| 11 活动 | 每个周末办一次集市 | 等 happenings 功能上线后补上 |
| [12 Pack](12-packs.zh-CN.md) | 看看这个版本带了哪些 pack；自己做一个 Entity Pack 并用上 | 10 分钟 |
| [13 被拒绝的时候](13-when-something-is-refused.zh-CN.md) | 教程里会碰到的每条拒绝信息，以及怎么改 | 随时查 |

第 02 到 05 章做完的世界，放在 [`examples/corner-shop/`](examples/corner-shop/world.yaml)。

## 教程里的词

MineWorld 自己的概念，教程里保留英文原名，因为命令的输出和文档里用的也是这些名字。
每一章第一次用到时，会先用中文说清它管什么。

| 英文原名 | 它是什么 |
| --- | --- |
| World Pack | 一个文件夹，写着一个世界里有哪些地方、哪些人、用哪些模块 |
| System Pack | 一个模块，负责让一类事情发生，比如走路、说话、买东西 |
| Entity Pack | 一组现成的东西，比如各种商品，哪个世界都能拿来用 |
| pack | 能安装、能分享的一份东西；System Pack 和 Entity Pack 都是 pack |
| Place | 人能待的地方，比如家、小店 |
| Person | 住在世界里的人 |
| key | 一个小写的短名字，比如 `ana`；它同时也是文件名 |
| seat | 玩家或程序能操控的人 |
| fact | 世界里发生过的一件事；世界会把每件事都记下来 |
| genesis fact | 世界刚开始时就成立的事 |
| passage | 两个地方之间的一扇门 |
| routine | 一个人每天打算几点待在哪里 |
| save | 一个文件夹，存着世界和它发生过的所有事 |
| biography | 一个人身上发生过的、对他这一生有意义的事，按时间排好 |
| class | 带同一个 tag 的一组东西，规则用它来指一群人 |
| Interaction List | 这个世界里谁能对谁做什么、在哪里能做 |
| invite | 玩家加入 server 时要输入的一串码 |

`server`、`client`、`YAML`、`build` 这类通用的技术词，教程直接用，不另外翻译。

每个文件能写哪些字段，都列在自动生成的[参考手册](../reference/README.md)里。
这些概念的准确定义在 [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md)。参考手册和这些规格文档只有英文版。

给 agent 看的：每一页的格式，以及测试对每一页的要求，写在 [`GUIDE_FORMAT.md`](../GUIDE_FORMAT.md)。
