# dwebCloud

仓库地址：<https://github.com/Gaubee/dweb_cloud>
关联客户端：<https://github.com/Gaubee/2fa>
Docker 镜像：<https://hub.docker.com/r/gaubee/dweb-cloud>

基于 `Rust` 的应用存储云内核，提供 `Web3` 风格密钥身份、应用隔离的存储空间，以及面向客户端应用的 `WebDAV` 接入能力。

当前仓库聚焦 `v1/v1.5`：为 [`gaubee-2fa`](https://github.com/Gaubee/2fa) 提供本地文件系统版 `WebDAV + 签名换 token` 最小闭环，并开始补齐用户自助、开发者模式与商业 plans 模型。它也将作为未来 `dweb_chain` 商业公链项目的基础设施之一。

## 文档

- [ROADMAP.md](./ROADMAP.md)：愿景、阶段目标、待验收事项与当前优先级总控。
- [specs/README.md](./specs/README.md)：产品与模块规格真源。
- [AGENTS.md](./AGENTS.md)：开发元规则、最佳实践与标准工作流。
- [CHAT.md](./CHAT.md)：来自 2FA 仓库拆分的原始需求轨迹。
- [infra/README.md](./infra/README.md)：本地运行、Docker 部署、反向代理与运维说明。
- [infra/2fa-webdav.md](./infra/2fa-webdav.md)：把 dwebCloud 接入 2FA 的快速手册。
- [infra/production-deploy.md](./infra/production-deploy.md)：Caddy / Nginx 反向代理与 HTTPS 部署模板。
- [.github/workflows/publish-docker.yml](./.github/workflows/publish-docker.yml)：基于 tag 或手动触发的 Docker 发布工作流。

## 目录

- `server/`：Rust HTTP/WebDAV 服务。
- `cli/`：Rust CLI，用于签发 token、账户自助管理、开发者与运维查询。
- `crates/identity-core/`：助记词派生、公钥身份、挑战签名与验签。
- `crates/storage-core/`：本地文件系统存储、challenge/token 持久化、plan 配置、统计与账户视图。
- `config/`：默认 `apps` 与 `plans` 配置。
- `infra/`：部署、反向代理、生产编排示例。
- `scripts/`：Docker 发布与公网 smoke 脚本。
- `specs/`：产品与工程规格。

## 当前状态

已完成：

- 工作区骨架与规格文档真源。
- 本地文件系统后端。
- challenge + signature + token 签发接口。
- app-scoped WebDAV 本地服务。
- `gaubee-2fa` 的默认 app 注册。
- Docker 自托管最小闭环。
- Docker Hub 发布脚本与 GitHub Actions 工作流。
- 2FA WebDAV 联调文档。
- Caddy / Nginx 公网反向代理模板。
- 公开 `apps / plans` 接口。
- 账户自助 `overview / token list / token revoke`。
- `developer mode` 元信息接口。
- 公网 `WebDAV` smoke 脚本。

后续推进：

- `S3` backend。
- Web 控制台与 operator 控制台。
- entitlement / payment / hosted quota。
- 面向 `dweb_chain` 生态的更通用 app onboarding 能力。

## 快速开始

### 本地 Rust 运行

启动服务：

```bash
cargo run -p dweb-cloud-server -- --http 127.0.0.1:9080 --data-dir ./.data
```

开发者模式启动：

```bash
cargo run -p dweb-cloud-server -- \
  --http 127.0.0.1:9080 \
  --data-dir ./.data \
  --developer-mode
```

签发 2FA 使用的 WebDAV 凭据：

```bash
cargo run -p dweb-cloud-cli -- token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

### Docker Hub 直接运行

拉取镜像：

```bash
docker pull gaubee/dweb-cloud:latest
```

直接运行：

```bash
docker run -d --name dweb-cloud \
  -p 9080:9080 \
  -e DWEB_CLOUD_HTTP=0.0.0.0:9080 \
  -e DWEB_CLOUD_DATA_DIR=/var/lib/dweb-cloud \
  -e DWEB_CLOUD_APP_CONFIG=/app/config/apps.json \
  -e DWEB_CLOUD_PLAN_CONFIG=/app/config/plans.json \
  -v dweb-cloud-data:/var/lib/dweb-cloud \
  gaubee/dweb-cloud:latest
```

### Docker Compose 运行

```bash
docker compose up -d --build
```

生产环境的 `dweb-cloud + caddy` 编排示例：

```bash
docker compose -f infra/compose.caddy.yml.example up -d
```

更多细节见：[infra/README.md](./infra/README.md)

## 公开接口与 CLI

查看 app 注册信息：

```bash
cargo run -q -p dweb-cloud-cli -- public apps --server http://127.0.0.1:9080 --json
```

查看商业 plans：

```bash
cargo run -q -p dweb-cloud-cli -- public plans --server http://127.0.0.1:9080 --json
```

查看账户概览：

```bash
cargo run -q -p dweb-cloud-cli -- account overview \
  --server http://127.0.0.1:9080 \
  --secret "your secret" \
  --json
```

列出账户 token：

```bash
cargo run -q -p dweb-cloud-cli -- account tokens list \
  --server http://127.0.0.1:9080 \
  --secret "your secret" \
  --json
```

撤销 token：

```bash
cargo run -q -p dweb-cloud-cli -- account tokens revoke \
  --server http://127.0.0.1:9080 \
  --secret "your secret" \
  --token-id <token-id> \
  --json
```

查看 developer mode 元信息：

```bash
cargo run -q -p dweb-cloud-cli -- developer meta --server http://127.0.0.1:9080 --json
```

查看 operator 本地统计：

```bash
cargo run -q -p dweb-cloud-cli -- admin stats --data-dir ./.data --json
```

## 与 2FA 联调

当前最小联调流程：

1. 启动 `dweb-cloud-server` 或 `docker compose up -d --build`。
2. 用 `dweb-cloud-cli token issue` 获取 WebDAV 凭据。
3. 打开 2FA Web 页面。
4. 在 WebDAV 卡片中填写：
   - `WebDAV Host = webdavBaseUrl`
   - `WebDAV Account = username`
   - `WebDAV Password = password`
   - `Vault Secret = 任意本地加密口令`
5. 先点击“验证配置”，再按需拉取或推送。

详细步骤见：[infra/2fa-webdav.md](./infra/2fa-webdav.md)

公网域名 smoke：

```bash
./scripts/smoke-public-webdav.sh https://cloud.example.com "your secret"
```

## 构建与测试

```bash
cargo build --workspace
cargo test --workspace
```

有 Docker 环境时，还应验证：

```bash
docker compose config
docker build -t gaubee/dweb-cloud:local .
```

发布镜像：

```bash
./scripts/publish-docker.sh
./scripts/publish-docker.sh 0.1.0
```
