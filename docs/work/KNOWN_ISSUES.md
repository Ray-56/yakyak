# 已知问题与未满足条件

当前状态依据：[进度真源](../roadmap/progress.json)、各项验收报告及 ADR；初始化历史依据见 [INIT-001 实际报告](../handoff/BOOTSTRAP-REPORT.md)。2026-10-21 已按 AC-00-05 第 3 项核对记录，随后按用户对 ADR-0001/0003 的单独确认更新 REVIEW-001；保留历史失败与未解决事项。

| ID | 状态 | 问题 / 已完成证据 | 解除条件 |
|---|---|---|---|
| ENV-001 | resolved | 独立 worktree 已初始化；原目录 96 文件字节/模式、index、HEAD、状态一致 | 见报告 INIT-01～04 |
| REF-001 | resolved | RustPBX 固定 commit 已取得并读出文件 | 见 reference-lock.json；不等于上游运行验证 |
| LAB-001 | resolved | 2026-10-20 固定配置 C `0d972857cc02e51d21d9c02c48dfd6dc40d4d535` 的全新 clone 基础检查及一次独立 SIPp/lo0 复验成功，6 包与双端日志匹配、三进程退出 0、端口释放；19/19 离线样本断言通过。Linphone 设备沿用用户确认；通话内媒体未测 | 用户收到[完整报告](../acceptance/ch00/AC-00-03.md)与整项确认说明后回复“没问题”，AC-00-03 已于 2026-10-20 人工验收 passed；本条实验室准备缺口解除，不外推为媒体互通通过；工具版本见 test-tools-lock.json |
| CI-001 | resolved | AC-00-01 交付提交 `a5d5179e80a3148a4d549e27699ddf8df699b8c3` 的 [push CI 39194109377](https://github.com/Ray-56/yakyak/actions/runs/39194109377) 与 [PR CI 39194121363](https://github.com/Ray-56/yakyak/actions/runs/39194121363) 均在 ubuntu-24.04、macos-15 成功 | 该次交付的 CI 缺口已解除；最终 head、[PR #14](https://github.com/Ray-56/yakyak/pull/14) 合入与主线双平台 CI 结果见 [CURRENT 历史交付记录](CURRENT.md)。这些历史结果不代表 AC-00-05 当前分支已运行 CI；本轮仅核对既有记录 |
| REVIEW-001 | resolved | SCOPE v1.0 已于 2026-10-18 确认；[ADR-0002](../architecture/adr/0002-protocol-runtime-time.md) 于 2026-10-20 Accepted。验收模板与三条记录规则已于 2026-10-21 确认沿用，AC-00-05 的[正式报告](../acceptance/ch00/AC-00-05.md)已于当日人工验收 passed；用户随后单独回复“同意01 和03”，[ADR-0001](../architecture/adr/0001-small-core.md)、[ADR-0003](../architecture/adr/0003-documentation-evidence.md) 据此于 2026-10-21 Accepted | 两份 ADR 的用户确认记录已分别保存；本次确认独立于 AC-00-05 整项验收，不扩展为凭据处理完成或后续 AC 实现授权 |
| LOCAL-001 | pending | 原目录 README 修改含疑似凭据；main 同步前已转存仓库外私有备份，未进入 Git | 用户撤销/轮换并自行处理本地内容；报告不保存值 |
| DOC-001 | resolved | 证据章目录符号链接可指向仓库外的缺陷已修复 | 新回归先失败，修复后 21 项通过；见报告 |
| DOC-002 | resolved | C `b24092bea144d04a42307a76a9b016a3da5c9447` 将归一化进度写回临时文件；原用例红转绿，独立 clean 副本完整检查通过：21 个 Python、6 个 CLI 及格式/Clippy/release 构建 | 见 AC-00-01-REVIEW.md 的原失败、定位与完整复验记录；原断言及生产校验保持不变 |

通信服务未实现；初始化测试不证明任何协议互通、网络资源释放或音频质量。
