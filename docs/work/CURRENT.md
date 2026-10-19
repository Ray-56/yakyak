# 当前工作

阶段：第 00 章，AC-00-01 已人工验收通过，Git 交付由 [PR #14](https://github.com/Ray-56/yakyak/pull/14) 跟踪。
当前任务：[AC-00-01](tasks/AC-00-01.md)。状态：passed，以 progress.json 为准；PR 合入并同步主线后，等待用户安排下一项。

2026-10-18（Asia/Shanghai），用户在本会话明确确认 SCOPE v1.0 的功能范围、排除项及自研/复用边界。两个历史 tag 已分别在独立本地 clone 中检出并验证干净，参考 commit 沿用初始化时的真实对象与文件证据。用户授权的 Wireshark 4.0.7 → 4.6.9 更新已完成，旧版保留在仓库外私有备份中。

用户安装软件后，已实测 SIPp 3.8.0-TLS-PCAP、Linphone 6.2.3、Wireshark/TShark 4.6.9 的版本命令；两个 app 的签名和 macOS 信任检查均通过。SIPp `-v` 返回 99，为官方定义的正常无呼叫退出，已保留原始退出码。实际测试端版本缺项已补齐，见 [test-tools-lock.json](../references/test-tools-lock.json)及 [AC-00-01 记录](../acceptance/ch00/AC-00-01.md)。图形界面、协议场景、抓包与音频仍未验证；没有修改权限或系统 PATH。

2026-10-19，按用户授权创建证据提交 A：`41857037d1fda8b7b406534f84aca843db8f68af`，固定本项完整材料；后续提交 B 登记 A 的真实 SHA 与待审阅状态。提交 A 的干净工作区文档检查通过，后续登记的检查结果见验收记录；当时人工审阅字段为空。

随后按用户要求完成技术审阅：AC-00-01 材料齐全，两个旧版本重新独立检出，参考 commit 的四个文件重新取得且哈希一致。但在干净 B `2cea4ed0e4a65ca85953e13d5367ee0c06897e7a` 上运行 `bash scripts/check.sh` 退出 1：21 个 Python 测试中 20 过、1 失败，Rust 检查未执行。已定位为测试只在内存归一化进度、CLI 重读临时磁盘时缺报告的问题。状态回到进行中，详见 [技术审阅记录](../acceptance/ch00/AC-00-01-REVIEW.md)。

用户随后询问修复办法，已用 3 行代码将归一化进度写回测试临时文件，单独保存于 C `b24092bea144d04a42307a76a9b016a3da5c9447`。原失败项红转绿；C 的独立干净副本上完整检查通过：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建全部成功。DOC-002 已解决，状态恢复待审阅；生产校验与原断言未改变，原始失败证据保留。

2026-10-19，用户在步骤三人工验收说明后明确回复“通过”。已将 AC-00-01 记为 passed，审阅者为“用户（本会话明确确认）”，日期为 2026-10-19；复验提交保持 C，原始证据与历史失败记录保留。

初始化任务 [INIT-001](tasks/INIT-001.md) 保持 ready_for_review，INIT-11 部分完成。此前检查和本地提交见[初始化报告](../handoff/BOOTSTRAP-REPORT.md)；初始化交付时 Linux 与远端 CI 为 not_run。2026-10-19 已核对[后续 main 的 CI](https://github.com/Ray-56/yakyak/actions/runs/38875493382)：提交 `cbe33dece07b21b259e393061c4c6d6e19d0e165` 的 ubuntu-24.04 与 macos-15 两项均成功；本次分支的独立 CI 记录见下文。当前通过 1 项，其余 124 项保持 not_started。

## Git 收尾记录（2026-10-19）

初次核对时，本地 `docs/ch00-ac00-01-scope` 的验收登记提交为 `7d366eed0999c850ec2b864f853b2f2a7b19865b`；当时实际远端 `main` 和同名分支都仍为 `cbe33dece07b21b259e393061c4c6d6e19d0e165`，没有对应 PR，工作区干净、无 stash。上述本地验收登记提交已运行 `bash scripts/check.sh` 并退出 0：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建通过。

用户随后回复“ok”，明确授权推送、创建 PR、等待 Ubuntu/macOS CI 后普通 merge 合入并同步本地 main。已推送 `a5d5179e80a3148a4d549e27699ddf8df699b8c3` 并创建 [PR #14](https://github.com/Ray-56/yakyak/pull/14)；该提交的 [push CI](https://github.com/Ray-56/yakyak/actions/runs/39194109377) 与 [PR CI](https://github.com/Ray-56/yakyak/actions/runs/39194121363) 均在 ubuntu-24.04、macos-15 成功。该 SHA 的本地完整检查也已退出 0（21 个 Python、6 个 CLI 及格式/Clippy/release）。

本段补录后，合入前仍须等待最终 PR head 的全部 CI 成功；实际 head、检查和合入提交以 PR 页面为准。使用普通 merge 保留验收引用的 C `b24092bea144d04a42307a76a9b016a3da5c9447` 及原始证据提交，合入后核对主线 CI、本地与远端 main 一致和 C 可达；保留历史分支、tag 和 worktree。最终执行结果保存在忽略的 `artifacts/ac00-01/20261019-git-closeout-vzi79j6w/`。

## 后续安排

1. 按 PR #14 的交付流程完成合入、主线 CI 与本地同步；完成后等待用户指定下一项，不自动开始 AC-00-02。
2. 合入后为下一项单独安排分支和任务卡；按顺序可先核对 AC-00-02 的 Linux/CI 验收条件。已有工具版本记录可复用于 AC-00-03。
3. 另行审阅初始化变更、三个 ADR 草案与证据流程；本次范围确认没有将 ADR 改为 Accepted。
4. 补齐 AC-00-03 受控发包、抓包权限、音频设备和合法样例；逐项完成第 00 章验收后，再由用户明确安排 AC-01-01。

本轮按用户授权推送并创建 PR，补齐双平台 CI 证据与交付顺序。未推进其他 AC 或修改 ADR 状态。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告末节。
