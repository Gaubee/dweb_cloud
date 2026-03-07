# Auth Token And Billing Spec

## 文档状态

- Status: Active
- Scope: challenge、signature、token 与计费边界

## 当前认证模型

状态：`Implemented`

- 用户输入任意 secret input
- 本地派生助记词与公钥身份
- 通过 challenge + signature 换取 app token

## Token 约束

状态：`Implemented`

- token 绑定 `account + app_id`
- token 有过期时间
- token 可扩展为可撤销模型

## 当前计费边界

状态：`Exploration`

- v1 只实现 token 签发，不实现正式收费逻辑
- 正式商业化后，token 仍应与 entitlement 绑定
