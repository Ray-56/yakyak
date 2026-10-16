# 旧实现索引

旧分支：`main`。旧 commit：`0a1cb3d0e9c3718fdd97b872f090c9600d0ae487`。
旧 tag：`legacy/pre-rewrite-20261016-160959`（带说明，解析到上述 commit）。

Git bundle 已通过 verify 和独立 clone 恢复，SHA-256：`e793cf6cd026fabd70ed69b5a5840fb882b3a7cfb7d58eadebf985bce58b7cdd`。
只覆盖本地可达 Git 对象与引用，不覆盖未提交文件，也不代表已备份实时远端 main。

LICENSE 原文和版权保留，SHA-256：`e0889b656d6240b04f8a72a6ebd86114963bdb920e24de0ac163ec89f257d0c0`。
`assets/images/logo.png` 保留为品牌资料，SHA-256：`894ecc6c310cab5248ab746aa25aeb1494674ee808f88008c99b928b2d299e00`。

旧 src、tests、migrations、test_api.sh、依赖、AUTH.md、CALL_FLOW.md、ROADMAP.md、README 和 CI 从活跃目录退出；历史中可读。完整清单与检查结果见 [初始化报告](../handoff/BOOTSTRAP-REPORT.md)。

```bash
git show legacy/pre-rewrite-20261016-160959:Cargo.toml
git show legacy/pre-rewrite-20261016-160959:src/main.rs
```

原工作区未提交修改继续原地保留，未进入新工程或 bundle。
