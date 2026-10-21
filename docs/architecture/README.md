# 架构与决策

当前只有无网络CLI骨架。下述边界是长期设计约束，不代表相关模块已实现。

协议解析不做路由业务；Transaction、Dialog、Call与MediaSession各有职责；状态机产生动作，运行时执行I/O；时间可控；最终媒体输出有明确所有者。先一个 Cargo 包与一个服务端进程，按真实需求逐步分模块；package 与 crate 的区别及后续拆分条件见 ADR-0001。

ADR：

- [0001：单 Cargo 包起步](adr/0001-small-core.md)（Accepted：2026-10-21 用户确认设计）
- [0002：状态机、I/O与时钟](adr/0002-protocol-runtime-time.md)（Accepted：2026-10-20 用户确认设计）
- [0003：需求、状态与证据的唯一来源](adr/0003-documentation-evidence.md)（Accepted：2026-10-21 用户确认制度）

初始化允许Codex写草案；用户未审阅不得将状态改为Accepted。ADR不是把所有未来接口提前锁死，应说明当前决定、原因、替代方案和何时重审。
