# INIT-001 初始化执行报告

日期：2026-10-16（Asia/Shanghai）。状态：**工程基础已交付，待用户审阅；INIT-11 部分完成**。

本次止于初始化：无通信核心、无第 01 章实现。以下 `verified` 表示已经观察并记录的技术结果，不是用户签署的功能验收。Linux 与远端 CI 未运行，不能宣布所有 INIT 项或第 00 章通过。

`tested_commit`：`25157c636873f839c133c923c5a500af66d8f4c9`。实现提交 C 的干净工作树、临时全新导出副本和负面试验恢复后的树均执行了完整检查；本报告在后续文档提交 D 中记录结果，D 不修改被测代码。

## 仓库、历史与原工作区

| 项目 | 实测值 |
|---|---|
| 原工作区 | 用户给定的 `yakyak/`；下文用 `$SOURCE` 代称 |
| 原分支 / 本次基点 | `main` / `0a1cb3d0e9c3718fdd97b872f090c9600d0ae487` |
| 开始时本地缓存 `origin/main` | `d50c3fcf82475ad2a69dd05d245115fa0d999df7` |
| `git ls-remote origin refs/heads/main` 现场结果 | `b4a03dcdad8557a2a922fb1b3eaef0e37f00fdbc`；只读查询，未 fetch / 合并 |
| 原目录状态（前后相同） | ` M README.md`；无暂存改动，无可见未跟踪文件 |
| 进行中的 Git 操作 | 未发现 merge / rebase / cherry-pick / revert |
| 新分支 | `rewrite/bootstrap-20261016-160959` |
| 新 worktree | 相对原工作区 `../yakyak-bootstrap-20261016-160959`，下文为 `$WORKTREE` |
| 带说明的旧 tag | `legacy/pre-rewrite-20261016-160959` |
| tag 解析到的 commit | `0a1cb3d0e9c3718fdd97b872f090c9600d0ae487` |
| bundle SHA-256 | `e793cf6cd026fabd70ed69b5a5840fb882b3a7cfb7d58eadebf985bce58b7cdd` |

备份位于私有本地目录 `$HOME/.local/share/yakyak-bootstrap/20261016-160959/`，目录权限 0700；下文为 `$BACKUP`，bundle 文件为 `pre-rewrite.bundle`，独立恢复目录为 `restore/`。公开报告不记录带用户名的绝对路径或含凭据的远端 URL。

真实执行：`git tag -a` → `git bundle create "$BACKUP/pre-rewrite.bundle" --all` → `git bundle verify` → `git clone --no-checkout` → 在恢复仓库 `rev-parse <tag>^{commit}`，全部退出 0。恢复所得 commit 等于基点；`git show <tag>:Cargo.toml` 和 `git show <tag>:src/main.rs` 与原对象逐字节一致。旧 Cargo.toml SHA-256 为 `061bbb91fa94147410fbcc07563b99c9e7065883b4d0811b9d2d43ade2148dc3`；旧 main.rs 为 `838c9f98bf49451101a7fb8ab9a76c6043968384e9e8451fc3b4e71771e80a5a`。

仓库非浅克隆，基点的 96 个跟踪文件无 Git LFS 指针 / 子模块。bundle 包含本地当时全部引用及其完整可达 Git 历史（包括缓存远端对象，已在恢复仓库 `cat-file -t` 验证），**不含未提交文件，不代表已备份实时远端 main**。本次没有尝试构建旧实现。

原目录 96 文件的字节 SHA-256 和权限模式、HEAD、分支、index tree、完整可见 status 前后一致；原 main 和缓存远端引用未更改。未读取或迁移被忽略的私人配置。原 README 未提交内容含疑似凭据，保持原地，未纳入新基线或报告；需用户自行撤销/轮换和处理。

`git worktree add -b <新分支> <新目录> <基点>` 退出 0；创建时树干净且 HEAD 等于基点。`git merge-base --is-ancestor <基点> <C>` 退出 0。没有删除 .git、orphan、reset --hard、git clean 或强推。

## 删除、保留与新增

删除前先形成逐文件迁移清单并提交到历史；完整现场清单另存忽略的 `artifacts/bootstrap/migration.json`。全部删除只发生在新 worktree 的已跟踪文件中。

| 旧文件类别 | 数量 | 处理 |
|---|---:|---|
| `LICENSE` | 1 | 原文、版权、字节不变 |
| `assets/images/logo.png` | 1 | 审核为独立品牌资料，450×373 PNG，原字节保留 |
| `src/**` | 79 | 旧通信实现退出；后来仅新增 CLI `src/main.rs` |
| `tests/**` | 3 | 旧 API / DB 测试退出；后来新增 bootstrap 测试和 fixture 约定 |
| `migrations/**` | 3 | 旧数据库迁移退出 |
| `test_api.sh` | 1 | 旧运行脚本退出 |
| `.github/workflows/rust.yml` | 1 | 旧 CI 退出；替换最小检查，当前工作流无发布 / 部署 |
| `Cargo.toml`、`Cargo.lock` | 2 | 旧依赖与锁退出；新最小 manifest，Cargo 重新生成 lock |
| `AUTH.md`、`CALL_FLOW.md`、`ROADMAP.md`、`README.md` | 4 | 旧功能宣传 / 文档退出；README 另建实际支持说明 |
| `.gitignore` | 1 | 替换为新工程与证据忽略规则 |

合计 96 个旧跟踪文件：保留 2，移除旧版本 94。旧内容可从 tag 读取，[历史索引](../archive/legacy-index.md)提供入口；活跃树中没有 `legacy/src` 副本。

LICENSE 迁移前后 SHA-256：`e0889b656d6240b04f8a72a6ebd86114963bdb920e24de0ac163ec89f257d0c0`。
logo 迁移前后 SHA-256：`894ecc6c310cab5248ab746aa25aeb1494674ee808f88008c99b928b2d299e00`。

新增内容：最小 Cargo 工程 / CLI、6 项进程测试、三入口脚本、固定工具链、只读 CI、协作约定及 roadmap / architecture / development / work / acceptance / references / handoff 文档。原始规划 ZIP / MD 和上游参考 clone 没有迁入活跃 Git 树。

## 工具链、依赖与参考版本

| 项目 | 实测版本 / 状态 |
|---|---|
| 本机 | macOS 27.0.1 (26A434)，Darwin arm64 |
| Rust | `rustc 1.96.0 (ac68faa20 2026-05-25)` |
| rustc 完整 commit | `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96` |
| host / LLVM | `aarch64-apple-darwin` / `22.1.2` |
| Cargo | `cargo 1.96.0 (30a34c682 2026-05-25)` |
| rustfmt / Clippy | `rustfmt 1.9.0-stable` / `clippy 0.1.96` |
| Python / Git | Python 3.14.6 / Git 2.50.1 (Apple Git-155) |
| 工程 | 单 package、单 binary `yakyak`，0.0.0，edition 2024，publish=false，MIT |
| 依赖 | metadata 中 dependencies=[]；cargo tree 仅 `yakyak v0.0.0` |
| 工具链锁 | 1.96.0，rustfmt / clippy；rust-version 与实测版一致 |
| 可选实验室工具 | SIPp、tshark 缺失；tcpdump 存在，权限/抓包未验证 |

使用已有 1.96.0 工具链，未更改全局默认或 shell 启动文件。本地脚本显式禁用 rustup 自动安装；隔离的空 RUSTUP_HOME 下三入口均失败且没有生成已安装工具链或下载产物。[rustup 官方说明](https://rust-lang.github.io/rustup/environment-variables.html)解释该行为。

RustPBX 固定到真实可读 commit [`0b1414bb3460a42abce8596f8f07d9b6d2889975`](https://github.com/restsend/rustpbx/commit/0b1414bb3460a42abce8596f8f07d9b6d2889975)。`clone --bare --depth 1`、解析 commit、`cat-file -t` 和读取 Cargo.toml / README.md / src/lib.rs / src/bin/rustpbx.rs 均退出 0，文件哈希见 [reference-lock.json](../references/reference-lock.json)。最初尝试不存在的 src/main.rs 返回 128，之后依据 Cargo manifest 找到真实入口并读取；失败记录保留。没有运行上游程序或声称其可构建，冻结范围保持不变。

## 正向检查与支持矩阵

以下除路径规范化外均对应实际命令。主证据为 `artifacts/bootstrap/full-check-macos.json`，从 `/tmp` 通过脚本绝对路径调用，验证任意当前目录均正确定位工程。`restored-copy-full-check.json` 记录从 C 全新导出的树在恢复全部负面改动后的完整成功；`post-negative-full-check.json` 记录原工作树再次成功。

| check.sh 顺序 | 退出码 | 实际结果 |
|---|---:|---|
| `python3 scripts/roadmap.py validate` | 0 | 27 章、125 目标，结构有效 |
| `python3 scripts/roadmap.py render --check` | 0 | 30 个生成文件一致，只读 |
| `python3 -m unittest discover -s scripts -p 'test_*.py' -v` | 0 | 21 tests，全部通过 |
| `cargo fmt --all -- --check` | 0 | 通过 |
| `cargo clippy --locked --all-targets -- -D warnings` | 0 | 通过 |
| `cargo test --locked --all-targets` | 0 | 6 个集成测试通过；无忽略 / 过滤项 |
| `cargo build --locked --release` | 0 | release 构建通过 |

二进制自身 unit target 是 0 tests，未把它计为功能成功；6 个实际测试位于 tests/bootstrap.rs。当前无库 / 文档测试，不声称运行了 doctest。无网络 / e2e 功能测试；它们明确未实现。

其他退出 0 的实际入口：`bash scripts/doctor.sh`、`bash scripts/test.sh bootstrap`、`cargo test --locked --test bootstrap -- --list`、`cargo metadata --locked --offline --format-version 1 --no-deps`、`cargo tree --locked --offline`、`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py show AC-01-02`、三个脚本的 `bash -n`。前两项也从 `/tmp` 调用。

| 真实进程测试 | 断言的行为 |
|---|---|
| `no_arguments_reports_unimplemented_service` | exit 0，空 stderr，精确“通信服务尚未实现”提示 |
| `help_describes_only_bootstrap_options` | exit 0，空 stderr，bootstrap / 范围 / 帮助参数 |
| `version_comes_from_cargo_package` | exit 0，版本来自 Cargo 构建信息 |
| `unknown_argument_is_a_usage_error` | 5 种错误输入均 exit 2，空 stdout，准确 stderr |
| `multiple_arguments_are_rejected` | 5 种组合均 exit 2，空 stdout，准确 stderr |
| `non_utf8_argument_is_rejected_without_panicking` | 非 UTF-8 参数 exit 2，不 panic |

源码仅标准库参数处理与终端输出，无 socket / 数据库 / 外部进程 / 公网调用；没有监听 5060、申请 RTP 端口或服务启动信息。该结论依据完整 main.rs 审阅和实际进程行为，未将它表述为抓包验收。

## 负面检查

所有变异均在 C 的临时导出副本发生，结束后已恢复并完整重跑，随后销毁该副本。正式源码和原用户目录没有负面注入残留。

| 场景 / 命令 | 实际退出码 | 观察 |
|---|---:|---|
| INDEX.md 追加漂移，`render --check` | 1 | 拒绝漂移且未改写文件 |
| 进度状态设 invalid，`validate` | 1 | 拒绝非法状态 |
| AC-00-01 换成 AC-99-99，`validate` | 1 | 拒绝不存在 ID |
| 仅将一项设 passed，`validate` | 1 | 拒绝缺少 commit / 证据 / 人工审阅 |
| `show AC-99-99` | 2 | 明确失败 |
| no-args 测试预期退出码由 0 改 1，`check.sh` | 101 | 对应真实引导测试 FAILED，错误透传，未运行后续 release 构建 |
| 清空 bootstrap 测试体，`test.sh bootstrap` | 1 | 明确拒绝 selected 0 tests |
| `test.sh network` / `e2e` | 各 2 | stderr 含 NOT_IMPLEMENTED |
| `test.sh unknown` / 无参数 / 额外参数 | 各 2 | 使用错误，非成功跳过 |
| 空 RUSTUP_HOME 下 doctor / test / check | 各 1 | 必需工具缺失，不自动安装、不吞错 |

此外，随包文档工具存在一个真实缺陷：证据章目录符号链接可把报告解析到仓库外。新增 `test_external_chapter_symlink_rejected` 先在旧实现上失败（exit 1），再补充 resolved path 必须位于仓库 root 的校验；修复后 21/21 通过。修改只限这项校验与对应回归，不改变 125 目标。

## INIT-01～INIT-14

| 编号 | 结果 | 命令 / 退出码与证据 |
|---|---|---|
| INIT-01 | verified | 原目录状态、96 文件字节/模式、HEAD/index 前后断言通过；source-preservation.json |
| INIT-02 | verified | tag / bundle verify / 独立 clone / 恢复对象读取均 0；setup-checks.json 与上方哈希 |
| INIT-03 | verified | worktree 创建及 ancestry 0；新分支继承基点，原 main 不变 |
| INIT-04 | verified | 94 旧文件退出、2 原样保留；LICENSE 哈希一致，迁移清单与 diff 审阅 |
| INIT-05 | verified | 原规划 27 章、125×5=625 个要求字段逐字一致；30 视图重生成无漂移；roadmap-audit |
| INIT-06 | verified | 真实上游 commit 对象与 4 文件读取为 0；reference-lock 与 reference-audit |
| INIT-07 | verified | 精确已装工具链；metadata/tree 0、单 package/binary、空依赖、Cargo 生成 lock 已提交 |
| INIT-08 | verified | 仅 CLI 源码审阅与六项真实进程测试；无通信实现 |
| INIT-09 | verified | `test.sh bootstrap` 0，6/6，包含合同四类及额外拒绝路径 |
| INIT-10 | verified | doctor/check/test 正向为 0；未实现/未知/空测试/断言失败均非零 |
| INIT-11 | partial | macOS 完整 check 0；Linux 容器缺工具探测 127；Linux 与两个远端 CI runner 为 not_run |
| INIT-12 | verified | Python 21/21；漂移/坏状态/坏 ID/无证据 passed 均拒绝；新增路径越界回归 |
| INIT-13 | verified | artifacts/target 均不在 Git 跟踪列表；证据 check-ignore 为 0；新文本凭据/私人路径检查无命中；125 项无 passed/reviewer |
| INIT-14 | delivered_pending_review | 实现本地提交 A/B/C 与本报告文档提交 D 可审阅；CURRENT、任务卡和剩余条件已更新，停在初始化 |

## Linux 与 CI 的真实边界

Docker daemon 可用，现有本地镜像 `98dea4a4300a` 的隔离 `--network none` 容器返回 Linux aarch64，但 Rust/Python 不存在，工具探测退出 127。未拉取新镜像或自动安装依赖；这不算 Linux 编译测试。Linux check 状态为 **not_run**，AC-00-02 未验收。

CI 配置为 `ubuntu-24.04` 和 `macos-15`；标签已核对[官方 runner 列表](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)。第三方 action 固定为 `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1`，由 [v7.0.1 release](https://github.com/actions/checkout/releases/tag/v7.0.1)和真实 Git 对象核验。工作流只允许 push / pull_request / 手动触发、contents: read，checkout 不保留凭据，运行同一 check.sh；无发布、上传、部署、生产 secret 或外呼步骤。

没有 push 或触发 GitHub Actions，所以 Ubuntu 和 macOS **远端 CI 都是 not_run**。本机 macOS 27 的通过也不冒充 macos-15 runner 已通过。

## 证据与规划完整性

输入交接包 SHA-256：`9fad4a3aefc1d7db0eefbbb3e0f9876f18327dbdd628b744ef0a770030a8c47d`；MANIFEST 中 79 项逐个校验通过。原规划 ZIP 内 targets.json 与新 targets.json 在分离 status/evidence、增加 schema_version 后结构相等；27 章元数据与 625 个目标要求字段一致，原独立 MD 与原 ZIP 内 ROADMAP.md 字节相同。progress.json 125 项均 not_started，reviewer 全为空；本次没有状态变化。

原始命令输出在忽略的 `artifacts/bootstrap/`，包括固定 C 上的 28 条正反向命令及环境、迁移、恢复、原目录保护记录。以下哈希供本地核验，不把原始日志提交到 Git：

| 相对 artifacts/bootstrap 的文件 | SHA-256 |
|---|---|
| SHA256SUMS.json（全部证据索引） | `b6d49c74f8a7d2067a29df1ffe3c74ba5fdad4e62c200c93da641cdbed7433ef` |
| verification-summary.json | `069428d46c79d89d45d522bd625d065661111f38baf45f0a705d0e4ca1d1150c` |
| full-check-macos.json | `a95d09b23f5368ce103bb022d784733f8082260ea07b6cf97d013c8c7ea87118` |
| negative-bootstrap-assertion.json | `e21c5095347f645a72f1632ff4a1edbed1ae36dd4d87fceb780061e03ef0bca9` |
| source-preservation.json | `869b2e6e5b3a5fae70614748b7ff703951095067fb98bab27e1a4f76f15ec219` |
| roadmap-audit/original-audit.json | `28a38e0cdab41fc1f7c82140d0c527676698b17835bbe467061896c67ea6af6c` |
| roadmap-audit/fix-validation.json | `9c2d7eb45c7e97c93ce88964dadd986e1a62696a0b0a696f34180819b4ae8c72` |
| reference-audit/evidence.json | `9b1509e926bd2780715b04b0779f0fc8585dacee6437da800d971d0df6ca0fb5` |

## 本地提交与停止位置

| 提交 | 内容 |
|---|---|
| `98780fc410aca444870e6dad3f96d62dd1f3ecc7`（A） | archive legacy baseline and reset active tree；旧树普通删除、许可/logo保留、迁移清单 |
| `f014ea61de4aea872742e266ce398a8b06d65f5f`（B） | import roadmap and bootstrap workflow；导入文档及原始工具 |
| `25157c636873f839c133c923c5a500af66d8f4c9`（C） | add minimal Rust bootstrap and local checks；实现、测试、脚本、工具链、CI、版本锁与文档工具修复 |
| 文档提交 D：`docs: record bootstrap verification and remaining gates` | 本报告、bootstrap-lock、当前任务、环境与剩余条件；不修改 C 的代码 |

报告不虚构包含自身内容的 commit 哈希。D 的实际哈希在最终交付消息给出，也可在交付分支执行 `git log -1 --format='%H %s' -- docs/handoff/BOOTSTRAP-REPORT.md` 获得。C 在最终分支历史中可达，未 amend / rebase。

未 push、未开 PR、未合并 main、未发版/上传、未外呼、未做生产变更。所有初始化改动和本地提交在新 worktree；原目录继续停留在原 main 和原未提交修改。

剩余事项：用户审阅三份 ADR 草案、冻结范围和证据流程；补齐 Linux 同 commit 完整检查；完成 SIPp/独立端、真实客户端、受控抓包、合法样例、音频设备等第 00 章实验；自行处理原目录疑似凭据。第 00 章逐项确认后，才由用户明确安排 AC-01-01。本次到此停止。
