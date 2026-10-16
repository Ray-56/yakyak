# 检查工具

随包提供：roadmap.py（生成/校验/查看目标）、test_roadmap.py（工具回归）。标准库Python3.10+。

INIT-001由Codex补齐：check.sh、test.sh、doctor.sh，合同见docs/handoff/BOOTSTRAP.md。当前没有这些脚本不能假称工程check已经通过。

工具不得自动标记passed，不删除用户文件，不修改全局环境，不访问生产服务。只读校验不能触发生成；render --check不能改写文件。
