# 2FA WebDAV Quickstart

本文档描述如何把 `dwebCloud` 作为 `Gaubee 2FA` 的 `WebDAV Provider` 使用。

## 前置条件

- `dwebCloud` 服务已启动
- 你持有一份自己的 secret input
- 2FA Web 已可访问：<https://gaubee.github.io/2fa/>

## 步骤 1：启动 dwebCloud

本地 Rust 方式：

```bash
cargo run -p dweb-cloud-server -- --http 127.0.0.1:9080 --data-dir ./.data
```

或者 Docker：

```bash
docker compose up -d --build
```

## 步骤 2：签发 WebDAV 凭据

```bash
cargo run -p dweb-cloud-cli -- token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

或者：

```bash
docker compose exec dweb-cloud dweb-cloud-cli token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

返回结果示例：

```json
{
  "appId": "gaubee-2fa",
  "webdavBaseUrl": "http://127.0.0.1:9080/dav/gaubee-2fa",
  "username": "<public_key_hex>",
  "password": "<app_scoped_token>",
  "expiresAtMs": 1775472000000
}
```

## 步骤 3：配置 2FA WebDAV

在 2FA 页面中填写：

- `WebDAV Host = webdavBaseUrl`
- `WebDAV Account = username`
- `WebDAV Password = password`
- `Vault Secret = 你自己的本地加密口令`

然后依次操作：

1. 点击“验证配置”
2. 点击“推送”把本地条目上传到远端
3. 在另一台设备或清空本地数据后点击“拉取”验证恢复

## 当前已验证的闭环

- `challenge -> signature -> token` 正常工作
- `PUT / GET / MKCOL` 已能满足 2FA 当前 `WebDAV` 使用方式
- 2FA 已能通过 `push / pull` 恢复加密快照

## 注意事项

- `Vault Secret` 只在 2FA 本地使用，`dwebCloud` 不保存明文
- token 当前有过期时间，过期后需要重新签发
- `dwebCloud` 目前仍是 `v1` 原型，不提供正式多租户运维能力
