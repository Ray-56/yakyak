# 已知问题与未满足条件

状态依据：[INIT-001 实际报告](../handoff/BOOTSTRAP-REPORT.md)及 [AC-00-01 部分验收报告](../acceptance/ch00/AC-00-01.md)。

| ID | 状态 | 问题 / 已完成证据 | 解除条件 |
|---|---|---|---|
| ENV-001 | resolved | 独立 worktree 已初始化；原目录 96 文件字节/模式、index、HEAD、状态一致 | 见报告 INIT-01～04 |
| REF-001 | resolved | RustPBX 固定 commit 已取得并读出文件 | 见 reference-lock.json；不等于上游运行验证 |
| LAB-001 | pending | 2026-10-20 工具版本与签名记录保留；用户手动完成 SIPp 本机收发/lo0 抓包，6 个 SIP 包与双端日志匹配、退出码均为 0。Linphone 本地设备检查获用户确认正常，截图可见默认设备与绿色电平；动态电平及实际听音依据用户确认，通话内媒体未测 | 固定场景/命令/版本、干净复现及完整验收材料与人工审阅仍待完成；见 AC-00-03 任务卡，当前版本见 test-tools-lock.json |
| CI-001 | resolved | AC-00-01 提交 `a5d5179e80a3148a4d549e27699ddf8df699b8c3` 的 [push CI 39194109377](https://github.com/Ray-56/yakyak/actions/runs/39194109377) 与 [PR CI 39194121363](https://github.com/Ray-56/yakyak/actions/runs/39194121363) 均在 ubuntu-24.04、macos-15 成功 | 已取得当前分支独立双平台证据；补录后最终 head 检查与合入信息见 [PR #14](https://github.com/Ray-56/yakyak/pull/14)，合入前须核对该 head 的全部 CI，合入后另核对 main CI |
| REVIEW-001 | pending | 用户于 2026-10-18 确认 SCOPE v1.0 功能范围、排除项及自研/复用边界；三份 ADR 与验收制度仍待人工确认 | 用户另行审阅 ADR 与验收制度；整个 AC 的验收单独记录 |
| LOCAL-001 | pending | 原目录 README 修改含疑似凭据；main 同步前已转存仓库外私有备份，未进入 Git | 用户撤销/轮换并自行处理本地内容；报告不保存值 |
| DOC-001 | resolved | 证据章目录符号链接可指向仓库外的缺陷已修复 | 新回归先失败，修复后 21 项通过；见报告 |
| DOC-002 | resolved | C `b24092bea144d04a42307a76a9b016a3da5c9447` 将归一化进度写回临时文件；原用例红转绿，独立 clean 副本完整检查通过：21 个 Python、6 个 CLI 及格式/Clippy/release 构建 | 见 AC-00-01-REVIEW.md 的原失败、定位与完整复验记录；原断言及生产校验保持不变 |

通信服务未实现；初始化测试不证明任何协议互通、网络资源释放或音频质量。
