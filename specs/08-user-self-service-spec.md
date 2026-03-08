# User Self-Service Spec

## 文档状态

- Status: Active
- Scope: 用户公开查询、账户自助与 token 生命周期管理

## 当前目标

状态：`Implemented / Ready for Acceptance`

- 提供 `public apps`
- 提供 `public plans`
- 提供 `account overview`
- 提供 `account tokens list`
- 提供 `account token revoke`

## 交互约束

- 账户自助接口必须使用 challenge + signature
- challenge 消费后即失效，避免重放
- revoke 后 token 不得继续写入 WebDAV
- 当前只提供 API 与 CLI，不提供 Web 控制台
