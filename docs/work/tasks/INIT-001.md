# INIT-001：保留历史的仓库初始化

状态：passed（用户已审阅 INIT-01～14 并确认）。执行者：Codex。人工审阅者：用户（本会话明确确认）。人工审阅日期：2026-10-19（Asia/Shanghai）。范围：仅工程初始化。

## 目标

按`../../handoff/BOOTSTRAP.md`的INIT-01～INIT-14交付安全新基线。旧代码可恢复，原工作区不变；新工程只含无网络CLI壳、测试、检查工具、CI与文档。

## 输入

用户当前本地仓库、此次交接包、已确认27章/125目标。真实Git基点与工具版本现场采集。

## 明确不做

SIP、SDP、RTP、事务、注册、Dialog、B2BUA、WebRTC、数据库、后台、AI及任何付费/公网通话。不能提前实现AC-01-01。

## 执行

只读盘点 → 验证备份 → 独立worktree → 审核迁移清单 → 导入文档 → 最小工程/脚本/CI → 正反向验证 → 真实报告 → 停止。

## 验收与证据

完整14项标准以BOOTSTRAP为准，不复制另一套更弱要求。报告路径`docs/handoff/BOOTSTRAP-REPORT.md`。每项必须有命令/结果/证据；不可运行的项标blocked/not_run并说明解除条件。

本任务可交付可用骨架与明确阻塞，但第00章验收必须按原要求逐项处理。用户未确认前不得签署passed。

## 原始交付（2026-10-16）

实现与验证基线：`25157c636873f839c133c923c5a500af66d8f4c9`。6 个真实 CLI 测试、21 个 Python 回归及 macOS 完整检查通过；负面验证确认漂移、坏状态/ID、无证据通过、引导断言失败、零测试和缺失工具链均非零。证据、命令和 14 项结果见[报告](../../handoff/BOOTSTRAP-REPORT.md)。

当时 Linux 和远端 CI 未运行，125 项功能状态未更改；这是原始被测提交的历史记录，不能用后续提交的结果覆盖。

## 后续补录（2026-10-19）

用户授权补齐记录后再自行审阅。后续复核基线为主线 `63e8420b37b2cc30348d33d5244a805de60ae3af`；[CI run 39194390065](https://github.com/Ray-56/yakyak/actions/runs/39194390065) 的 Ubuntu 24.04.5 x86_64、macOS 15.7.9 arm64 两项均成功。日志确认两个 checkout 的完整 SHA 一致，均运行 doctor/check，分别通过 21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建。原始 `25157c6…` 的 Linux/远端 CI `not_run` 保留。

原始提交至复核基线的工程路径仅增加 `test_roadmap.py` 的 3 行临时 fixture 落盘；生产源码、CLI 测试、检查脚本及 CI 未变。保留初始化 worktree 内的 44 份历史证据逐文件哈希一致，28 条原始检查退出码均符合预期；没有重做手工负面注入。当前进度为 AC-00-01 已由用户确认 passed、其余 124 项 not_started。

本次使用文档分支 `docs/init-001-review-evidence`，只更新本任务卡、初始化报告与 CURRENT。完整 14 项“标准—结果—证据—限制”对照表及实际日志入口见[报告文末的后续复核与人工验收](../../handoff/BOOTSTRAP-REPORT.md)。原始日志保留在忽略目录；审阅材料提交为 `5cacd60f0c94292dcb4532528cf1f3441cccc805`，记录补齐时曾保持 ready_for_review。

用户随后在收到审阅表和本地提交信息后明确回复“没问题”，据此将 INIT-001 记为 passed；审阅者与日期见页首，原始被测提交和后续复核基线不变。

Git 交付：用户明确授权后已推送本分支并创建 [PR #15](https://github.com/Ray-56/yakyak/pull/15)。本次补录后的最终 head 须在 Ubuntu/macOS 全部 CI 成功后普通 merge，保留 `5cacd60…` 与 `b817778…` 原始提交；合入后核对 main CI 与本地同步。实际交付状态见 PR 页面，执行记录归档至忽略的 `artifacts/init-001/20261019-git-closeout-wwflioo8/`。

下一步：Git 交付完成后等待用户安排新任务。三份 ADR、证据制度及实验室仍需后续独立审阅；本次不开始或验收 AC-00-02，也不实现通信核心。
