# YakYak

Talk the Yak, Connect the Pack!

用Rust从头学习和实现可编程通信引擎。最终范围参考RustPBX的非AI能力；当前处于重写初始化阶段，**尚不支持SIP、通话或媒体转发**。

## 从哪里开始

- 人工开发规则：[CONTRIBUTING.md](CONTRIBUTING.md)
- 当前工作：[docs/work/CURRENT.md](docs/work/CURRENT.md)
- 文档地图：[docs/README.md](docs/README.md)
- 27章路线图：[docs/roadmap/INDEX.md](docs/roadmap/INDEX.md)
- 当前进度：[docs/roadmap/PROGRESS.md](docs/roadmap/PROGRESS.md)
- 当前不支持的功能：[docs/UNSUPPORTED.md](docs/UNSUPPORTED.md)
- 本次初始化：[docs/handoff/BOOTSTRAP.md](docs/handoff/BOOTSTRAP.md)

## 当前运行方式

当前是 `0.0.0`、edition 2024、仅标准库的单一二进制，固定 Rust `1.96.0`。仅支持 `--help`、`--version`；无参数会提示“通信服务尚未实现”并退出。

```bash
bash scripts/doctor.sh
bash scripts/check.sh
bash scripts/test.sh bootstrap
cargo run --locked -- --help
```

检查结果与未完成条件见[初始化报告](docs/handoff/BOOTSTRAP-REPORT.md)，依赖环境见[环境说明](docs/development/ENVIRONMENT.md)。

不要使用旧版本README中的数据库、STUN/TURN或PBX启动说明来运行新骨架。旧实现通过[历史索引](docs/archive/legacy-index.md)中现场填写的tag恢复。功能完成只以测试和验收报告为准，不以文件或模块存在为准。
