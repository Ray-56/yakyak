# 目标与进度

手写要求源：`targets.json`。手写状态源：`progress.json`。静态范围：`SCOPE.md`。验收规则：`TESTING_CONTRACT.md`。

`chapters/*.md`、`INDEX.md`、`PROGRESS.md`、`ROADMAP.md`由`scripts/roadmap.py render`生成。CI运行`render --check`只比较，不写文件。

初始保留27章125项。所有目标初始未开始，未因导入文档或引导测试自动通过。第00章由初始化任务部分准备，环境与人工评审要求不取消。

常用命令：

```bash
python3 scripts/roadmap.py show AC-01-02
python3 scripts/roadmap.py validate
python3 scripts/roadmap.py render
python3 scripts/roadmap.py render --check
```

`validate`检查数据和报告路径；`--git`可额外检查已填写的commit对象在当前仓库可达。它只核对结构和引用，不能证明报告内容真实，也不能代替人工验收。
