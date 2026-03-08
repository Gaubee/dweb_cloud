# Auth Token And Billing Spec

## 文档状态

- Status: Active
- Scope: challenge、signature、token、账户自助认证与计费边界

## 当前认证模型

状态：`Implemented`

- 用户输入任意 secret input
- 本地派生助记词与公钥身份
- 通过 challenge + signature 换取 app token
- 通过固定 `dweb-cloud-account` scope 完成账户自助认证

## Token 约束

状态：`Implemented`

- token 绑定 `account + app_id`
- token 有过期时间
- token 支持撤销
- token 列表与 revoke 仅允许账号自身通过签名请求访问

## 当前计费与套餐边界

状态：`In Progress / Exploration`

- 当前通过 `config/plans.json` 描述 `free/self-host` 与 `hosted` plans
- hosted 套餐可表达：价格、容量上限、到期后只读、保留时长
- v1.5 仍未实现 entitlement 写入、支付回调与配额执行
