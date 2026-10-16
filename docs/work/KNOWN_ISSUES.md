# 已知问题与未满足条件

状态依据：[INIT-001 实际报告](../handoff/BOOTSTRAP-REPORT.md)。

| ID | 状态 | 问题 / 已完成证据 | 解除条件 |
|---|---|---|---|
| ENV-001 | resolved | 独立 worktree 已初始化；原目录 96 文件字节/模式、index、HEAD、状态一致 | 见报告 INIT-01～04 |
| REF-001 | resolved | RustPBX 固定 commit 已取得并读出文件 | 见 reference-lock.json；不等于上游运行验证 |
| LAB-001 | pending | SIPp/tshark 缺失；真实 SIP 客户端、抓包权限、音频设备、合法样例未验证 | 第 00 章真实受控实验与证据 |
| CI-001 | blocked | macOS 完整检查通过；现有 Linux 容器缺 Rust/Python；远端 CI 未触发 | 同 tested_commit 的 Linux check.sh；远端运行另需用户安排 |
| REVIEW-001 | pending | 三份 ADR 为草案；范围与验收制度待人工确认 | 用户完成审阅；代理不代签 |
| LOCAL-001 | pending | 原目录 README 未提交修改含疑似凭据，未迁移或提交 | 用户撤销/轮换并自行处理本地内容；报告不保存值 |
| DOC-001 | resolved | 证据章目录符号链接可指向仓库外的缺陷已修复 | 新回归先失败，修复后 21 项通过；见报告 |

通信服务未实现；初始化测试不证明任何协议互通、网络资源释放或音频质量。
