# 当前工作

阶段：第 00 章，记录已确认范围并补齐基线证据。
当前任务：[AC-00-01](tasks/AC-00-01.md)。状态：in_progress，以 progress.json 为准。

2026-10-18（Asia/Shanghai），用户在本会话明确确认 SCOPE v1.0 的功能范围、排除项及自研/复用边界。两个历史 tag 已分别在独立本地 clone 中检出并验证干净，参考 commit 沿用初始化时的真实对象与文件证据。用户授权的 Wireshark 4.0.7 → 4.6.9 更新已完成，旧版保留在仓库外私有备份中。

用户安装软件后，已实测 SIPp 3.8.0-TLS-PCAP、Linphone 6.2.3、Wireshark/TShark 4.6.9 的版本命令；两个 app 的签名和 macOS 信任检查均通过。SIPp `-v` 返回 99，为官方定义的正常无呼叫退出，已保留原始退出码。实际测试端版本缺项已补齐，见 [test-tools-lock.json](../references/test-tools-lock.json)及 [AC-00-01 记录](../acceptance/ch00/AC-00-01.md)。图形界面、协议场景、抓包与音频仍未验证；没有修改权限或系统 PATH。整个 AC 尚未人工验收。

初始化任务 [INIT-001](tasks/INIT-001.md) 保持 ready_for_review，INIT-11 部分完成。此前检查和本地提交见[初始化报告](../handoff/BOOTSTRAP-REPORT.md)；Linux 与远端 CI 在初始化交付时为 not_run，后续 main 推送的 CI 结果仍须另行核对。除 AC-00-01 外的 124 项保持 not_started，没有目标被标为 passed。

## 下一步（尚未执行）

1. 形成包含 AC-00-01 完整记录的可达提交并填入真实 SHA，完成文档检查后再交用户审阅整个 AC；工具版本记录可复用于 AC-00-03。
2. 另行审阅初始化变更、三个 ADR 草案与证据流程；本次范围确认没有将 ADR 改为 Accepted。
3. 核对 main 推送后的实际 CI 结果，并在具备精确工具链的 Linux 环境对初始化报告的 tested_commit 运行 check.sh。
4. 补齐 AC-00-03 受控发包、抓包权限、音频设备和合法样例；逐项完成第 00 章验收后，再由用户明确安排 AC-01-01。

本次已补齐用户所装工具的版本记录，保留已有范围确认、历史核验和 Wireshark 更新证据；文档尚未提交，没有创建第 01 章任务或实现通信核心。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告末节。
