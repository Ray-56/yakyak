# 当前工作

阶段：第 00 章；INIT-001 与 AC-00-01 均已人工验收通过。
当前任务：[INIT-001](tasks/INIT-001.md) 已登记 passed，等待用户安排后续 Git 交付或下一任务；[AC-00-01](tasks/AC-00-01.md) 保持 passed，以 progress.json 为准。

2026-10-18（Asia/Shanghai），用户在本会话明确确认 SCOPE v1.0 的功能范围、排除项及自研/复用边界。两个历史 tag 已分别在独立本地 clone 中检出并验证干净，参考 commit 沿用初始化时的真实对象与文件证据。用户授权的 Wireshark 4.0.7 → 4.6.9 更新已完成，旧版保留在仓库外私有备份中。

用户安装软件后，已实测 SIPp 3.8.0-TLS-PCAP、Linphone 6.2.3、Wireshark/TShark 4.6.9 的版本命令；两个 app 的签名和 macOS 信任检查均通过。SIPp `-v` 返回 99，为官方定义的正常无呼叫退出，已保留原始退出码。实际测试端版本缺项已补齐，见 [test-tools-lock.json](../references/test-tools-lock.json)及 [AC-00-01 记录](../acceptance/ch00/AC-00-01.md)。图形界面、协议场景、抓包与音频仍未验证；没有修改权限或系统 PATH。

2026-10-19，按用户授权创建证据提交 A：`41857037d1fda8b7b406534f84aca843db8f68af`，固定本项完整材料；后续提交 B 登记 A 的真实 SHA 与待审阅状态。提交 A 的干净工作区文档检查通过，后续登记的检查结果见验收记录；当时人工审阅字段为空。

随后按用户要求完成技术审阅：AC-00-01 材料齐全，两个旧版本重新独立检出，参考 commit 的四个文件重新取得且哈希一致。但在干净 B `2cea4ed0e4a65ca85953e13d5367ee0c06897e7a` 上运行 `bash scripts/check.sh` 退出 1：21 个 Python 测试中 20 过、1 失败，Rust 检查未执行。已定位为测试只在内存归一化进度、CLI 重读临时磁盘时缺报告的问题。状态回到进行中，详见 [技术审阅记录](../acceptance/ch00/AC-00-01-REVIEW.md)。

用户随后询问修复办法，已用 3 行代码将归一化进度写回测试临时文件，单独保存于 C `b24092bea144d04a42307a76a9b016a3da5c9447`。原失败项红转绿；C 的独立干净副本上完整检查通过：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建全部成功。DOC-002 已解决，状态恢复待审阅；生产校验与原断言未改变，原始失败证据保留。

2026-10-19，用户在步骤三人工验收说明后明确回复“通过”。已将 AC-00-01 记为 passed，审阅者为“用户（本会话明确确认）”，日期为 2026-10-19；复验提交保持 C，原始证据与历史失败记录保留。

初始化任务 [INIT-001](tasks/INIT-001.md) 已登记 passed。用户先要求补齐记录后自行审阅，随后在收到文档提交 `5cacd60f0c94292dcb4532528cf1f3441cccc805` 及 14 项对照表后回复“没问题”；审阅者为“用户（本会话明确确认）”，日期为 2026-10-19。INIT-11 后续环境证据仍来自主线 `63e8420b37b2cc30348d33d5244a805de60ae3af` 的[双平台 CI](https://github.com/Ray-56/yakyak/actions/runs/39194390065)，各有 21 个 Python、6 个 CLI 测试及格式/Clippy/release 通过。原始 `25157c6…` 在初始化交付时的 Linux/远端 CI not_run 保留，不冒称旧提交已验证。125 项路线图中仍仅通过 1 项，其余 124 项保持 not_started；INIT-001 不计入该 AC 总数。

## Git 收尾记录（2026-10-19）

初次核对时，本地 `docs/ch00-ac00-01-scope` 的验收登记提交为 `7d366eed0999c850ec2b864f853b2f2a7b19865b`；当时实际远端 `main` 和同名分支都仍为 `cbe33dece07b21b259e393061c4c6d6e19d0e165`，没有对应 PR，工作区干净、无 stash。上述本地验收登记提交已运行 `bash scripts/check.sh` 并退出 0：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建通过。

用户随后回复“ok”，明确授权推送、创建 PR、等待 Ubuntu/macOS CI 后普通 merge 合入并同步本地 main。已推送 `a5d5179e80a3148a4d549e27699ddf8df699b8c3` 并创建 [PR #14](https://github.com/Ray-56/yakyak/pull/14)；该提交的 [push CI](https://github.com/Ray-56/yakyak/actions/runs/39194109377) 与 [PR CI](https://github.com/Ray-56/yakyak/actions/runs/39194121363) 均在 ubuntu-24.04、macos-15 成功。该 SHA 的本地完整检查也已退出 0（21 个 Python、6 个 CLI 及格式/Clippy/release）。

最终分支提交 `0491442a50ed414cce42f928f027c0ed64074747` 的 push/PR 四项检查全部成功后，PR #14 已用普通 merge 合入，合并提交为 `63e8420b37b2cc30348d33d5244a805de60ae3af`；[主线 CI](https://github.com/Ray-56/yakyak/actions/runs/39194390065) 两平台也成功。本地 main 已 fast-forward 同步，交付时与实际远端 main 一致、工作区干净，验收引用的 C `b24092bea144d04a42307a76a9b016a3da5c9447` 保持祖先可达；历史分支、tag 和 worktree 均保留。最终执行结果保存在忽略的 `artifacts/ac00-01/20261019-git-closeout-vzi79j6w/final-closeout.json`。

## INIT-001 记录补齐（2026-10-19）

基于上述主线建立 `docs/init-001-review-evidence`，按用户授权补齐初始化记录与审阅材料。历史日志位于保留的 `../yakyak-bootstrap-20261016-160959/artifacts/bootstrap/`，已核对 44/44 文件哈希及 28 条原始检查结果；新复核记录位于忽略的 `artifacts/init-001/20261019-review-zdaregbc/`。审阅材料保存于 `5cacd60…`，用户确认后单独登记人工验收结果；报告将历史执行与后续复核分开，三个 ADR 仍 Proposed。当前分支尚未推送或合入 main。

## 后续安排

1. INIT-001 人工验收已完成；按用户后续安排处理当前文档分支的推送、PR 与合入，保留被审阅提交哈希。
2. 用户另行指定后，为 AC-00-02 单独安排任务卡与分支，核对全新检出、Linux/CI 验收条件；本轮尚未开始。已有工具版本记录可复用于 AC-00-03。
3. 另行审阅三个 ADR 草案与证据流程；本次初始化验收没有将 ADR 改为 Accepted。
4. 补齐 AC-00-03 受控发包、抓包权限、音频设备和合法样例；逐项完成第 00 章验收后，再由用户明确安排 AC-01-01。

本轮按用户明确回复登记 INIT-001 的人工验收、审阅者和日期；只修改报告、INIT-001 任务卡与 CURRENT，未推进其他 AC 或修改 ADR 状态。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告“后续授权：直接同步 main”一节。
