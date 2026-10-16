# YakYak 仓库重建与开发工作流：Codex 初始化交接

版本：v1.0 · 日期：2026-10-16 · 目标仓库：`Ray-56/yakyak`

**本次任务：只完成仓库初始化，不实现通信功能。**

用户长期目标是亲自用 Rust 写通信内核，最终覆盖已确认的 RustPBX 非 AI 功能范围。既有规划为 6 阶段、27 章、125 项验收目标。本交接把这套规划落成一个安全、可复现、可逐项验收的开发仓库。工程脚手架可以由 Codex 完成；第 01 章起的 SIP、SDP、RTP、事务、Dialog 和呼叫业务默认由用户本人编写。

本文件是执行规范，**不是“初始化已完成”的报告**。包内所有版本锁待现场填写，125项进度初始全为未开始。用户没有授权本次对远端强推、发布、付费外呼或实施生产变更。

## 0. 执行入口与资料

交接包有三个重要入口：

| 位置 | 用途 | 是否整体复制到仓库 |
|---|---|---|
| `HANDOFF.md` | 本文件，Codex本次执行合同 | 仓库内已提供同文 `docs/handoff/BOOTSTRAP.md` |
| `repo-overlay/` | 待审阅、迁入的新文档和文档工具 | 在安全新分支中逐项迁入，不盲目覆盖 |
| `source-inputs/` | 上一轮原始规划 ZIP 和 MD | 不进入活跃仓库；校验与留档用 |

包内不包含通信实现，不包含生成的 Rust 工程，不包含已通过的 Rust/电话测试报告。Codex要按照本规范在用户的真实仓库里建立工程，而不是把包名或文档完成当作工程完成。

先读本文件，再读 `repo-overlay/AGENTS.md`、`docs/roadmap/SCOPE.md`、`docs/work/tasks/INIT-001.md`。不要逐章加载全部源码参考后再动手；本次只需知道全局边界和第00章要求。

## 1. 什么叫“重置”：重建文件，不重写历史

采用：**保留旧提交 → 创建可恢复检查点 → 在独立工作区建立重写分支 → 普通提交删除旧实现并加入新骨架。**

不采用删除 `.git`、orphan分支、`git reset --hard`、`git clean -fdx`、强推覆盖 main 或整仓 `rm -rf`。后续回到旧版通过明确历史引用检出；正常合并与回退由用户审核，不依赖破坏历史。

默认分支名：`rewrite/bootstrap-<YYYYMMDD-HHMMSS>`。
默认旧代码tag：`legacy/pre-rewrite-<YYYYMMDD-HHMMSS>`。
已有同名引用不得覆盖，选择新后缀。**当前本地 HEAD、远端 main 和用户未提交改动可能不同，必须分别记录。** 不默认“网页 main 就是用户正在工作的版本”。

### 1.1 只读盘点

在用户给定的仓库路径中检查：

```bash
git rev-parse --show-toplevel
git rev-parse HEAD
git branch --show-current
git status --short
git diff --stat
git diff --cached --stat
git worktree list
git rev-parse --is-shallow-repository
git ls-files
```

记录源路径、分支、HEAD、已跟踪改动、暂存改动、未跟踪文件的存在情况。远端URL如含凭据必须脱敏，不能写入公开报告。检查合并/变基等未完成操作；不能擅自结束用户的操作。

**原工作区非空不是删除这些文件的许可。** 首选独立worktree，原目录保持原样，用户的未提交改动不进入新基线。若这些改动可能是用户想保留的设计，列入遗留事项，不自动提交或丢弃。不要读取或打包 `.env`、私钥、录音等敏感内容来“方便交接”。

### 1.2 保存已提交历史并验证可恢复

在已核实的基点建立带说明的本地tag，另在仓库外的私有备份目录生成Git bundle。记录tag解析到的commit及bundle校验值。

以下命令是**在完成路径、基点和工作区检查后逐条执行的模板，不是一键清仓脚本**：

```bash
# BASE_SHA/TAG/BUNDLE/RESTORE_DIR必须由现场盘点得出；路径必须非空且不存在冲突。
git tag -a "$TAG" "$BASE_SHA" -m "YakYak pre-rewrite checkpoint"
git bundle create "$BUNDLE" --all
git bundle verify "$BUNDLE"
git clone "$BUNDLE" "$RESTORE_DIR"
git -C "$RESTORE_DIR" rev-parse "$TAG^{commit}"
```

最后一个结果必须等于 `BASE_SHA`，并抽查旧 `Cargo.toml` 和旧源码能从该tag读出。恢复目录应位于仓库外，不能污染主工程。**“旧代码可恢复”不要求旧代码现在能构建。**

bundle只保护Git对象与引用，不覆盖未提交文件、Git LFS内容或子模块实际内容。现场若存在LFS/子模块要单独清点并记录保护方式；不能把未验证的bundle称作完整备份。浅克隆不得宣称保留完整远端历史；能安全补全则补全，否则标记历史备份受限，暂停删除旧跟踪文件，但可继续文档整理。

原工作区的未提交/未跟踪文件继续原地保留。需要额外备份时只按已审阅路径复制到本地私有位置，不上传、不写入新提交，也不把凭据打进交接ZIP。

### 1.3 建立独立重写工作区

```bash
# NEW_WORKTREE必须是仓库外、尚不存在的目录；不覆盖用户已有目录。
git worktree add -b "$NEW_BRANCH" "$NEW_WORKTREE" "$BASE_SHA"
```

检查新工作区干净、HEAD等于基点、原工作区的暂存和未暂存状态未被改变。后续只在新工作区修改。若当前Codex沙箱不允许创建兄弟目录，可在允许的位置创建；无法安全创建则交付补丁和阻塞说明，不能借机清空原目录。

### 1.4 重建前先列迁移清单

本次公开页面可见的旧项目包含 `src/`、`tests/`、`migrations/`、`scripts/`、`.github/workflows/`，以及 `AUTH.md`、`CALL_FLOW.md`、旧 `ROADMAP.md` 等；这只是盘点起点，最终以现场 `git ls-files` 为准。

| 类别 | 处理原则 |
|---|---|
| `.git`、远端配置、提交历史、用户原目录 | 保留；不得删除或强制改写 |
| `LICENSE`及既有版权 | 原文保留，记录迁移前后哈希一致 |
| 旧源码、旧集成测试、迁移SQL、旧运行脚本 | 在新分支以普通删除提交退出活跃构建；tag中保留 |
| 旧依赖和lockfile | 新最小工程不继承；新lock由Cargo生成，不手工编辑旧lock |
| 旧功能文档、宣传性“已完成”、旧路线图 | 移出活跃入口；`docs/archive/legacy-index.md`指向历史位置 |
| 图片、品牌名称、其他独立资料 | 审核后选择保留或历史引用，不能猜测全部无用 |
| 旧CI、发布、部署、定时工作流 | 审核后退出新主线，替换最小检查；不能误触旧生产部署 |
| 私人配置、证书、录音、未跟踪文件 | 不迁入、不删除、不提交；保留在原目录 |
| 本包的新文档与文档工具 | 按目录映射迁入，先处理路径冲突 |

迁移清单写入 `docs/handoff/BOOTSTRAP-REPORT.md`；大量删除只限新工作区中已确认的跟踪文件。禁止使用“复制覆盖并自动删除目的端额外文件”的粗暴同步方式。不要在活跃仓库再放一份 `legacy/src/`；避免旧代码、旧依赖和旧代理指令继续混入搜索与构建。

## 2. 初始化后应该长什么样

```text
yakyak/
├── AGENTS.md                        # 精简、长期的代理协作约定
├── README.md                        # 当前实际能力与入口
├── CONTRIBUTING.md                  # 开发与提交方式
├── LICENSE                          # 原文保留
├── CHANGELOG.md                     # 只记实际改变
├── Cargo.toml                       # 单package，初期std-only
├── Cargo.lock                       # Cargo生成并提交
├── rust-toolchain.toml              # 精确固定已验证工具链
├── .gitignore
├── .gitattributes                   # 协议样例保留原始CRLF/字节
├── .github/workflows/ci.yml         # 不发布、不部署、不用生产secret
├── .github/pull_request_template.md
├── src/main.rs                      # 仅初始化CLI壳
├── tests/bootstrap.rs               # 真实运行二进制的引导测试
├── tests/fixtures/README.md          # 样例来源、字节和隐私约定
├── scripts/
│   ├── check.sh                     # Codex实现：统一本地/CI入口
│   ├── test.sh                      # Codex实现：显式测试组
│   ├── doctor.sh                    # Codex实现：只诊断不改系统
│   ├── roadmap.py                   # 本包提供：校验/生成/查看目标
│   └── test_roadmap.py              # 本包提供：文档工具回归测试
├── docs/
│   ├── README.md                    # 文档地图
│   ├── roadmap/                     # 目标、分章和当前进度
│   ├── architecture/                # 边界说明与ADR
│   ├── development/                 # 每日工作流、测试和环境
│   ├── work/                        # 当前工作、任务卡和问题
│   ├── acceptance/                  # 小型、脱敏、可提交的验收报告
│   ├── templates/                   # 任务/验收/ADR模板
│   ├── handoff/                     # 本交接与Codex执行报告
│   ├── references/                  # 版本锁、来源
│   └── archive/                     # 旧资料索引及原规划校验值
└── artifacts/                       # 本地日志/抓包/音频；整个目录忽略
```

这是目标清单；本包提供其中的文档与文档工具，Cargo、Rust壳、CI和环境脚本由Codex创建并现场验证。**不要预建27章对应的27个crate，不要先建立空SIP/RTP/队列/WebRTC模块。**

后续需要拆模块时按真实职责增加；有稳定复用需求再拆workspace。未来架构方向记录在文档中，不用大量 `todo!()` 类型把“未来可能”伪装成现在的实现。

## 3. 文档放置与唯一来源

| 信息 | 唯一手写来源 | 生成/展示方式 |
|---|---|---|
| 最终范围、自研与依赖边界 | `docs/roadmap/SCOPE.md` | 完整路线图引用 |
| 27章、125目标及其验收要求 | `docs/roadmap/targets.json` | `chapters/*.md`、`INDEX.md`、`ROADMAP.md` |
| 每项的真实状态、commit、证据 | `docs/roadmap/progress.json` | `PROGRESS.md` |
| 定量门槛与通用验收定义 | `docs/roadmap/TESTING_CONTRACT.md` | 章节及报告引用 |
| 今天具体做什么 | `docs/work/CURRENT.md`和一张任务卡 | 每次工作开始先读 |
| 为什么这样设计 | `docs/architecture/adr/*.md` | 代码注释/任务引用 |
| 真实测试结果 | `docs/acceptance/**` | 进度只链接，不抄写结果 |
| 原始抓包/日志/音频 | 忽略的 `artifacts/**`或授权私有存储 | 报告记录相对位置与哈希 |
| 上游参考版本 | `docs/references/reference-lock.json` | 不自动跟随main |

禁止手改生成文件后不更新源数据。禁止“文档里通过、JSON里没通过”的两套状态。上轮125个目标的编号、标题、实现任务与验收原文保持不变；本次只分离状态、自动生成视图并增加执行规则。

`AGENTS.md`只放稳定短约定和导航，不塞完整路线图。它不是沙箱或权限控制的替代品；代理仍要遵守用户授权与当前环境权限。

## 4. 本次允许Codex写哪些代码

允许：最小Cargo项目、无网络CLI壳、对应引导测试、本地检查脚本、文档校验、CI配置。允许修复本包文档工具的实际缺陷，但必须补回归测试并说明。

禁止：SIP解析器、URI模型、事务状态机、注册认证、SDP协商、RTP转发、Dialog、B2BUA、数据库、Web后台、WebRTC、AI、完整协议库封装。不得用现成 `rsipstack`、`rustrtc`、Asterisk、FreeSWITCH 代替自研学习核心。

### 4.1 最小Rust工程要求

使用单一package、单一binary，name为`yakyak`。初始化version建议`0.0.0`，`publish = false`，明确尚未到v0.1通信里程碑。edition定为2024；工具链至少支持该edition，使用**实际安装、可验证的精确稳定版本**，不是浮动字符串`stable`。`rust-version`不得宣称未测试的更低MSRV；可以与当前验证版本一致。

依赖初始为空，仅用std实现CLI；Tokio等在相应章节因真实需求加入并记录，不提前使用`features = ["full"]`堆依赖。保留MIT许可声明及原LICENSE。使用Cargo生成lockfile并提交。

精确版本写入`rust-toolchain.toml`，包含`rustfmt`和`clippy`组件；同时记录`rustc -Vv`、`cargo -V`。不强制升级用户全局默认，不修改shell启动文件。

### 4.2 CLI壳的行为合同

| 输入 | 行为 | 是否代表通信能力 |
|---|---|---|
| `yakyak --help` | 退出0；说明用途、当前仅bootstrap、已有参数 | 否 |
| `yakyak --version` | 退出0；版本来自Cargo构建信息 | 否 |
| 无参数 | 退出0；明确提示“通信服务尚未实现”，立即返回 | 否 |
| 未知参数或无效组合 | 退出2；stderr说明错误，不panic | 否 |

任何输入都不能监听5060、申请RTP端口、连接数据库或调用公网。不能输出“PBX ready”“SIP server started”等虚假启动信息。

`tests/bootstrap.rs`必须运行真实二进制，至少覆盖上述四类；断言退出码、stdout/stderr与当前支持范围。不能只测 `2+2` 或 `assert!(true)`。引导测试通过只证明脚手架行为，不计作任何SIP目标通过。

## 5. 工具、命令与CI合同

### 5.1 应当支持的日常入口

```bash
bash scripts/doctor.sh
bash scripts/check.sh
bash scripts/test.sh bootstrap
python3 scripts/roadmap.py validate
python3 scripts/roadmap.py render --check
python3 scripts/roadmap.py show AC-01-02
```

Python要求3.10或以上，仅文档工具使用标准库，不进入通信引擎运行时。缺少工具时明确报错，不能自动安装或执行远端shell。`doctor.sh`检查Git/Rust/Cargo/格式工具/Clippy/Python与可选SIPp、抓包工具，分别输出required/optional状态，不输出密钥和完整环境变量。

`check.sh`必须从任意工作目录正确定位仓库，启用严格错误传播；失败不得被`|| true`或日志管道吞掉。按顺序执行：

```bash
python3 scripts/roadmap.py validate
python3 scripts/roadmap.py render --check
python3 -m unittest discover -s scripts -p 'test_*.py' -v
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

若以后有文档测试，单列`cargo test --locked --doc`；若有workspace，明确增加覆盖，不把名字写了`all`就当全部验证。

### 5.2 不允许“零测试通过”

初始化只有`bootstrap`测试组存在。执行`bash scripts/test.sh network`或`e2e`，必须返回非零并明确`NOT_IMPLEMENTED`；未知组也非零。不得打印SKIP然后让它成为验收通过。

后续新增功能时，把实际测试目标和案例名写入任务卡；运行单项过滤前检查`cargo test ... -- --list`确实存在对应测试。**Cargo过滤后运行0个测试，不是目标通过。** 不要创建几十个空测试然后把CI涂绿。

### 5.3 CI

本地与CI调用同一`check.sh`。默认Linux为必跑基线，macOS为第二环境；使用明确runner标签，现场核对可用性。Rust从仓库精确工具链安装；锁依赖，不自动更新lockfile。

触发仅push/pull_request或手动；权限最小化为只读仓库。不加入发布、包上传、生产部署、外线呼叫、secret读取或远程写操作。第三方actions应使用经过核对的固定commit并注释版本来源；无法核实时保留阻塞说明，不能填随机SHA。

创建CI YAML不等于CI执行通过。没有Linux环境或没有远端CI运行权限时，报告`not_run`，保留`AC-00-02`未验收；可以交付本地通过的部分，不能声称双平台全部绿色。此次不自动push来触发CI。

### 5.4 要验证检查本身会失败

在临时副本做负面验证，不污染提交：制造一个文档漂移、一个非法状态或不存在的AC编号、一个缺少证据的passed项，文档检查必须拒绝；使一个引导断言失败，`check.sh`必须非零。测试未知/未实现组也必须失败。恢复后再跑完整检查。

## 6. INIT验收：与27章的功能验收分开

以下是本次初始化任务的14项验收。结果写`docs/handoff/BOOTSTRAP-REPORT.md`，初始全部`not_run`，不能由本包预先勾选。

| 编号 | 可观察的验收标准 | 所需证据 |
|---|---|---|
| INIT-01 | 记录真实基点与原目录状态，原未提交/未跟踪文件未被改动 | 脱敏盘点、前后状态比对 |
| INIT-02 | tag解析到基点，bundle验证及临时clone恢复成功 | SHA、校验值、恢复命令/退出码；备份范围说明 |
| INIT-03 | 新worktree与新分支继承旧历史，不改main、不强推 | 新分支、基点、ancestry检查、工作区路径的脱敏记录 |
| INIT-04 | 旧实现退出活跃构建；LICENSE哈希不变；旧CI不再触发产品部署 | 文件处置清单、diff审阅 |
| INIT-05 | 27章/125目标原文保持；唯一状态源，视图可再生成 | 校验报告、原文对比、生成前后无差异 |
| INIT-06 | 固定可读的RustPBX真实commit，不改变冻结范围 | reference-lock、取得对象和读取文件的记录 |
| INIT-07 | 精确工具链、单package、无运行时依赖、lock已提交 | toolchain/Cargo文件、cargo metadata或tree输出 |
| INIT-08 | 仅CLI壳，无通信实现和服务端口 | 变更审阅、运行行为、支持矩阵 |
| INIT-09 | 四类真实二进制测试通过，无空断言 | 测试名、数量、结果 |
| INIT-10 | check/doctor/test脚本可用；未实现组非零；错误不被吞 | 正反向命令结果 |
| INIT-11 | 已声明环境的完整检查真实运行；未运行环境明确标记 | 环境版本、命令、退出码、CI记录（若有） |
| INIT-12 | 数据/生成器校验能拒绝漂移、坏ID和无证据通过 | Python回归测试及临时副本负面测试 |
| INIT-13 | 本地证据不进Git，报告脱敏；没有伪造passed项 | git状态、忽略规则检查、progress检查 |
| INIT-14 | 交付完整执行报告与本地可审阅提交；停止在初始化 | commit清单、剩余事项、下一步明确 |

### 6.1 与第00章如何对应

| 原目标 | 本次能做什么 | 何时才可通过 |
|---|---|---|
| AC-00-01 | 保存历史、建立版本锁、列出环境 | 真实SHA和工具版本齐全，范围经用户确认 |
| AC-00-02 | 最小工程与持续检查 | 包含Linux在内的声明检查真实通过 |
| AC-00-03 | 准备实验室操作说明与记录模板 | SIPp/独立端、真实客户端、外部合法样例与抓包实际就绪 |
| AC-00-04 | 提供状态机/I/O/时钟ADR草案 | 用户完成设计评审并确认 |
| AC-00-05 | 建立状态、证据、已知问题制度 | 校验和记录流程实际跑通并经用户确认 |

**脚手架交付不等于第00章全过。** 无真实客户端、无抓包权限或无Linux环境时保持阻塞/待验收。不要为凑完成度实现假的电话端或虚假的PCAP。

本次可以形成“初始化工程已交付、部分环境/人工验证待完成”的诚实结论。任何必需条件缺失，不能宣布所有INIT项或整个第00章通过。

## 7. 用户日常怎样一点一点开发

每次只做一个AC，或同一个AC内部的一个小步骤。推荐一个任务卡对应一个可审阅PR/本地变更集，不跨多个章节批量生成。

1. 打开`docs/work/CURRENT.md`、当前章节和目标；确认依赖已通过。
2. 新建任务卡，写清输入、输出、拒绝路径、资源结束条件、本次不做什么。
3. 写出可失败的最小测试；保存真实失败原因，不把环境错误当预期失败。
4. 由用户实现最小代码；Codex默认只解释、评审或按指定范围补用例。
5. 跑目标测试、相关旧回归，再跑`check.sh`；与实际网络有关的目标另做独立端验证。
6. 保存commit、运行环境、命令、退出码和证据；提交小型脱敏报告。
7. 将状态标为`ready_for_review`。用户检查断言与证据后才改`passed`，更新CURRENT。

推荐分支命名：`feat/ch01-ac01-02-byte-boundaries`。提交消息引用目标，例如`test(sip): cover byte boundaries [AC-01-02]`、`feat(sip): enforce body length [AC-01-02]`。这是建议，不需要安装提交消息框架。

本地TDD可以短暂红灯，合并主线前必须全绿且证据齐全。需求变化要先改`targets.json`或SCOPE并留下ADR，再改测试与实现；不得为了现有代码通过而悄悄降低验收要求。

### 7.1 进度状态

`not_started → in_progress → ready_for_review → passed`；阻塞用`blocked`并写原因。发现回归可以从passed退回in_progress，保留旧报告和失效说明。不可使用“基本完成”“99%”代替可验证状态。

Codex默认不能替用户签署人工验收。没有明确授权或确认，不填写人类reviewer，不把执行者名字当审核通过。`passed`必须带完整实现commit、存在的报告路径、reviewer和日期。

### 7.2 一项功能需要什么证据

使用`docs/templates/ACCEPTANCE.md`。至少包含AC编号、规范/假设、测试前置、精确环境、命令、选中的测试数、预期与实际、失败路径、清理路径、commit、证据校验值、已知限制。

不适用的测试维度可写`N/A`并解释，例如纯解析器没有UDP端口；不能无脑让所有目标都做抓包，也不能对媒体目标只做内存状态断言。

### 7.3 如何避免报告commit的循环依赖

先提交实现及测试得到commit C，在干净的C上执行验收。之后用单独的文档提交D加入报告，报告记录tested_commit=C。D不应偷偷改通信源码。不要试图让报告引用包含自身内容的commit；若改了被测代码或amend了C，必须重跑并更正版本。

报告引用的commit必须在最终仓库历史中可达。若后续squash/rebase改变哈希，在最终提交上复验或保留已验证的可达引用；不能让证据指向永久丢失的本地提交。

## 8. 功能验收的统一标准

**格式检查/编译成功不是功能成功。测试写了不是测试跑了。测试绿了不是双向音频正确。**

| 功能性质 | 需要证明的事情 | 反例 |
|---|---|---|
| 解析/协商 | 正常、非法、边界、语义保持、独立语料 | 自己生成自己解析就声称互通 |
| 状态机/事务 | 状态、动作、定时器、重复、取消竞态 | 只打印“重传”但网络未发送 |
| 传输 | 独立进程真实收发、地址和连接选择 | 只检查函数返回Ok |
| 媒体 | 双方向、内容/解码、隔离、结束 | 只看RTP包计数或bridge.active |
| 呼叫 | 被叫真实振铃/接听/拒接，双侧正确收尾 | 自动接听冒充双端呼叫 |
| 资源 | 无流量仍能取消，任务join，端口/FD可释放 | 只设置running=false |
| 应用/管理 | 外部行为、权限、失败回退、配置真源 | 后台表单成功但引擎没加载配置 |

每项AC的具体验收要求已经在125项目标内，不另造一套低标准。需要实网/设备的测试不能由单元测试替代；缺设备就记录待验收。

### 8.1 证据与隐私

`docs/acceptance/`提交小型报告、脱敏统计和可公开的测试说明。`artifacts/`保存日志、抓包、音频和性能产物，不进Git。报告记录SHA-256、相对位置或用户授权的私有存储引用；不要写私人绝对路径。

示例协议样例使用保留测试地址/域名，记录来源与生成方式，保留原始字节。真实凭据、号码、录音、Authorization头不得进入公开仓库。不要创建一个`.gitignore`就声称秘密绝不会泄漏；暂存前仍需审查新增文件。

## 9. 初始化的建议提交划分

先备份验证，再按以下逻辑形成小提交，具体可按现场权限调整：

- `chore: archive legacy baseline and reset active tree`：旧实现退出活跃主线，保留许可与历史索引。
- `docs: import roadmap and bootstrap workflow`：目标、进度、ADR草案、开发与验收规范。
- `chore: add minimal Rust bootstrap and local checks`：CLI壳、测试、脚本、工具链和CI。
- `docs: record bootstrap verification and remaining gates`：真实报告、未完成事项、当前任务更新。

这里的reset指文件树重建，不是执行`git reset --hard`。不要用`git add -A`盲目把用户文件提交；按新工作区已审阅的文件清单暂存。不能自动push、开PR、合并main、创建GitHub release、上传包或改变远端保护设置。

若当前环境不允许Git写操作，产出diff/补丁及可执行说明，明确没有创建commit，不编造SHA。

## 10. Codex最终要向用户交付什么

交付`docs/handoff/BOOTSTRAP-REPORT.md`，包含：

1. 实际分支、基点、旧tag、bundle验证和恢复结果；原工作区是否完全保留。
2. 删除/保留/新增文件摘要，LICENSE校验结果。
3. 精确工具链与环境；RustPBX真实参考SHA或具体阻塞。
4. INIT-01至INIT-14的真实结果，命令、退出码、证据位置。
5. Rust测试与文档测试的数量、覆盖行为；任何未运行/跳过项。
6. 125目标的状态变化；默认没有自动passed项，绝不批量标记27章完成。
7. 本地commit清单；没有push、没有外呼、没有生产变更。
8. 用户接下来只需完成的事项：审阅ADR、补齐第00章实验室/环境、确认后开始AC-01-01。

完成这些后停止。不要顺手实现第一版解析器，不要自动开始第01章。

## 11. 本次交接的验证边界

本交接包只验证文档组织、125目标保留和随包Python文档工具；验证记录见包根`VALIDATION.md`。它不证明用户仓库已经重建。包内版本锁空值是明确待办，不能被Codex删除来让检查“通过”。

本次可公开读取仓库文件，但未得到可验证的完整上游SHA，也未运行Rust编译或实际电话测试。精确现场结果必须由接手的Codex重新取得并记录。来源见`docs/references/SOURCES.md`。

**本次任务的成功，是把你之后亲自写通信代码的工作台准备好，并能逐项验收；不是让Codex提前把整套通信引擎写出来。**
