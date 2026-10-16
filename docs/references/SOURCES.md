# 初始化交接的来源与核验边界

整理日期：2026-10-16（Asia/Singapore）。以下为公开资料查阅，不是代码编译或运行证明。

## 本次直接读到的仓库信息

1. YakYak 根目录（main）：https://github.com/Ray-56/yakyak
   可见 src、tests、docs、migrations、scripts、.github/workflows，以及旧的 AUTH.md、CALL_FLOW.md、ROADMAP.md 等。现场必须重新列清单，不能把这个列表当作穷尽列表。
2. 依赖清单：https://raw.githubusercontent.com/Ray-56/yakyak/main/Cargo.toml
   当前公开内容是 edition 2021、版本 0.1.0；包含 rsip、webrtc、rtp、axum、sqlx 等，默认启用 postgres。新核心自研路线不直接继承这套依赖。
3. 许可证：https://raw.githubusercontent.com/Ray-56/yakyak/main/LICENSE
   可见 MIT 及既有版权行。初始化保留文件原文，不擅自改作者、年份或许可。
4. 忽略规则：https://raw.githubusercontent.com/Ray-56/yakyak/main/.gitignore
   现场复核后保留必要私人文件隔离；新规则不忽略 Cargo.lock。
5. 功能参考：https://github.com/restsend/rustpbx
   本交接不重新定义27章范围；仍以已确认的路线图为准，AI仍排除。

## 工具行为的官方资料

- Git worktree：https://git-scm.com/docs/git-worktree
- Git bundle：https://git-scm.com/docs/git-bundle
  bundle保存Git对象与引用，不自动备份工作区未提交、未跟踪和被忽略文件。
- Cargo项目布局：https://doc.rust-lang.org/cargo/guide/project-layout.html
- Cargo.lock：https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
- rustup工具链覆盖：https://rust-lang.github.io/rustup/overrides.html
- Codex项目约定：https://developers.openai.com/codex/agent-configuration/agents-md
- Codex协作实践：https://developers.openai.com/codex/learn/best-practices
- GitHub Actions runner：https://docs.github.com/en/actions/reference/runners/github-hosted-runners

## 本次没有取得或没有执行

未取得两个仓库可验证的完整 commit SHA；容器 git 读取因域名解析失败，GitHub提交API也未返回可用结果。
因此版本锁保持空值，不能把现在的 main 文本误称为固定提交快照。
未在用户本地执行任何 Git 命令、备份、删除、提交或推送。
未运行 Rust 构建或 SIP/媒体测试。交接包内 Python 文档工具的检查结果另见包根 VALIDATION.md。
所有目录规划、初始化编号、门槛和工作流都是本次给 YakYak 设计的工程约定，不是上游项目要求。
