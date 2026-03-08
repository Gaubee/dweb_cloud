# Product Model

## 文档状态

- Status: Active
- Scope: 核心对象、账号、app 空间、plan 与开发者模式模型

## 核心对象

### Account

- 由密钥派生的 `public_key_hex`
- 是用户在 `dwebCloud` 中的稳定身份标识

### App Space

- 每个 app 拥有独立空间
- 同一用户在不同 app 下的数据完全隔离

### Token

- 由 challenge + signature 换取
- 与 `account + app_id` 绑定
- 用作 `WebDAV` 凭据中的 password

### Management Scope

- 固定 scope：`dweb-cloud-account`
- 用于账户自助接口签名认证
- 不直接等价于任意 app 的 WebDAV token

### Plan

- 描述商业套餐、容量上限、到期行为与保留时长
- 当前通过 `config/plans.json` 配置并公开只读暴露

### Developer Mode

- 通过 `DWEB_CLOUD_DEVELOPER_MODE=true` 开启
- 提供面向 app builder / operator 的元信息接口
- 用于本地集成与未来 `dweb_chain` 基础设施调试
