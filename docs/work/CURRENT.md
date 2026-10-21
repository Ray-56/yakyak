# 当前工作

阶段：第 00 章；INIT-001、AC-00-01、AC-00-02、AC-00-03 与 AC-00-04 均已人工验收通过。
当前任务：[AC-00-05](tasks/AC-00-05.md) in_progress。2026-10-21，第 1～3 项已完成；用户在第 4 项执行说明后回复“好的，你来处理”，本轮开始固定本地文档提交 C、独立干净复验和证据保存。第 5 项正式报告尚未开始，不提前登记整项待审阅或通过。

当前分支：`docs/ch00-ac00-05`，工作目录为用户原项目目录；准备基线 `8720de6c2ad8b62f6ad491497984a25bc2d42385`。`git fetch origin` 后确认本地 main 与 origin/main 相同、工作区干净，再创建并切换分支。未提交或推送；后续文档范围和检查计划见任务卡。

第 2 项已新增[不支持功能表](../UNSUPPORTED.md)，在根 README 和文档地图添加入口，依据已确认的 SCOPE v1.0 记录“范围内尚未实现”和“范围排除”，并列出对应章节、验证边界及更新方式。普通录音、PCM、DTMF 等保留在范围内；测试端准备不计为 YakYak 通信支持。

第 3 项核对：初始提交 `f014ea61de4aea872742e266ce398a8b06d65f5f` 的 125 项全部 not_started，提交/证据/审阅字段为空；前四项固定提交均真实且为当前 HEAD 的祖先，进度、五份报告和既有人工确认一致。独立代理核对相应本地证据引用存在，AC-00-03 清单 73/73、AC-00-04 清单 42/42 文件均存在；未重算原始哈希或重跑历史实验。问题清单过期描述已修正，8 条记录保留 6 resolved、2 pending；待确认 ADR 和凭据处理的解除条件保留，详见任务卡与 [KNOWN_ISSUES](KNOWN_ISSUES.md)。

第三项通过与整项人工验收分开记录：本项状态保持 in_progress，被测提交、证据、整项审阅字段在执行前仍为空。SCOPE、targets.json、AC-00-01～04 的通过状态和 ADR 状态不变。本轮仅获第 4 项本地固定与复验授权，不实现通信代码或推送。

第 4 项执行前：本项 8 份准备文档已完整备份并核对，index 为空、无其他改动；备份位于忽略的 `artifacts/ch00/ac00-05/20261021-stage4-preparation-oo4e4di0/`。独立代理只读预审未发现必须修正的实质问题。先创建 C，再以完整 C 新建本地独立 clone，记录前后 clean，执行文档校验、生成检查、现有 Python 回归及差异检查，保存日志、环境和哈希；实际结果在 C 之后登记，不预写通过或自行开始第 5 项。

第三项登记检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 及一次性链接/范围核对均退出 0，27 章/125 项有效、30 个生成视图一致，3 份工作文档的 34 个本地链接存在。测试执行 0 项，未运行回归、构建、CI、网络或音频实验。历史证据只做记录与元数据核对，不冒称重新测试；命令与一次无匹配搜索说明见任务卡。

第二项检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 及一次性链接/范围检查均退出 0：27 章/125 项有效，30 个生成视图一致，4 份 Markdown 的 50 个本地链接目标存在，12 个尚未实现能力组和 5 个范围排除组均已记录。独立代理对照 SCOPE 未发现实质问题。测试执行 0 项，未运行回归、构建、CI、网络或音频实验；未提交工作区的检查不替代第 4 步固定提交复验，详情见任务卡。

第一项登记（历史）的 render、validate --git、render --check、差异和范围检查均退出 0：27 章/125 项有效，30 个生成视图一致；仅本项进度改变，模板、ADR 与其他目标未变。测试执行 0 项，未运行回归、构建、CI 或协议实验；不是固定提交上的正式验收，详见任务卡。

Git 准备阶段（历史）的 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项与已有提交引用有效，30 个生成视图一致。测试执行 0 项；未运行 Python/Rust/CLI 回归、构建、远端 CI 或协议实验。检查与一次只读路径错误的说明见任务卡；这些结果不构成本项正式验收。

上一任务：[AC-00-04](tasks/AC-00-04.md) passed，[PR #20](https://github.com/Ray-56/yakyak/pull/20) 已合入 main，主线双平台 CI 成功。固定设计 C `aab3180fb645ed46972d683e7c1654d46e5b7e47`、初次报告 D `00a829f34202cb6b8f6d2870fb07d8b2437c20c3` 和人工登记 `41728e2ea1798a61c47739c0753586e03b3cebd8` 保留；完整结果见[验收报告](../acceptance/ch00/AC-00-04.md)。上一轮按用户“帮我操作一下”的授权完成 Git 与工作区收尾，当时未开始 AC-00-05。

上一轮收尾分支：`docs/implementation-authorization`，工作目录为用户原项目目录；三份既有授权约定改动已原样保存为 `eb567abd2b56cff41980840dce50c917d888e9f1`，与 AC-00-04 的 PR #20 分开交付。旧 AC-00-04 工作树已完整归档后移除；主仓库 Git 历史和初始化工作树保留。该独立文档 PR 在双平台 CI 通过后普通 merge，最终同步回 main；实际最终 SHA、PR、CI 与 clean 核对见下文收尾回执目录。

AC-00-03 的 passed、配置 C `0d972857cc02e51d21d9c02c48dfd6dc40d4d535` 及原始证据不变，完整交付历史见下文与[报告](../acceptance/ch00/AC-00-03.md)。下文各轮“本轮未开始下一 AC”等描述属于对应日期和阶段的历史边界。

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

登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章、125 项目标及提交/报告引用有效，30 份生成文件一致，21 个 Python 回归全部通过。原实现固定提交的 Rust 与 release 结果沿用上述完整复验与新 CI，该轮未重复运行；当时新增验收文档尚未提交。

## AC-00-02 人工验收（2026-10-19）

用户在收到验收报告、技术复验结果与明确人工确认说明后回复“通过”。据此将 AC-00-02 登记 passed，人工审阅者为“用户（本会话明确确认）”，确认日期为 2026-10-19（Asia/Shanghai）；被测提交保持 `9ff5c3fb2dbefa076061cba0a93748f495e66792`，原始证据、哈希和 CI 链接保留。

人工验收登记时，125 项中 2 项 passed、123 项 not_started，其他 AC 与三份 Proposed ADR 未改变。当时验收文档尚未提交、没有 PR，未合入 main；该次人工确认本身不包含 Git 交付或开始下一 AC 的授权。

人工确认登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章、125 项目标及提交/报告引用有效，30 份生成文件一致，21 个 Python 回归全部通过。本轮未重跑 Rust 测试、release 构建或远端 CI，被测提交和既有完整证据保持不变。

## AC-00-02 Git 交付（2026-10-19）

在收到“提交这五份验收记录，再创建 PR”的下一步说明后，用户回复“这些你可以帮我处理”，另行授权 Codex 暂存、提交验收文档，推送当前分支并创建 PR。本轮保留 `9ff5c3fb2dbefa076061cba0a93748f495e66792` 与原始证据，核对交付提交的 Ubuntu/macOS CI；合并不在本次范围内。Git 操作及最终核对记录保留在忽略的 artifacts 目录。

已将五份验收材料保存为 `323fd88d907536df71e42f66fc06606a56cef739` 并推送，创建 [PR #16](https://github.com/Ray-56/yakyak/pull/16)。该材料提交前 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试及格式/Clippy/release 构建通过；`validate --git` 与 diff 空白检查也通过。初次日志重定向使用了不存在的目录，命令退出 1 且检查未启动；随后在正确目录重新运行并成功，没有将路径错误计为测试失败或通过。

补录后的最终 PR head 为 `03600b3e94770a29a57ef8a90443ebc28209aa5c`，push/PR 双平台四项检查全部成功，材料提交和被测实现提交保留。实际 SHA、状态与原始日志归档至忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/git-delivery-58ed1pbl/`。该轮交付结束时 PR 保持打开，Codex 未执行合并、分支删除或下一 AC。

## AC-00-02 合并收尾（2026-10-19）

用户随后自行合并 PR #16，实际合并时间为 2026-10-19 18:18:09（Asia/Shanghai），合并提交为 `78686a8ae1520e61f5ff16241409823495bbd066`。[主线 CI 39236480386](https://github.com/Ray-56/yakyak/actions/runs/39236480386) 的 Ubuntu/macOS 均成功：各 21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试，以及格式、Clippy、release 构建通过。

收尾开始时工作区干净。已 fetch main 并使用 `git merge --ff-only origin/main` 将本地 main 同步到上述实际远端 SHA；被测实现 `9ff5c3f…`、验收材料 `323fd88…` 和 PR head `03600b3…` 均保持祖先可达，未删除任何分支、tag 或 worktree。Git 同步与主线 CI 日志保存在忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/merge-closeout-j7b36ffj/`。补录时四份收尾文档暂留本地，随后用户授权将其在独立 AC-00-02 分支提交并创建 PR。

合并核对阶段的文档检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0；27 章、125 项目标与提交/报告引用有效，30 份生成文件一致，无空白错误。该阶段未重跑本地测试，主线测试数量与结果依据上述 CI 原始日志。

用户指出这四份文档全部属于 AC-00-02，随后回复“好的，帮我处理一下”，授权单独建立 `docs/ch00-ac00-02-closeout`、提交、推送和创建 PR。只处理验收报告、progress.json 的 note、CURRENT 与 AC-00-02 任务卡；不混入 AC-00-03。收尾分支的 PR 与 CI 结果以 GitHub 对应分支记录为准，原始交付记录保存在忽略的 artifacts 中。

独立收尾分支的提交前完整检查 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试及格式/Clippy/release 构建通过；`validate --git` 与 `git diff --check` 也退出 0。提交前仅上述四份文档存在改动，所有 AC 状态、被测实现与原始证据不变；新提交的 CI 单独核对。原始输出与交付记录保存在忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/docs-closeout-__kh_hjv/`。

## AC-00-03 任务准备（2026-10-20）

开始时本地 main 为 `0bb5d017a791e2834a12efe85b7ed3d33890799f`，与本轮 `git ls-remote` 查询的实际远端 main 一致，工作区干净。此前只读核查已确认 AC-00-02 独立收尾 PR #17 合入，以及 [main CI 39239117830](https://github.com/Ray-56/yakyak/actions/runs/39239117830) 双平台成功；该历史 CI 不能证明 AC-00-03 实验通过，本轮未重查 CI。

按用户请求完成第 1 项：建立 `test/ch00-ac00-03` 分支，创建 [AC-00-03 任务卡](tasks/AC-00-03.md)，记录原验收、已有工具材料、待定拓扑、操作顺序、可观察结果及证据计划，并将 CURRENT 指向该卡。只修改两份工作文档；progress.json 保持 not_started，未创建验收报告或实验产物。

工具版本沿用已有记录，GUI、真实 SIP 场景、抓包权限与音频仍未验证。用户下一步先检查工具可用性；受控端点、端口、抓包接口及场景授权/配额/保留期须在实际实验前具体确定，本轮没有执行网络或音频操作。首次按文件名查找新任务卡时无匹配、退出 1；随后通过任务目录清单确认卡尚未存在，不属于测试失败或通过。

本轮文档检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check` 与 `git diff --check` 均退出 0，27 章、125 项目标结构与提交引用有效，30 份生成文件一致；另用一次性 Python 断言检查两份工作文档的 20 个本地文件链接、行尾空白和文件末尾换行，退出 0。这些是文档检查，未执行 Python/Rust 测试用例、完整基础检查或新 CI，不代表 AC-00-03 实验通过。

## AC-00-03 工具启动反馈（2026-10-20）

用户在收到 SIPp 版本与帮助命令、Linphone 启动、Wireshark 窗口与接口列表的三项手动检查说明后回复“都没有问题”。仅据此补录用户确认工具启动检查无问题；用户未附原始输出，本轮未新增实际版本、退出码或代理独立复验结果，历史工具锁记录保持原样。该反馈不包含实际抓包权限、SIP 收发、音频验证或整个 AC 的人工验收。

任务卡补入本机候选方案：SIPp 发送进程与独立 SIPp 接收进程经 loopback 通信，端口占用、具体地址/端口/协议、抓包接口与范围、场景授权及证据配额/保留期仍待确定。只补录两份工作文档，progress.json 保持 not_started；Codex 未运行 GUI、发包、抓包或音频操作，未提交或推送。

补录后 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0（27 章/125 项结构有效、30 份生成文件一致）；两份文档的 20 个本地文件链接与空白检查也通过。没有运行 Rust/Python 测试用例或实验。

## AC-00-03 第 3 步方案说明（2026-10-20）

用户询问第 3 步如何操作，任务卡已加入本机方案草案：两个独立 SIPp 进程经 UDP `127.0.0.1:15060` → `127.0.0.1:15061` 通信，在 `lo0` 仅按这两个端口捕获；一次正常建立与结束。附加控制口拟用 `127.0.0.1:18888/18889`，后续执行须显式限制 SIP 与控制 socket 的本机绑定，已对照 SIPp v3.8.0 官方源码核对参数。本次不启用媒体播放或 RTP echo。抓包建议上限为 60 秒/10 MiB，原始产物建议保留 7 天，仍待场景授权与具体确认，不自动删除产物。

本轮仅只读核对：`ifconfig lo0` 退出 0，确认 UP/LOOPBACK 与 `127.0.0.1`；`lsof -nP -iUDP:15060 -iUDP:15061` 退出 1 且无输出/错误，对 UDP 18888/18889 的查询也相同，表示当前用户可见范围内没有匹配 socket，启动前须复查。`command -v dumpcap` 退出 1（不在 PATH），随后确认 app 内 `tshark`、`dumpcap` 文件存在，未改环境或权限。未运行发包、抓包、音频或协议测试，progress.json 保持 not_started。

方案补录后，`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0；27 章/125 项结构与引用有效，30 份生成文件一致，两份工作文档的 20 个本地文件链接及空白检查通过。未执行 Rust/Python 测试用例或完整基础检查。

## AC-00-03 回环接口显示核对（2026-10-20）

用户报告 Wireshark 中未看到 `lo0`。只读执行 app 内 `dumpcap -D` 退出 0，明确列出第 17 项 `lo0 (Loopback)`；`ifconfig lo0` 退出 0。由此仅确认系统接口与 helper 枚举正常，尚未确认 GUI 显示原因或实际抓包权限。只读读取 Wireshark 窗口的工具报 ScreenCaptureKit `-3811`，未建立窗口绑定或取得当前界面；没有调整权限、安装组件或开始抓包。下一步核对接口名称、类型筛选与隐藏项，progress.json 保持 not_started。

已按 Wireshark v4.6.9 官方文档核对 Manage Interfaces → Local Interfaces 的 Show 控制接口显隐，具体用户操作路径见任务卡；没有远程修改配置。补录检查 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项、30 份生成文件），两份工作文档的 20 个本地链接与空白检查通过；未执行测试用例或实际捕获。

## AC-00-03 BPF 权限错误（2026-10-20）

用户确认 `lo0` 已能找到，随后反馈填写 filter 后报错；询问原文后取得的是设备 `ap1` 的 `/dev/bpf0: Permission denied`。该原文指向打开抓包设备时的权限拒绝，没有过滤器语法错误的证据；报错接口也不是拟用的 `lo0`。本轮只查看 dumpcap 帮助与本机权限，未执行过滤器编译或捕获。

只读核对显示 `/dev/bpf0` 为 root:wheel、`crw-------`，当前用户无读写访问且不在 `access_bpf` 组；标准 ChmodBPF LaunchDaemon 和支持目录未发现。已定位 app 自带的官方 `Install ChmodBPF.pkg`，`pkgutil --check-signature` 退出 0，签名主体 Wireshark Foundation、Apple 公证可信。官方 macOS 安装说明要求实际抓包使用 ChmodBPF；安装会改变机器 BPF 权限，需用户决定后手动操作。本轮没有打开安装器、修改权限、发包或抓包。

后续用户截图显示已选中 `Loopback: lo0`，欢迎页明确提示缺少抓包权限，底部为 `Unable to check capture filter`；当前不应归因为过滤器语法错误。已告知安装包在本机，可由用户点击官方 `installing ChmodBPF` 链接打开并手动确认安装，Codex 未执行安装。该轮文档检查 `validate --git`、`render --check`、`git diff --check` 均退出 0，27 章/125 项、30 份生成文件一致；两份文档的 20 个本地链接与空白检查通过，未执行测试用例或捕获。

## AC-00-03 ChmodBPF 只读复核（2026-10-20）

用户处理后反馈“现在看起来可以了”。只读复核确认标准 ChmodBPF LaunchDaemon 与支持目录已存在；`/dev/bpf0` 为 root:access_bpf、`crw-rw----`，当前工具会话读写访问检查为 True。`launchctl print system/org.wireshark.ChmodBPF` 退出 0、last exit code 为 0，权限脚本执行完成后 exited/not running 属正常结束。权限准备已有实证，实际 `lo0` 捕获尚未运行；Codex 未安装、修改权限、发包或抓包，未将本次反馈登记为场景授权或 AC 人工验收。

补录后 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项结构与引用有效、30 份生成文件一致），两份工作文档的 20 个本地链接与空白检查通过；没有运行 Rust/Python 测试用例或实际捕获。

## AC-00-03 第 4 步手动教学（2026-10-20）

用户表示未用过 SIPp，请求讲解如何完成第 4 步。任务卡已加入用户手动指南：新建独立原始记录目录，终端 A 启动内置 uas，Wireshark 限定 lo0/端口与自动停止，终端 B 启动一次内置 uac，两端自动退出后核对实际统计并保存 pcapng 与消息日志。SIP/控制 socket 都显式绑定回环，单次/单并发，主叫 20 秒超时、被叫等待上限 300 秒；图形抓包按 60 seconds/10 megabytes 设置（约 9.54 MiB，低于计划 10 MiB，停止阈值可能多最后一包）。仅准备讲解，不代用户执行场景或捕获，不将教学请求或期望结果记为通过。

本轮仅执行 `sipp -h` 选项核对（退出 99，为帮助命令正常无呼叫退出）及只读端口查询：四个候选 UDP 端口无匹配 socket，lsof 退出 1 且无输出/错误。命令、内置场景与 GUI 参数已对照官方版本资料。首次手动试跑记录不等于固定提交的正式验收；当前指导文档未提交，正式验收仍需固定场景/命令/版本并干净复现。progress.json 保持 not_started，未发包、抓包、录音、提交或推送。

教学补录检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项结构与引用有效、30 份生成文件一致。两份工作文档的 20 个本地链接与空白检查通过；指南的 3 个 bash 代码块均用 `bash -n` 静态解析成功，没有执行块内命令。这些不是 SIPp 场景或实际捕获结果；Rust/Python 测试用例未运行。

## AC-00-03 第 4 步实际结果核对（2026-10-20）

用户完成手动操作并提供 `/Users/ray/Downloads/sip-loopback.pcapng`，反馈“都成功了”。只读 capinfos/TShark 核对确认文件 3140 字节、唯一接口 lo0、原计划过滤器与两个本机端口，6 个 SIP 包属于同一 Call-ID，顺序 INVITE/180/200/ACK/BYE/200。两端退出码文件均为 0，6 条实际 UDP 载荷均与两端消息日志逐字节对应，必要头及 Content-Length 检查通过。本次独立手动信令与抓包结果已确认，不能替代后续音频检查或整个 AC 的人工验收。

保留 Downloads 原文件，在忽略的 `artifacts/ch00/ac00-03/manual-ZTgdQjAj/` 归档同字节副本、真实 INVITE 样例、capinfos 输出与 `verification.json`（14/14 离线核对通过）。原始抓包 SHA-256 为 `9d9473eb780f74b0d7ae57af11cd6632c4c08249bc496de2c3d3a52459115ec2`。未提供最终 Statistics 屏幕；首末包跨度约 1.002462 秒不证明整个捕获窗口或自动停止设置，限制已在摘要与任务卡中说明。

首次临时摘要脚本因时间字段为 ISO 字符串而退出 1，修正离线解析后退出 0，原始抓包未改。只将 AC-00-03 状态从 not_started 更新为 in_progress，待审阅提交、报告与人工审阅字段保持空值，随后用 render 更新生成视图。当前为用户手动试跑证据，尚未固定配置提交或干净复现；未修改核心代码、重跑网络、提交或推送。

登记后 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项结构与引用有效，30 份生成文件一致；实际仅 PROGRESS 生成视图变化。另核对只有 AC-00-03 的源状态与 note 改变，当前 2 项 passed、1 项 in_progress、122 项 not_started；两份工作文档 20 个本地链接和空白检查通过，原始抓包/样例/摘要均被 Git 忽略。本轮没有运行 Rust/Python 回归、release 构建、新 CI 或重跑网络场景。

## AC-00-03 第 5 步音频检查说明（2026-10-20）

用户请求第 5 步操作，并确认 Linphone 已进入联系人/通话主界面。只读 Info.plist 核实版本为 6.2.3，系统音频清单识别默认输入 MacBook Pro Microphone、默认输出 MacBook Pro Speakers；只证明设备被系统识别，未采集或试听。安装包 qml 目录两次文本查找未匹配、退出 1，后改读官方 6.2.3 源码，未读取用户账号配置。

任务卡已加入准确界面路径及手动检查：Application options → Settings → Calls → Devices，选择麦克风/输出设备，用户手动授权麦克风并说话观察 Linphone 实时电平；该版无独立测试音/录制回放按钮，输出另用 macOS 提醒音确认。离开设备页停止本地采集。该操作不需公网号码或保存录音；系统提示音不能证明 Linphone 通话内播放或端到端互通。当前尚未收到实际输入/听音结果，progress.json 保持 in_progress，Codex 未操作 GUI、启用麦克风、录音或呼叫。

首次文档补丁因上下文不匹配被拒绝且未写入，纠正后成功。补录后的 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项、30 份生成文件），两份文档 20 个本地链接及空白检查通过。本轮未运行 Rust/Python 测试用例、网络场景或音频实验。

## AC-00-03 第 5 步用户确认（2026-10-20）

用户在收到手动设备检查步骤后回复“都没有问题正常工作”并提供截图。截图直接确认 Linphone Devices 页选择 Default Playback/Default Capture，并有绿色麦克风电平；动态电平随说话变化及系统提醒音听见依据用户手动确认，不冒称静态截图或 Codex 独立听音已证明。只读系统音频清单再次确认复核时默认输入 MacBook Pro Microphone、默认输出 MacBook Pro Speakers，命令退出 0；截图本身只显示默认设备别名。

用户确认与截图文字观察保存在忽略的 `artifacts/ch00/ac00-03/manual-ZTgdQjAj/audio-device-check.json`，SHA-256 为 `8214b01767b046f82f57d937ffd8e35e1c7d57b13dfb3eb2b2ebb28139d160f9`；未保存原始截图、摄像头硬件 ID 或录音，未操作麦克风/摄像头或通话。更新任务卡、progress note 与 LAB-001 的过时描述，in_progress 和空的人工审阅/提交/报告字段保持不变，未将本地设备正常视为通话音频互通或整体 AC 通过。

补录后 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项结构与引用有效，30 份生成文件一致），三份工作文档 22 个本地链接与空白检查通过，音频检查文字记录被 Git 忽略。本轮未运行 Rust/Python 回归、网络场景或实际音频采集/播放。

## AC-00-03 第 6 步实际完成（2026-10-20）

固定配置本地提交 C 为 `0d972857cc02e51d21d9c02c48dfd6dc40d4d535`，仅包含本 AC 文档/进度，原始产物未入 Git。以 `git clone --no-local --no-checkout` 创建独立 clone 并 detach 检出 C，前后均 clean。doctor 与完整 check 均退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 测试全部通过，格式/Clippy/release 构建通过。原始日志位于忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/fresh-checkout-bwna4wgq/`。

从 C 任务卡提取既有参数，执行一次同端口的本机 SIPp/lo0 复验：15:33:39.572569–15:33:45.867780 +08:00，编排单调耗时 6.295164 秒。UAC/UAS 各累计成功 1、失败 0，dumpcap 及两端均退出 0，所有子进程结束，四个 UDP 端口可同时重新绑定。抓包 3140 字节、6 个 SIP 包，INVITE/180/200/ACK/BYE/200；双端 12 条发送/接收记录与六个载荷逐字节对应，真实 INVITE 已提取。图形界面、设备反馈沿用用户操作，不冒称本次重测音频。

独立离线核对最终退出 0、19/19 项本样例断言通过；首次离线脚本曾因错误要求 SIPp stderr 为空退出 1，实际为正常 localhost 解析提示，保留失败脚本/摘要后修正，只重跑离线核对，未重跑实验或改动原始证据。完整报告区分用户首次实验、本次干净复验及用户设备确认，并保留历史失败与未运行项。

本次抓包 SHA-256：`355bd9d8b080e437d4e5a97a833c36133b17c03851a8d99be4f9176e43115e74`；真实 INVITE 为 `cb84303c03cb47b86ca69b3139dd77b26107701cd9b96cf27020c3eb6f94d05f`。人工审阅字段保持空，仅登记 ready_for_review；当前 125 项中 2 passed、1 ready_for_review、122 not_started。Rust/测试/工程脚本/依赖与目标要求未改，其他 AC 与 ADR 不变；未推送、创建 PR、合并或开始后续核心任务。

登记材料检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章/125 项与真实提交/报告引用有效，30 个生成视图一致，21 个 Python 回归通过。本轮仅 PROGRESS 生成视图改变。报告关键原始哈希逐一匹配，73 个原始证据文件均被 Git 忽略，新 clone 仍 clean；完整清单在复验目录 manifest.json。两位代理分别只读审查验收覆盖与离线样例，未发现待修正的协议实验问题。登记文档阶段未重复 Rust/release、信令、音频或新 CI。

最终一次性文档核对曾退出 1：报告的命令路径占位符未加引号，无法通过 bash 语法检查；已补正引用方式，再核对语法、文件链接与原始哈希。该检查只解析文档，不执行占位命令或重跑实验。

## AC-00-03 人工验收（2026-10-20）

用户在收到本地材料提交 `0535b7aaa36e7c25237258f1a7fe5ed68e72467d`、完整报告、待人工验收结果及明确整项确认说明后回复“没问题”。据此将 AC-00-03 登记 passed，审阅者为“用户（本会话明确确认）”，日期为 2026-10-20；被测配置保持 `0d972857cc02e51d21d9c02c48dfd6dc40d4d535`，原始证据、哈希和未测媒体范围保留。LAB-001 的本项实验室准备缺口记为 resolved，其他问题、AC 与 ADR 状态不变。

当前 125 项中 3 passed、122 not_started。此次仅补录人工验收记录，尚未提交或推送，不创建 PR、合并或自动开始下一 AC；未重跑网络/抓包、音频、Rust/release 或新远端 CI。

人工验收登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章/125 项及真实提交/报告引用有效，30 个生成视图一致，21 个 Python 回归全部通过。生成视图仅 PROGRESS 改变；原始配置、证据与其他 AC 未改，不冒称本轮重跑完整工程或协议实验。

## AC-00-03 人工验收登记提交（2026-10-20）

用户回复“没啥问题，就提交吧”，授权本地提交六份人工验收登记。仅更新验收报告、progress.json、生成的 PROGRESS、CURRENT、任务卡及 LAB-001 记录；原始产物不入 Git，被测 C 与全部证据引用不变。本次不推送、创建 PR、合并或开始下一 AC，历史提交与分支保留。

提交前 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项及提交/报告引用有效，30 个生成视图一致，无空白错误。此次只提交文档，未重复 Python/Rust 测试、release 构建、网络/音频实验或远端 CI；沿用上轮 21 个 Python 回归及固定 C 的完整复验结果。

## AC-00-03 推送与创建 PR（2026-10-20）

用户在“推送当前分支并创建 PR”的下一步说明后回复“帮我完成1”，授权推送与创建 PR。开始时工作区干净，实际远端 main 为准备基线 `0bb5d017a791e2834a12efe85b7ed3d33890799f`，远端尚无该分支或对应 PR。已推送人工验收提交 `62be8689926ed2f689d8213d433be51a9916e385` 并创建 [PR #18](https://github.com/Ray-56/yakyak/pull/18)，目标 main；本轮不合并、删除分支或开始下一 AC。

交付前在上述 head 执行 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 测试及格式/Clippy/release 构建全部通过；`python3 scripts/roadmap.py validate --git` 与 `git diff --check main...HEAD` 也退出 0。未重跑信令、音频或变更被测 C。范围仅六份脱敏文档/状态文件，原始产物与 LICENSE 未进入变更。

PR 创建后补录本节、任务卡、报告与 progress note，并单独提交推送；最终 head 的 push/PR 双平台 CI 由 GitHub 触发，实际状态以 PR Checks 为准。本轮不将历史 C 的检查外推为新 head 的远端结果。首次 `gh pr view --repo ...` 遗漏显式 PR 编号，查询退出 1；加上 18 后查询成功，没有影响已创建 PR。

原始本地检查、PR 描述及交付核对记录在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/pr-delivery-t4tt2qih/`。

交付补录的 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项与真实引用有效，30 个生成视图一致。该补录仅改四份文档/进度 note，没有改状态或生成视图；未重复本地完整检查或协议实验，最终 head 的远端检查单独核对。

## AC-00-03 用户合并后的核对（2026-10-20）

用户告知“the pr is merged”。GitHub 实际显示 PR #18 于 15:56:16（Asia/Shanghai）合入，PR head 为 `62be8689926ed2f689d8213d433be51a9916e385`，merge commit 为 `b32cad4cb4b6352d8f6b8bf884eb9bb124da6862`。验收材料与人工 passed 登记均已进入 main；PR head 的 push/PR 四项平台检查均成功。

交付记录补录 `f5f17c576a59713d181089b61b7a4ad56040444b` 在用户合并后才推送，因此未包含于该 PR。原分支和补录提交保留，没有强推、重置、删除或冒称其已合入。原交付核对脚本曾断言 PR 仍 OPEN 且 head 等于本地最新 SHA，因并发合并而退出 1；随后按实际 MERGED 状态与真实合并 head 重新核对，不属于协议或 CI 失败。

[main CI 39389863121](https://github.com/Ray-56/yakyak/actions/runs/39389863121) 在上述 merge commit 的 Ubuntu 24.04/macOS 15 两平台均 success：各 21 个 Python、2 个 Rust 单元、6 个 CLI 测试及格式/Clippy/release 构建通过，0 失败/忽略/过滤。原始 job 日志、run/jobs/summary 与完整性清单在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/merge-closeout-stxqchh7/`。

已 fetch 并以 `git switch main`、`git merge --ff-only origin/main` 同步本地 main，核对前后 clean、本地与实际远端 main 相同，C、材料提交 `0535b7a…` 和人工验收提交 `62be868…` 均保持祖先可达；f5f17c5 不属于 main 祖先。同步原始记录在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/merge-sync-fjb781gx/`。

随后建立独立本地 `docs/ch00-ac00-03-closeout`，将未进入 main 的四份交付补录与本次合并事实一起整理，保留真实时序。该轮当时尚未提交、推送或新建 PR，也未合并或开始下一 AC；当时未重跑本地完整检查、信令或音频，主线测试结果依据上述新 CI 原始日志。

收尾文档检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项及提交/报告引用有效，30 个生成视图一致，无空白错误。仅四份文档/进度 note 改变；passed、被测 C、原证据与其他 AC/ADR 不变，未重复本地测试或媒体实验。

## AC-00-03 独立收尾交付（2026-10-20）

用户请求将这四份收尾记录“提交 推送 pr”，并询问完成后能否准备 0004，授权当前独立分支的提交/推送/PR 交付。只处理报告、progress note、CURRENT 与 AC-00-03 任务卡，不改变目标、passed、被测 C、人工审阅信息或原始证据，不混入 AC-00-04 任务卡、ADR 或核心实现。

本轮先完成全部文档再一次提交、推送并创建收尾 PR；最终提交、真实 PR 链接和 CI 状态保存在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/closeout-delivery-0d_x7ou0/`，避免为补 PR 编号继续向已可合并的分支追加提交。最终状态也可从 GitHub 的同名分支 PR/Checks 核对；本轮不执行合并。

本轮提交前 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 测试及格式/Clippy/release 构建通过，27 章/125 项结构有效、30 个生成视图一致；本地链接和四文件变更范围已核对。`python3 scripts/roadmap.py validate --git` 与差异空白检查另行通过。原始日志保存于上述交付目录；未重跑协议、抓包或音频，本项状态和 AC-00-04 的 not_started 不变。

## 实现授权约定补充（2026-10-20）

按用户要求在根目录 AGENTS.md 明确：只有用户明确要求开始实现具体任务时才进入实现阶段，讨论、评审或文档准备不构成实现授权。此次仅修改协作约定及工作记录，AC 状态不变；下一步等待用户明确安排。`git diff --check` 退出 0；未运行测试（0 项），未运行构建或协议实验，无新增运行时风险。

## AC-00-04 设计确认与文档落地（2026-10-20）

用户确认的是完整设计草稿：状态机与执行器分工、消息/定时器/结果事件、两种时间与虚拟时钟、旧定时器及迟到结果失效、无流量停止与 join、共享资源与协议保留状态，以及八类后续测试设计。内容保存于 [ADR-0002](../architecture/adr/0002-protocol-runtime-time.md)，用户确认记录随 ADR 保存，架构目录同步标注 Accepted；ADR-0001 和 ADR-0003 不变。

本轮建立单一 [AC-00-04 任务卡](tasks/AC-00-04.md)，只将 progress.json 中本项改为 in_progress，并由 roadmap.py 生成进度视图。implementation_commit、evidence、reviewer、reviewed_at 保持空值，不以准备基线冒充包含本设计的提交，不把聊天确认扩写为整项 AC 已验收。未修改 targets.json、源码、依赖或测试实现。

本轮在未提交文档上执行 `python3 scripts/roadmap.py render`、`validate --git`、`render --check`、`git diff --check`，均退出 0：27 章/125 项结构与现有引用有效，30 个生成视图一致；只有 PROGRESS 的本项状态与汇总发生变化。`python3 -m unittest discover -s scripts -p 'test_*.py' -v` 退出 0，21 项文档工具测试通过、0 失败。收尾补录后复查前三类只读检查，日志加 `final-` 前缀。原始结果位于独立工作树忽略的 `artifacts/ch00/ac00-04/20261020-design-nf_i1ijb/`，任务卡记录完整命令及首轮补丁格式失败的修正。

本轮没有运行 Rust/CLI 测试、格式/Clippy/release 构建、远端 CI 或网络/音频实验；ADR 的八类协议场景仍为测试设计。首次在独立工作树整理时，原工作区三份未提交改动保持原样；本轮将分支移回原目录后，AGENTS.md 与 AC-00-03 任务卡仍逐字节不变，CURRENT 的用户补充已完整合并。其中新的实现授权约定同样遵守。本轮只处理确认后的设计文档，不将其视为核心实现授权。

## AC-00-04 分支回到原项目目录（2026-10-20）

用户要求“分支没有操作，操作一下”，本轮先备份原工作区三份改动、独立工作树六份设计文档及双方差异，再让原独立工作树保持 detached HEAD，释放已有分支，最后在原项目目录执行 `git switch docs/ch00-ac00-04`。两处 HEAD 均保持准备基线，未产生提交，也未删除分支、工作树或历史。

六份设计文档及原始检查产物已转回当前目录；CURRENT 合并用户的“实现授权约定补充”原文，AGENTS.md 与 AC-00-03 任务卡内容未改。原始备份与本轮检查保存到忽略目录 `artifacts/ch00/ac00-04/20261020-branch-transfer-x2klwzwc/`。设计内容、Accepted 与 in_progress 不变，不把分支切换记成验收通过。

本轮 `python3 scripts/roadmap.py render`、`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项结构与现有引用有效，30 个生成视图一致，无空白错误。未重复 Python 回归（沿用前轮 21 项通过的记录），未运行 Rust/CLI 测试、构建或网络/音频实验。本轮检查不构成固定提交上的正式验收。

下一步固定本地设计提交、在干净提交上复核并准备验收报告；用户后续审阅整项报告。此前对设计的确认已经记录，不要求用户重复确认同一设计；本轮仅操作分支和文档，没有自行提交或推进验收状态。

## AC-00-04 本地固定与干净复核授权（2026-10-20）

用户在本地设计提交、干净复核与验收报告的说明后回复“好的”。本轮按任务卡先提交六份设计/工作文档形成 C，再以新 clone 检出完整 C，运行文档校验、生成一致性、21 项文档工具回归及差异检查；实际 SHA 和检查结果在执行后由验收报告及后续提交 D 登记，不预写通过或 self-reference。

只暂存本项改动；用户既有 AGENTS.md、AC-00-03 任务卡与 CURRENT 的“实现授权约定补充”留在原工作区，不纳入本 AC 提交。CURRENT 的任务记录按内容选择暂存，用户补充与备份保留。不实现通信内核，不运行网络、抓包、音频或远端 CI，不推送、创建 PR 或合并。复核材料齐全后最多 ready_for_review，等待用户审阅整项报告。

## AC-00-04 固定提交复核与待审阅登记（2026-10-20）

已创建本地设计提交 C `aab3180fb645ed46972d683e7c1654d46e5b7e47`，只含本项六份文档；用户三份既有改动继续保留，未纳入 C。使用 `git clone --no-local --no-checkout` 新建独立副本，再明确检出完整 C，检查前后 status 均空。

C 上 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check`、`git show --format= --check <C>` 均退出 0：27 章/125 项有效、30 个生成视图一致，21 项 Python 测试通过、0 失败。独立代理只读设计评审六项关注点未发现原 AC 的实质遗漏；不是人工整项验收。

首轮证据汇总数量正则过度转义，汇总退出 1；修正后仅重跑汇总退出 0，五项检查和原始输出不变。原始环境、命令、前后状态、技术评审、错误回执和哈希位于忽略的 `artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/design-review-i3dxcz0h/`，详细结果见报告。没有运行 Rust/CLI 测试、构建、远端 CI、网络/抓包/音频或八类协议测试场景。

验收报告、进度、生成视图、任务卡和 CURRENT 在后续本地 D 单独登记，不改变 C 或 ADR 内容。登记检查日志放原始证据目录的 `registration/`；implementation_commit 指向 C、evidence 指向报告、状态为 ready_for_review，reviewer/reviewed_at 保持空值。登记前原工作区的 `validate --git`、`render --check`、21 项 Python 回归和 `git diff --check` 另行全部退出 0；登记这一轮与 C 的干净结果区分，不冒称原工作区无用户改动或 D 已运行完整工程检查。补齐说明后复查文档/差异检查再提交 D。用户原有 AGENTS.md、AC-00-03 任务卡与 CURRENT 授权补充不纳入 D，本轮不 push/PR/merge 或开始 AC-00-05。

## AC-00-04 整项人工验收确认（2026-10-20）

用户收到固定 C、报告提交 D、完整报告链接、复核结果及明确整项确认提示后回复“通过”。本次据此登记 passed，reviewer 为“用户（本会话明确确认）”，reviewed_at 为 2026-10-20。此前只确认 ADR 内容的记录与本次整项确认分开保留。

仅修改报告、progress.json、生成的 PROGRESS、任务卡与 CURRENT，保存本地人工验收登记；C、原始证据、历史失败记录和未测限制不变。用户既有 AGENTS.md、AC-00-03 任务卡与 CURRENT 补充继续保留，不混入登记提交。`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项及引用有效、30 个生成视图一致。本轮测试执行 0 项，没有重复 Python/Rust/CLI、构建、CI 或协议/音频实验；首轮补丁上下文失败与修正见任务卡。登记检查、授权回执与备份位于忽略的 `artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/human-acceptance-gd2xhlyc/`。不 push/PR/merge 或开始 AC-00-05。

## AC-00-04 推送与创建 PR 交付（2026-10-20）

用户在“推送当前分支并创建 PR”的说明后回复“可以”，授权本轮远端交付。开始时实际远端 main 为准备基线 `8861c04fbd6dfd00805feb18f73bf0e1208ed180`，远端尚无该分支或对应 PR；本地已有设计、证据和人工登记三个提交。

交付前新建本地 clone 检出人工登记提交 `41728e2ea1798a61c47739c0753586e03b3cebd8`，前后 clean；`bash scripts/doctor.sh`、`bash scripts/check.sh` 均退出 0：21 项 Python、2 项 Rust 单元、6 项 CLI 及 fmt/Clippy/release 构建通过，27 章/125 项与 30 个生成视图有效。该轮是后续交付预检，不外推为 C 的新增功能结果，未运行协议、网络、抓包或音频场景。

本轮先写完 CURRENT 与任务卡两份交付记录，单独提交后一次推送、创建 PR，再核对最终 head 的 CI。最终 SHA、PR 链接、CI 运行、原始日志与完整性回执在忽略目录 `artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/pr-delivery-yk_oabgr/`；不为回填 PR 编号向已交付分支反复追加提交。CI 实际状态以该 head 的 Checks 和回执为准，不预写成功。

只追加交付记录，不改 passed、C、人工审阅字段、原始证据、其他 AC/ADR 或目标。用户既有 AGENTS.md、AC-00-03 任务卡和 CURRENT 授权补充仍不纳入提交；不推送这些工作区改动，不 merge 或开始 AC-00-05。

## AC-00-04 合并与工作区收尾（2026-10-20）

用户在合并 PR #20、另交付三份授权约定改动、备份并清理旧工作树的说明后回复“帮我操作一下”。PR #20 已普通 merge，合并提交为 `dd72b3fd9bc4160bb347ef88fc6f81dfdcfcc796`（2026-10-20 18:29:00，Asia/Shanghai）；本地 main 已 fast-forward 同步。其 [main CI 39408251000](https://github.com/Ray-56/yakyak/actions/runs/39408251000) 的 Ubuntu 24.04/macOS 15 均成功：各 21 项 Python、2 项 Rust 单元、6 项 CLI 测试，以及格式、Clippy、release 构建通过。这是合并提交的基础工程检查，不改写设计 C 的原始复核范围。

旧 `../yakyak-ac00-04` 是早期设计的 detached 工作树，不是另一套待实现代码。其 123 个文件（633697 字节，含未提交文档、原始产物和缓存）已完整归档并逐文件核对内容、权限；独立代理只读复核确认无遗漏。删除前再次核对一致后，用 `git worktree remove --force` 移除该工作树。没有删除历史分支、tag、主仓库 Git 数据或 `../yakyak-bootstrap-20261016-160959`。

收尾回执与备份目录：`artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/workspace-closeout-2ksiub4p/`。完整旧工作树在其中 `archived-ac00-04-worktree/`；`old-worktree-manifest.json` 记录哈希与权限，`archive-restore.txt` 说明恢复方法。归档中的 `.git` 是旧工作树指针，恢复时应创建新工作树再复制内容，不能直接视为独立仓库。原始证据仍受 Git 忽略，保留且不自动删除。

此前留在工作区的 AGENTS.md、CURRENT 授权补充和 AC-00-03 任务卡原文，已单独固定为上述授权约定提交；本轮只再更新 CURRENT 与 AC-00-04 任务卡。先完成本次记录，再提交、推送并创建独立 PR；其最终 head 的 push/PR 双平台检查、普通合并、主线 CI 与同步结果只登记于上述回执目录，避免为回填编号反复追加提交。AC-00-04 的 passed、C、人工审阅与原始证据，以及其他 AC/ADR/目标均不变。

本轮本地 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项与提交引用有效，30 个生成视图一致，无空白错误。只核对文档，本地测试执行 0 项，没有重跑 Rust/CLI、构建或协议/音频实验；新 head 和合并后 main 的完整基础检查由实际 CI 单独核对。

## 后续安排

执行前冻结记录（2026-10-20，历史）：用户回复“6 应该你可以帮我处理了”。当时按任务卡冻结现有 UAC/UAS 命令、工具版本与内置 XML 哈希，计划新 clone 运行 doctor/完整基础检查及一次相同端口的有界 lo0 抓包实验，最长 60 秒、低于 10 MiB，原始产物保留 7 天且不自动删除。仅操作本轮子进程，不安装软件或放宽权限，不开麦或录音；此前设备用户确认单独引用。当时尚未复验，状态仍 in_progress，先创建本地固定提交 C 再补录结果；现已完成，实际结果见上节，不 push/PR/merge。

1. 第 6 步已完成本地固定提交、干净复验与完整报告。原始产物保留 7 天，至 2026-10-27 由用户决定，不自动删除。
2. AC-00-04 的固定 C、干净复核、报告、人工验收及 PR #20 合入已完成。本轮独立交付授权约定与收尾记录，核对 CI、合入并同步 main；最终结果见收尾回执。
3. ADR-0002 保持 Accepted；ADR-0001、ADR-0003 仍 Proposed，另行审阅。本次确认只覆盖 AC-00-04，不自动开始 AC-00-05。
4. 后续 AC 需用户明确安排，不因本项 passed 自动开始下一项；逐项完成第 00 章后再安排 AC-01-01。

当前 AC-00-04 为 passed，设计评审与人工验收已完成，通信内核未实现。AC-00-03 保持 passed，原始信令与设备确认沿用原报告，通话内媒体仍未测。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告“后续授权：直接同步 main”一节。
