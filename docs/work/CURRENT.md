# 当前工作

阶段：第 00 章；INIT-001、AC-00-01 与 AC-00-02 均已人工验收通过。
当前任务：[AC-00-03](tasks/AC-00-03.md) 进行中；第 4 步 SIPp/lo0 与第 5 步用户设备确认已完成。用户已授权 Codex 执行第 6 步：固定本地提交、全新 clone 基础检查与一次同边界 loopback 复验、整理完整报告；成功后最多 ready_for_review，不推送或合并。[INIT-001](tasks/INIT-001.md)、[AC-00-01](tasks/AC-00-01.md) 与 [AC-00-02](tasks/AC-00-02.md) 保持 passed，其他 AC 与 ADR 状态不变。AC-00-02 的独立收尾 [PR #17](https://github.com/Ray-56/yakyak/pull/17) 已合入。

当前分支：`test/ch00-ac00-03`，从 main 的 `0bb5d017a791e2834a12efe85b7ed3d33890799f` 建立。2026-10-20（Asia/Shanghai），初始仅授权本地分支/任务卡准备与讲解；第 6 步新增授权见下文，历史阶段的未执行记录保留。历史分支和原验收提交保留。

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

## 后续安排

第 6 步授权与固定输入（2026-10-20）：用户回复“6 应该你可以帮我处理了”。按任务卡冻结现有 UAC/UAS 命令、工具版本与内置 XML 哈希；新 clone 将运行 doctor/完整基础检查及一次相同端口的有界 lo0 抓包实验，最长 60 秒、低于 10 MiB，原始产物保留 7 天且不自动删除。只操作本轮子进程，不安装软件或放宽权限，不开麦或录音；此前设备用户确认单独引用。尚未执行复验，状态仍 in_progress；将先创建本地固定提交 C，再补录实际结果，不 push/PR/merge。

1. 第 4 步本次信令/抓包已核对，第 5 步本地设备检查获用户确认；进入第 6 步证据整理与验收收尾。原始产物拟保留 7 天，不自动删除。
2. 固定实际场景/命令/版本并干净复现，整理原始样例、对应抓包、本地设备的用户确认及完整验收材料；当前文档尚未提交、推送或创建 PR。
3. 另行审阅三个 ADR 草案与证据流程；本次初始化验收没有将 ADR 改为 Accepted。
4. 完成 AC-00-03 实验后整理独立样例、对应抓包及版本/地址/端口/拓扑证据，进入人工验收；逐项完成第 00 章验收后，再由用户明确安排 AC-01-01。

当前 AC-00-03 为 in_progress；本次用户手动 SIPp 信令与抓包已完成离线核对，本地音频设备获用户确认正常；通话内媒体未测，源码、其他 AC 与 ADR 状态不变。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告“后续授权：直接同步 main”一节。
