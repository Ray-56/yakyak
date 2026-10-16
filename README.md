# YakYak

Talk the Yak, Connect the Pack!

用Rust从头学习和实现可编程通信引擎。最终范围参考RustPBX的非AI能力；当前处于重写初始化阶段，**尚不支持SIP、通话或媒体转发**。

## 从哪里开始

- 人工开发规则：[CONTRIBUTING.md](CONTRIBUTING.md)
- 当前工作：[docs/work/CURRENT.md](docs/work/CURRENT.md)
- 文档地图：[docs/README.md](docs/README.md)
- 27章路线图：[docs/roadmap/INDEX.md](docs/roadmap/INDEX.md)
- 当前进度：[docs/roadmap/PROGRESS.md](docs/roadmap/PROGRESS.md)
- 本次初始化：[docs/handoff/BOOTSTRAP.md](docs/handoff/BOOTSTRAP.md)

## 当前运行方式

这是交接模板，迁入时由Codex按真实结果更新。初始化完成后仅有`--help`、`--version`和明确的未实现提示，不启动网络服务。工程检查入口为`bash scripts/check.sh`。依赖环境见[环境说明](docs/development/ENVIRONMENT.md)。

不要使用旧版本README中的数据库、STUN/TURN或PBX启动说明来运行新骨架。旧实现通过[历史索引](docs/archive/legacy-index.md)中现场填写的tag恢复。功能完成只以测试和验收报告为准，不以文件或模块存在为准。
