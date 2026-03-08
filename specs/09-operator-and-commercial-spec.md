# Operator And Commercial Spec

## 文档状态

- Status: Active
- Scope: 自托管 operator 能力、套餐配置与商业化边界

## operator 当前能力

状态：`Implemented / In Progress`

- 本地 `admin stats` CLI
- `plans.json` 套餐配置
- Docker / 反向代理 / smoke 运维路径

## 当前套餐模型

- `free-local`: 浏览器本地或自托管使用，不需要官方 entitlement
- `cloud-sync-1000`: 官方托管写入能力，容量上限 1000 条 OTP
- 套餐模型可描述到期后只读与保留时长

## 暂未实现

- entitlement 持久化
- payment provider 接入
- hosted quota enforcement
- operator Web 管理台
