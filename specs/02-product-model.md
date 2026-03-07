# Product Model

## 文档状态

- Status: Active
- Scope: 核心对象、账号、app 空间与 token 模型

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

### WebDAV Identity

- `username = public_key_hex`
- `password = app-scoped token`
- `base_url = /dav/:app_id`
