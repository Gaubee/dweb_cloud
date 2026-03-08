# Infra

`dwebCloud v1/v1.5` 当前提供可实际运行的自托管闭环：本地文件系统 backend + `WebDAV` + `challenge/signature -> token` + Docker / 反向代理 / smoke 验证。

## 运行方式

### 1. 本地 Rust 运行

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

### 2. Docker Hub 镜像运行

拉取镜像：

```bash
docker pull gaubee/dweb-cloud:latest
```

启动服务：

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

生产环境更推荐把服务绑定到本机回环地址，再由反向代理暴露公网入口。详见：[production-deploy.md](./production-deploy.md)。

### 3. Docker Compose 运行

启动服务：

```bash
docker compose up -d --build
```

查看日志：

```bash
docker compose logs -f dweb-cloud
```

### 4. 生产编排示例

`dweb-cloud + caddy` 示例：

```bash
docker compose -f infra/compose.caddy.yml.example up -d
```

使用前需要修改：

- `infra/caddy/Caddyfile.compose.example` 中的域名
- `config/apps.json`
- `config/plans.json`

## 用户与开发者 CLI

签发 WebDAV 凭据：

```bash
cargo run -q -p dweb-cloud-cli -- token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

查看 plans：

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

查看 developer mode 元信息：

```bash
cargo run -q -p dweb-cloud-cli -- developer meta --server http://127.0.0.1:9080 --json
```

查看 operator 本地统计：

```bash
cargo run -q -p dweb-cloud-cli -- admin stats --data-dir ./.data --json
```

## 镜像发布

本地发布：

```bash
./scripts/publish-docker.sh
./scripts/publish-docker.sh 0.1.0
```

GitHub Actions 发布：

- 推送 tag，例如 `v0.1.0`
- 或手动触发 `.github/workflows/publish-docker.yml`
- 需要仓库 secrets：`DOCKERHUB_USERNAME` 与 `DOCKERHUB_TOKEN`

## 默认配置

容器默认环境变量：

- `DWEB_CLOUD_HTTP=0.0.0.0:9080`
- `DWEB_CLOUD_DATA_DIR=/var/lib/dweb-cloud`
- `DWEB_CLOUD_APP_CONFIG=/app/config/apps.json`
- `DWEB_CLOUD_PLAN_CONFIG=/app/config/plans.json`
- `DWEB_CLOUD_DEVELOPER_MODE=false`

数据卷：

- `dweb-cloud-data`：保存 challenge、token、app 私有文件

挂载配置：

- `./config:/app/config:ro`

## 数据布局

默认本地文件结构：

- `challenges/`：待消费 challenge
- `tokens/`：token 记录
- `accounts/<public_key_hex>/apps/<app_id>/`：该 app 的私有文件空间

## 公网部署

已提供：

- [production-deploy.md](./production-deploy.md)
- [caddy/Caddyfile.example](./caddy/Caddyfile.example)
- [caddy/Caddyfile.compose.example](./caddy/Caddyfile.compose.example)
- [nginx/dweb-cloud.conf.example](./nginx/dweb-cloud.conf.example)
- [compose.caddy.yml.example](./compose.caddy.yml.example)

## 与 2FA 联调

详见：[2fa-webdav.md](./2fa-webdav.md)

公网 smoke：

```bash
./scripts/smoke-public-webdav.sh https://cloud.example.com "your secret"
```

## 当前非目标

`v1/v1.5` 暂不在这里展开：

- `S3` backend
- 在线授权回调页
- 多 app 自助 onboarding
- operator Web 管理台
- 多节点编排与高可用

这些能力在进入下一阶段之前，仍应先回写 `specs/` 再实现。
