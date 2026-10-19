# AC-00-01 技术审阅记录

日期：2026-10-19（Asia/Shanghai）。执行者：Codex。人工审阅者：用户（本会话明确确认）；确认日期：2026-10-19。

证据提交 A：`41857037d1fda8b7b406534f84aca843db8f68af`。
被审阅提交 B：`2cea4ed0e4a65ca85953e13d5367ee0c06897e7a`。
修复与复验提交 C：`b24092bea144d04a42307a76a9b016a3da5c9447`。
比较基点：`cbe33dece07b21b259e393061c4c6d6e19d0e165`。

审阅命令：`git diff cbe33dece07b21b259e393061c4c6d6e19d0e165...2cea4ed0e4a65ca85953e13d5367ee0c06897e7a`；共两次提交、10 份文档。完整检查在 B 的干净工作区运行，本文件及后续问题登记不在被审阅快照中。

技术审阅结论：Standards 初审发现的 1 项 P2 已在 C 修复并通过完整复验；Spec 未发现缺项、范围扩张或错误结论，当前没有未解决的技术审阅问题。用户随后明确确认 AC-00-01 验收通过，当前状态为 `passed`。下文保留 B 上的失败与定位过程。

## Standards

依据：AGENTS.md、CONTRIBUTING.md、docs/development/WORKFLOW.md、docs/roadmap/TESTING_CONTRACT.md、docs/roadmap/README.md、docs/templates/ACCEPTANCE.md。

**[P2，已修复] 测试临时数据未与真实进度隔离，导致完整检查失败。**

`docs/roadmap/progress.json:6` 将 AC-00-01 登记为 `ready_for_review` 后，`scripts/test_roadmap.py:24–29` 将真实进度复制到临时磁盘，却只在 `self.progress` 内存数据中归一化；`scripts/test_roadmap.py:152` 的 CLI 测试重新读取磁盘时，因临时目录没有进度引用的验收报告而先返回 1，未知 ID 分支的预期返回码为 2。因此 `bash scripts/check.sh` 退出 1，未满足 CONTRIBUTING.md“合入前 check.sh 通过”的要求。

临时对照复现：

| 输入环境 | `show AC-99-99` 返回码 | 实际原因 |
|---|---:|---|
| 真实仓库（数据有效） | 2 | unknown objective |
| 测试原有临时目录 | 1 | 引用的 AC-00-01 报告不存在 |
| 仅在临时目录将已归一化的进度写回磁盘 | 2 | unknown objective |

初审建议修复测试临时数据的持久化，使 CLI 从有效、独立的输入进入未知 ID 分支，保留返回码 2 的断言与所有报告验证标准。初审仅在临时目录做诊断；随后按用户要求完成最小修复，见下文。

其余文档、状态、授权与归档约定未发现问题；未发现值得单列的文档结构问题。原始日志处于忽略目录中，Git 没有跟踪抓包、录音或凭据。

## Spec

依据：`docs/roadmap/targets.json` 的 AC-00-01。逐项结论：

| 要求 | 核对结果 |
|---|---|
| 旧实现保留且能独立检出 | 两个 tag 在新建独立本地 clone 中检出成功，工作区均 clean；四个历史文件与对象内容及既有哈希一致 |
| 路线图版本记录 | SCOPE v1.0 已记录，功能矩阵、排除项及自研边界未变更 |
| 参考 commit 真实 | 重新从官方仓库取得 `0b1414bb3460a42abce8596f8f07d9b6d2889975`，对象类型为 commit；四个文件的字节数、SHA-256、Git blob 均与 reference-lock 一致 |
| 工具链与测试端版本记录 | 工具链、SIPp 3.8.0、Linphone 6.2.3、Wireshark/TShark 4.6.9 已有实际输出；三份历史原始 JSON 哈希与报告一致，SIPp 的 99 未被伪写成 0 |
| 用户确认范围 | 2026-10-18 本会话明确确认三项边界，记录未扩大为整个 AC、ADR 或第 00 章通过 |

未发现缺项、范围扩张或错误的 AC 证据结论。网络、抓包、音频和互通属于后续实验，本项未将其假称为已验证。

## 本轮实际验证

新建历史 clone 与重新取得参考 commit 的 25 条 Git 命令均退出 0；旧代码与上游程序均未执行。`bash scripts/check.sh` 在干净 B 上的结果：

| 检查 | 数量 / 结果 | 状态 |
|---|---|---|
| roadmap validate | 27 章、125 目标 | 退出 0 |
| render --check | 30 个生成文件一致 | 退出 0 |
| Python unittest | 21 个执行，20 通过、1 失败；`test_unknown_show_returns_nonzero` | 失败，导致 check.sh 退出 1 |
| cargo fmt | 未执行，前序失败后脚本停止 | not_run |
| cargo clippy | 未执行，前序失败后脚本停止 | not_run |
| cargo test | 未执行，不能沿用历史 6 项结果当作本轮通过 | not_run |
| cargo build --release | 未执行，前序失败后脚本停止 | not_run |

本轮未运行 GUI、Linux/远端 CI、协议场景、抓包、音频或真实通话；没有安装软件、修改权限或推送远端。

审阅记录与问题状态写入后，`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 三项均退出 0（27 章/125 目标、30 个生成文件一致）；这三项文档检查不替代上文失败的完整检查。

## 证据与后续

原始产物保存在忽略的 `artifacts/ac00-01/20261019-review-99zim7gz/`：

| 文件 | SHA-256 |
|---|---|
| full-check.json | `3342966cfb921051e04351d309414a5b215f173df119e7d0ee9471f26f52f77f` |
| independent-evidence.json | `2c575fc2d741a71fb52e86418338a67ff4a34c5e1206e6009a7df8bb85ec8f30` |
| test-fixture-diagnosis.json | `38e46e5ff32d7bb58dd45fd9814932c8bf4ba4d7fb3387528ff2108e52051e34` |

初审时的后续安排是先修复测试临时数据并在真实可达提交上完整复验；当时 AC-00-01 回到 `in_progress`，证据提交仍为 A，人工审阅字段留空。此前 B 的文档检查与失败结果均保留为历史记录。当前修复及状态见下节。

## P2 修复与复验（2026-10-19）

用户询问该 P2 如何修复后，执行最小修复：在 `RoadmapTests.setUp()` 归一化 `self.progress` 后，增加三行将它序列化写回 `self.root` 下的临时 `docs/roadmap/progress.json`。三行代码只写测试临时目录，生产脚本、未知 ID 返回 2 的断言与报告验证标准均未改变；没有新增依赖、辅助抽象或重复测试。

先运行 `cd scripts` 后的 `python3 -m unittest test_roadmap.RoadmapTests.test_unknown_show_returns_nonzero -v`：修复前 1 个执行、1 个失败，退出 1（2 != 1）；修复后同一命令 1 个执行、1 个通过，退出 0。现有用例已覆盖本次真实回归，未另写镜像测试。

修复单独保存于 C `b24092bea144d04a42307a76a9b016a3da5c9447`。从本地仓库新建独立 clone 并检出 C，确认 HEAD 正确、工作区 clean，再运行 `bash scripts/check.sh`，整体退出 0：

| 检查 | 实际结果 |
|---|---|
| roadmap validate | 27 章、125 目标，退出 0 |
| render --check | 30 个生成文件一致，退出 0 |
| Python unittest | 21 个执行，全部通过，退出 0 |
| cargo fmt | 退出 0 |
| cargo clippy | 退出 0 |
| cargo test | 6 个 CLI 集成测试通过，0 忽略/过滤；main 单元 target 的 0 个测试未计入通过数量 |
| cargo build --release | 退出 0 |

复验日志：`artifacts/ac00-01/20261019-p2-fix-qn_an2w2/full-check.json`，SHA-256 `fe64e475356e5b38751963bb9cfa03e4d708554b25147df5b3831da843ae0149`。先前 B 上的失败日志原样保留。

DOC-002 解决后，AC-00-01 当时恢复 `ready_for_review`，`implementation_commit` 指向 C，人工审阅字段留空。该轮只验证 macOS 基础检查，没有执行 Linux/远端 CI、GUI、网络发包、抓包、音频或互通测试；没有推送远端。

## 人工验收登记（2026-10-19）

步骤三已向用户说明本次确认针对 AC-00-01 的证据与复验结果；用户在本会话明确回复“通过”。据此将 AC-00-01 记为 `passed`，审阅者为“用户（本会话明确确认）”，日期为 2026-10-19，复验提交仍为 C。此次确认不改变其他 AC、ADR 或整章状态，也不将前述未运行项目记为通过。

本轮仅更新验收元数据与文档；此前真实测试结果和原始日志保持不变。
