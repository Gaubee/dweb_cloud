# dwebCloud

基于 `Rust` 的应用存储云内核，提供 `Web3` 风格密钥身份、应用隔离的存储空间，以及面向客户端应用的 `WebDAV` 接入能力。

当前仓库聚焦 `v1`：为 `gaubee-2fa` 提供本地文件系统版 `WebDAV + 签名换 token` 最小闭环。

## 文档

- [ROADMAP.md](./ROADMAP.md)：愿景、阶段目标、待验收事项与当前优先级总控。
- [specs/README.md](./specs/README.md)：产品与模块规格真源。
- [AGENTS.md](./AGENTS.md)：开发元规则、最佳实践与标准工作流。
- [CHAT.md](./CHAT.md)：来自 2FA 仓库拆分的原始需求轨迹。

## 目录

- `server/`：Rust HTTP/WebDAV 服务。
- `cli/`：Rust CLI，用于签名换取 app-scoped WebDAV 凭据。
- `crates/identity-core/`：助记词派生、公钥身份、挑战签名与验签。
- `crates/storage-core/`：本地文件系统存储、challenge/token 持久化、app 空间隔离。
- `config/`：默认 app 配置。
- `infra/`：部署与运行说明。
- `specs/`：产品与工程规格。

## 当前状态

已完成：

- 工作区骨架与规格文档真源。
- 本地文件系统后端。
- challenge + signature + token 签发接口。
- app-scoped WebDAV 本地服务。
- `gaubee-2fa` 的默认 app 注册。

后续推进：

- `S3` backend。
- Web 网盘管理页。
- 在线授权回调与 app 接入流程。

## 本地开发

启动服务：

```bash
cargo run -p dweb-cloud-server -- --http 127.0.0.1:9080 --data-dir ./.data
```

签发 2FA 使用的 WebDAV 凭据：

```bash
cargo run -p dweb-cloud-cli -- token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

CLI 会返回：

- `webdavBaseUrl`
- `username`
- `password`
- `expiresAtMs`

## 与 2FA 联调

当前最小联调流程：

1. 启动 `dweb-cloud-server`。
2. 用 `dweb-cloud-cli token issue` 获取 WebDAV 凭据。
3. 打开 2FA Web 页面。
4. 在 WebDAV 卡片中填写：
   - `WebDAV Host = webdavBaseUrl`
   - `WebDAV Account = username`
   - `WebDAV Password = password`
   - `Vault Secret = 任意本地加密口令`
5. 先点击“验证配置”，再按需拉取或推送。

当前已验证：

- `challenge -> signature -> token` 可用。
- 使用返回的 Basic Auth 凭据可以通过 WebDAV 成功 `PUT / GET` 文件。
- `MKCOL` 在已存在目录场景下返回 `405`，可被 2FA 当前实现正常接受。

## 构建与测试

```bash
cargo build --workspace
cargo test --workspace
```
