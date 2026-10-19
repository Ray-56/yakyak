# 已知问题与未满足条件

状态依据：[INIT-001 实际报告](../handoff/BOOTSTRAP-REPORT.md)及 [AC-00-01 部分验收报告](../acceptance/ch00/AC-00-01.md)。

| ID | 状态 | 问题 / 已完成证据 | 解除条件 |
|---|---|---|---|
| ENV-001 | resolved | 独立 worktree 已初始化；原目录 96 文件字节/模式、index、HEAD、状态一致 | 见报告 INIT-01～04 |
| REF-001 | resolved | RustPBX 固定 commit 已取得并读出文件 | 见 reference-lock.json；不等于上游运行验证 |
| LAB-001 | pending | 2026-10-18 已核实 SIPp 3.8.0、Linphone 6.2.3、Wireshark/TShark 4.6.9；两款 app 的签名与 macOS 信任检查通过。工具版本已记录，图形界面、抓包权限、音频设备、合法 SIP 样例及对应抓包仍未验证 | 完成 AC-00-03 受控实验与证据；当前版本见 test-tools-lock.json |
| CI-001 | blocked | macOS 完整检查通过；现有 Linux 容器缺 Rust/Python；初始化交付时远端 CI 未触发，后续授权 main 推送将触发检查 | 核对 main 推送后的实际 CI 结果；Linux 验收需真实成功记录 |
| REVIEW-001 | pending | 用户于 2026-10-18 确认 SCOPE v1.0 功能范围、排除项及自研/复用边界；三份 ADR 与验收制度仍待人工确认 | 用户另行审阅 ADR 与验收制度；整个 AC 的验收单独记录 |
| LOCAL-001 | pending | 原目录 README 修改含疑似凭据；main 同步前已转存仓库外私有备份，未进入 Git | 用户撤销/轮换并自行处理本地内容；报告不保存值 |
| DOC-001 | resolved | 证据章目录符号链接可指向仓库外的缺陷已修复 | 新回归先失败，修复后 21 项通过；见报告 |

通信服务未实现；初始化测试不证明任何协议互通、网络资源释放或音频质量。
