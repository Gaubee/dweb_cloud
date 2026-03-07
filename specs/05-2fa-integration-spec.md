# 2FA Integration Spec

## 文档状态

- Status: Active
- Scope: `dwebCloud` 如何为 `gaubee-2fa` 提供 WebDAV 能力

## v1 集成目标

状态：`In Progress`

- 默认 app 为 `gaubee-2fa`
- CLI 可签发用于 2FA 的 WebDAV 配置
- 2FA 通过手动输入 host/account/password 完成接入

## 验收要求

- 本地启动 dwebCloud
- 为 `gaubee-2fa` 签发 token
- 2FA 可 push/pull 加密快照
