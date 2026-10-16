# 来源、标准与核验边界

核验日期：2026-10-16。以下为公开官方文档、源码或 IETF/RFC 原始规范；没有使用第三方性能宣传作为验收目标。仓库 main 会变化，第 00 章必须记录实际参考 commit。

## [R01] RustPBX README：功能类别与版本边界

`https://github.com/restsend/rustpbx`

## [R02] RustPBX Addon Architecture：公开与商业扩展分界

`https://raw.githubusercontent.com/restsend/rustpbx/main/src/addons/README.md`

## [R03] RustPBX RWI：命令、事件、通话腿、媒体及监督接口

`https://raw.githubusercontent.com/restsend/rustpbx/main/docs/rwi.md`

## [R04] RustPBX media crate：媒体模块职责

`https://raw.githubusercontent.com/restsend/rustpbx/main/crates/rustpbx-media/src/lib.rs`

## [R05] RustPBX API Integration Guide：管理、路由、事件接口

`https://raw.githubusercontent.com/restsend/rustpbx/main/docs/api_integration_guide.md`

## [R06] RustPBX 媒体端到端测试：网络验收参考

`https://raw.githubusercontent.com/restsend/rustpbx/main/tests/proxy_e2e/test_media_e2e.rs`

## [R07] RustPBX MediaBridge：转发、转码与视频边界

`https://raw.githubusercontent.com/restsend/rustpbx/main/crates/rustpbx-media/src/media_bridge.rs`

## [R08] RustPBX CallApp：通话应用与生命周期

`https://raw.githubusercontent.com/restsend/rustpbx/main/src/call/app/mod.rs`

## [R09] RustPBX 配置导航：用户、线路、管理和存储

`https://raw.githubusercontent.com/restsend/rustpbx/main/docs/configuration.md`

## [R10] RustPBX SipFlow：信令、媒体与存储

`https://raw.githubusercontent.com/restsend/rustpbx/main/docs/config/08-sipflow.md`

## [R11] RustPBX cluster_sync：状态广播参考，不等于无损接管保证

`https://raw.githubusercontent.com/restsend/rustpbx/main/src/proxy/cluster_sync.rs`

## [S01] RFC 3261：SIP 基础

`https://www.rfc-editor.org/rfc/rfc3261.html`

## [S02] RFC 6026：INVITE 成功响应事务修正

`https://www.rfc-editor.org/rfc/rfc6026.html`

## [S04] RFC 4475：SIP 异常与边界测试消息

`https://www.rfc-editor.org/rfc/rfc4475.html`

## [S05] RFC 8760：SIP Digest 认证更新

`https://www.rfc-editor.org/rfc/rfc8760.html`

## [S06] RFC 8866：SDP 格式

`https://www.rfc-editor.org/rfc/rfc8866.html`

## [S07] RFC 3264：SDP Offer/Answer

`https://www.rfc-editor.org/rfc/rfc3264.html`

## [S08] RFC 3550：RTP 与 RTCP

`https://www.rfc-editor.org/rfc/rfc3550.html`

## [S09] RFC 3262：100rel 与 PRACK

`https://www.rfc-editor.org/rfc/rfc3262.html`

## [S10] RFC 3311：UPDATE

`https://www.rfc-editor.org/rfc/rfc3311.html`

## [S11] RFC 4028：Session Timer

`https://www.rfc-editor.org/rfc/rfc4028.html`

## [S12] RFC 4733：RTP telephone-event/DTMF

`https://www.rfc-editor.org/rfc/rfc4733.html`

## [S13] RFC 3515：REFER

`https://www.rfc-editor.org/rfc/rfc3515.html`

## [S14] RFC 3891：Replaces

`https://www.rfc-editor.org/rfc/rfc3891.html`

## [S15] RFC 6665：SIP 事件订阅

`https://www.rfc-editor.org/rfc/rfc6665.html`

## [S16] RFC 4235：Dialog 状态事件包

`https://www.rfc-editor.org/rfc/rfc4235.html`

## [S17] RFC 3428：SIP MESSAGE

`https://www.rfc-editor.org/rfc/rfc3428.html`

## [S18] RFC 5764：DTLS-SRTP

`https://www.rfc-editor.org/rfc/rfc5764.html`

## [S19] RFC 7118：SIP over WebSocket

`https://www.rfc-editor.org/rfc/rfc7118.html`

## [S20] RFC 3581：rport 与返回路径

`https://www.rfc-editor.org/rfc/rfc3581.html`

## [S21] RFC 8825：WebRTC 协议总览

`https://www.rfc-editor.org/rfc/rfc8825.html`

## [S22] RFC 8445：ICE

`https://www.rfc-editor.org/rfc/rfc8445.html`

## [S23] RFC 8656：TURN

`https://www.rfc-editor.org/rfc/rfc8656.html`

## [T01] SIPp 官方文档：独立 XML 测试场景

`https://sipp.readthedocs.io/en/latest/scenarios/ownscenarios.html`

## 延伸阅读：到相应章节再核对

除上面已查阅内容外，实施时应检查各 RFC 的 updates/obsoletes/errata。MWI 查 RFC 3842，SIP 服务定位查 RFC 3263，SIP Outbound 查 RFC 5626，RTP profiles 查 RFC 3551，RTCP mux 查 RFC 5761，SRTP 查 RFC 3711 及所用加密 profile 的更新。这里是阅读入口，不意味着本次完成了这些协议的全文审阅。

资料与设计冲突时先回到对应规范和真实互通证据；参考项目源码不是协议标准。建议保留每章自己的“规范条款→用例→实现位置”表。
