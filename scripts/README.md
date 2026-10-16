# 检查工具

所有脚本从任意工作目录调用时均定位本仓库。Bash、Python 3.10+ 与精确 Rust 工具链必须预先可用；本地脚本不会安装工具。

- `bash scripts/doctor.sh`：诊断 required / optional 工具，必需项缺失返回非零。
- `bash scripts/check.sh`：文档校验、生成漂移检查、Python 回归、Rust 格式、Clippy、测试和 release 构建。
- `bash scripts/test.sh bootstrap`：列出并运行真实 CLI 测试。`network` / `e2e` 返回非零 `NOT_IMPLEMENTED`；未知组也失败。
- `python3 scripts/roadmap.py validate`：校验目标、状态和证据结构；`validate --git` 额外核对状态中引用的 commit 可达性。
- `python3 scripts/roadmap.py render --check`：只读检查生成视图；需要更新时运行 `render`。
- `python3 scripts/roadmap.py show AC-01-02`：只显示指定目标。

工具不会自动标记 passed。新文档工具逻辑由 `python3 -m unittest discover -s scripts -p 'test_*.py' -v` 覆盖。
