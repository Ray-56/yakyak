# 当前工作

阶段：第 00 章；INIT-001、AC-00-01 与 AC-00-02 均已人工验收通过。
当前任务：[AC-00-02](tasks/AC-00-02.md) 已按用户明确确认登记 passed；固定提交的全新检出、双平台 CI 与人工确认记录见[验收报告](../acceptance/ch00/AC-00-02.md)，以 progress.json 为准。验收材料已提交、推送并创建 [PR #16](https://github.com/Ray-56/yakyak/pull/16)，尚未合入 main。[INIT-001](tasks/INIT-001.md) 与 [AC-00-01](tasks/AC-00-01.md) 保持 passed。

工作分支：`test/ch00-ac00-02`，工程基线为 `bb036b758ee06febbe76266ff73d10a4d9ff43c5`，被测实现提交为 `9ff5c3fb2dbefa076061cba0a93748f495e66792`。代码由用户亲自实现、提交与推送；本轮 Codex 按用户后续授权处理五份验收文档的提交、分支推送与 PR，并核对最终 head CI；不修改代码，不合并或开始其他 AC。

2026-10-18（Asia/Shanghai），用户在本会话明确确认 SCOPE v1.0 的功能范围、排除项及自研/复用边界。两个历史 tag 已分别在独立本地 clone 中检出并验证干净，参考 commit 沿用初始化时的真实对象与文件证据。用户授权的 Wireshark 4.0.7 → 4.6.9 更新已完成，旧版保留在仓库外私有备份中。

用户安装软件后，已实测 SIPp 3.8.0-TLS-PCAP、Linphone 6.2.3、Wireshark/TShark 4.6.9 的版本命令；两个 app 的签名和 macOS 信任检查均通过。SIPp `-v` 返回 99，为官方定义的正常无呼叫退出，已保留原始退出码。实际测试端版本缺项已补齐，见 [test-tools-lock.json](../references/test-tools-lock.json)及 [AC-00-01 记录](../acceptance/ch00/AC-00-01.md)。图形界面、协议场景、抓包与音频仍未验证；没有修改权限或系统 PATH。

2026-10-19，按用户授权创建证据提交 A：`41857037d1fda8b7b406534f84aca843db8f68af`，固定本项完整材料；后续提交 B 登记 A 的真实 SHA 与待审阅状态。提交 A 的干净工作区文档检查通过，后续登记的检查结果见验收记录；当时人工审阅字段为空。

随后按用户要求完成技术审阅：AC-00-01 材料齐全，两个旧版本重新独立检出，参考 commit 的四个文件重新取得且哈希一致。但在干净 B `2cea4ed0e4a65ca85953e13d5367ee0c06897e7a` 上运行 `bash scripts/check.sh` 退出 1：21 个 Python 测试中 20 过、1 失败，Rust 检查未执行。已定位为测试只在内存归一化进度、CLI 重读临时磁盘时缺报告的问题。状态回到进行中，详见 [技术审阅记录](../acceptance/ch00/AC-00-01-REVIEW.md)。

用户随后询问修复办法，已用 3 行代码将归一化进度写回测试临时文件，单独保存于 C `b24092bea144d04a42307a76a9b016a3da5c9447`。原失败项红转绿；C 的独立干净副本上完整检查通过：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建全部成功。DOC-002 已解决，状态恢复待审阅；生产校验与原断言未改变，原始失败证据保留。

2026-10-19，用户在步骤三人工验收说明后明确回复“通过”。已将 AC-00-01 记为 passed，审阅者为“用户（本会话明确确认）”，日期为 2026-10-19；复验提交保持 C，原始证据与历史失败记录保留。

初始化任务 [INIT-001](tasks/INIT-001.md) 已登记 passed。用户先要求补齐记录后自行审阅，随后在收到文档提交 `5cacd60f0c94292dcb4532528cf1f3441cccc805` 及 14 项对照表后回复“没问题”；审阅者为“用户（本会话明确确认）”，日期为 2026-10-19。INIT-11 后续环境证据仍来自主线 `63e8420b37b2cc30348d33d5244a805de60ae3af` 的[双平台 CI](https://github.com/Ray-56/yakyak/actions/runs/39194390065)，各有 21 个 Python、6 个 CLI 测试及格式/Clippy/release 通过。原始 `25157c6…` 在初始化交付时的 Linux/远端 CI not_run 保留，不冒称旧提交已验证。当时 125 项路线图中通过 1 项，其余 124 项为 not_started；INIT-001 不计入该 AC 总数。

## Git 收尾记录（2026-10-19）

初次核对时，本地 `docs/ch00-ac00-01-scope` 的验收登记提交为 `7d366eed0999c850ec2b864f853b2f2a7b19865b`；当时实际远端 `main` 和同名分支都仍为 `cbe33dece07b21b259e393061c4c6d6e19d0e165`，没有对应 PR，工作区干净、无 stash。上述本地验收登记提交已运行 `bash scripts/check.sh` 并退出 0：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建通过。

用户随后回复“ok”，明确授权推送、创建 PR、等待 Ubuntu/macOS CI 后普通 merge 合入并同步本地 main。已推送 `a5d5179e80a3148a4d549e27699ddf8df699b8c3` 并创建 [PR #14](https://github.com/Ray-56/yakyak/pull/14)；该提交的 [push CI](https://github.com/Ray-56/yakyak/actions/runs/39194109377) 与 [PR CI](https://github.com/Ray-56/yakyak/actions/runs/39194121363) 均在 ubuntu-24.04、macos-15 成功。该 SHA 的本地完整检查也已退出 0（21 个 Python、6 个 CLI 及格式/Clippy/release）。

最终分支提交 `0491442a50ed414cce42f928f027c0ed64074747` 的 push/PR 四项检查全部成功后，PR #14 已用普通 merge 合入，合并提交为 `63e8420b37b2cc30348d33d5244a805de60ae3af`；[主线 CI](https://github.com/Ray-56/yakyak/actions/runs/39194390065) 两平台也成功。本地 main 已 fast-forward 同步，交付时与实际远端 main 一致、工作区干净，验收引用的 C `b24092bea144d04a42307a76a9b016a3da5c9447` 保持祖先可达；历史分支、tag 和 worktree 均保留。最终执行结果保存在忽略的 `artifacts/ac00-01/20261019-git-closeout-vzi79j6w/final-closeout.json`。

## INIT-001 记录补齐（2026-10-19）

基于上述主线建立 `docs/init-001-review-evidence`，按用户授权补齐初始化记录与审阅材料。历史日志位于保留的 `../yakyak-bootstrap-20261016-160959/artifacts/bootstrap/`，已核对 44/44 文件哈希及 28 条原始检查结果；新复核记录位于忽略的 `artifacts/init-001/20261019-review-zdaregbc/`。审阅材料保存于 `5cacd60…`，用户确认后在 `b817778…` 单独登记人工验收结果；报告将历史执行与后续复核分开，三个 ADR 仍 Proposed。

用户随后明确授权 Git 交付，已推送本分支并创建 [PR #15](https://github.com/Ray-56/yakyak/pull/15)。本次补录后的最终 head 须等待 Ubuntu/macOS 全部 CI 成功，再普通 merge 并核对 main CI、本地与远端 main 一致、上述原始提交可达；保留历史分支、tag 和 worktree。实际 head、检查与合并状态以 PR 页面为准，最终执行记录归档至忽略的 `artifacts/init-001/20261019-git-closeout-wwflioo8/`。

2026-10-19 后续核对：PR #15 已于当日 14:41:10（Asia/Shanghai）普通 merge 合入，合并提交为 `bb036b758ee06febbe76266ff73d10a4d9ff43c5`；[main CI 39211342299](https://github.com/Ray-56/yakyak/actions/runs/39211342299) 的 Ubuntu/macOS 均成功。任务准备开始前，本地 main 与实际远端 main 一致、工作区干净；当时已在上述提交上的 `test/ch00-ac00-02` 分支。此前等待合入的安排已完成，原验收提交和历史记录保留。

## AC-00-02 任务准备（2026-10-19）

任务准备阶段，用户明确要求执行“创建任务卡、更新 CURRENT”这一步，并要求不提交代码。当时仅准备 [AC-00-02 任务卡](tasks/AC-00-02.md)，列出原验收、授权边界、已有 CI 与后续测试计划；只修改两份工作文档，`progress.json` 保持 not_started，没有产生实现提交或验收报告。用户随后自行以 `8e11a87` 提交任务文档。

当时的 main CI 各通过 21 个 Python、6 个 CLI 集成测试与格式/Clippy/release 构建；Rust 单元测试目标为 0 项。任务卡将补充有意义的 Rust 单元测试列为后续缺口，旧 CI 仅作为基线。三份 ADR 状态保持 Proposed。

任务准备阶段的文档检查：`python3 scripts/roadmap.py validate --git` 退出 0（27 章、125 项目标及已有提交引用结构有效）；`python3 scripts/roadmap.py render --check` 退出 0（30 份生成文件一致）；`git diff --check` 退出 0。当时没有执行测试用例。

## AC-00-02 实现与待审阅登记（2026-10-19）

用户亲自提取 CLI 参数判定并增加两项 Rust 单元测试，修复开发中的输出、退出码、换行及尾表达式问题后，本地完整检查通过；用户自行以 `9ff5c3fb2dbefa076061cba0a93748f495e66792` 提交并推送。本报告使用固定提交的最终证据，不将未提交草稿的失败或较早工程基线的成功混记为该提交结果。

固定提交的本地独立全新 clone 检查前后均 clean；doctor、完整检查与测试清单均退出 0。随后核对 [push CI 39232969429](https://github.com/Ray-56/yakyak/actions/runs/39232969429)：Ubuntu 24.04.5 x86_64 与 macOS 15.7.9 arm64 均全新检出同一完整 SHA，各通过 21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试，以及格式、Clippy、release 构建。当时实际远端分支与本地被测 SHA 一致，没有 PR、未合入 main。

已整理[验收报告](../acceptance/ch00/AC-00-02.md)，记录原文、环境、命令、实际数量、限制和原始日志 SHA-256；本地复验与 CI 日志保存在忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/`。当时仅将 AC-00-02 登记 ready_for_review，人工审阅字段留空；用户此前表示运行成功、自行实现或推送均未被视为整项人工验收。当时 125 项中 1 项 passed、1 项 ready_for_review、123 项 not_started，其他 AC 与三份 ADR 不变。

登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章、125 项目标及提交/报告引用有效，30 份生成文件一致，21 个 Python 回归全部通过。原实现固定提交的 Rust 与 release 结果沿用上述完整复验与新 CI，本轮未重复运行；本次新增验收文档尚未提交。

## AC-00-02 人工验收（2026-10-19）

用户在收到验收报告、技术复验结果与明确人工确认说明后回复“通过”。据此将 AC-00-02 登记 passed，人工审阅者为“用户（本会话明确确认）”，确认日期为 2026-10-19（Asia/Shanghai）；被测提交保持 `9ff5c3fb2dbefa076061cba0a93748f495e66792`，原始证据、哈希和 CI 链接保留。

人工验收登记时，125 项中 2 项 passed、123 项 not_started，其他 AC 与三份 Proposed ADR 未改变。当时验收文档尚未提交、没有 PR，未合入 main；该次人工确认本身不包含 Git 交付或开始下一 AC 的授权。

人工确认登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章、125 项目标及提交/报告引用有效，30 份生成文件一致，21 个 Python 回归全部通过。本轮未重跑 Rust 测试、release 构建或远端 CI，被测提交和既有完整证据保持不变。

## AC-00-02 Git 交付（2026-10-19）

在收到“提交这五份验收记录，再创建 PR”的下一步说明后，用户回复“这些你可以帮我处理”，另行授权 Codex 暂存、提交验收文档，推送当前分支并创建 PR。本轮保留 `9ff5c3fb2dbefa076061cba0a93748f495e66792` 与原始证据，核对交付提交的 Ubuntu/macOS CI；合并不在本次范围内。Git 操作及最终核对记录保留在忽略的 artifacts 目录。

已将五份验收材料保存为 `323fd88d907536df71e42f66fc06606a56cef739` 并推送，创建 [PR #16](https://github.com/Ray-56/yakyak/pull/16)。该材料提交前 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试及格式/Clippy/release 构建通过；`validate --git` 与 diff 空白检查也通过。初次日志重定向使用了不存在的目录，命令退出 1 且检查未启动；随后在正确目录重新运行并成功，没有将路径错误计为测试失败或通过。

本次补录提交保留上述材料提交和被测实现提交；最终 PR head 的 push/PR 双平台检查以 [PR Checks](https://github.com/Ray-56/yakyak/pull/16/checks) 为准，实际 SHA、状态与原始日志归档至忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/git-delivery-58ed1pbl/`。PR 保持打开，未执行合并、分支删除或下一 AC。

## 后续安排

1. 以 PR #16 的最终 head 核对 push/PR 双平台 CI，保留材料提交、被测提交及已有证据。
2. 等待用户安排 PR #16 合并或新任务；不因 AC-00-02 通过自动合并或开始下一项。
3. 另行审阅三个 ADR 草案与证据流程；本次初始化验收没有将 ADR 改为 Accepted。
4. 用户另行指定后，补齐 AC-00-03 受控发包、抓包权限、音频设备和合法样例；已有工具版本记录可复用。逐项完成第 00 章验收后，再由用户明确安排 AC-01-01。

本轮处理 AC-00-02 已授权的文档提交、推送与 PR；源码与其他 AC、ADR 状态不变。原实现验收仍引用固定提交的证据，新交付 head 的检查单独核对；网络与音频实验未运行。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告“后续授权：直接同步 main”一节。
