# 原始规划存档说明

上一轮 v1.0 原始 Markdown 和 ZIP 随交接包放在 `source-inputs/`，不要整包导入活跃仓库。
此目录的 receipt.json 保存它们的 SHA-256 和迁移规则。需求字段保持原文；只有状态、视图和路径组织改变。

当前唯一目标源是 `../../roadmap/targets.json`，当前唯一状态源是 `../../roadmap/progress.json`。
原始规划里的静态“未开始”字样仅表示创建时状态，不是第二套实时进度。

本交接包不携带旧 Rust 代码。旧实现由初始化现场建立的 Git tag、离线备份与历史提交保留。
