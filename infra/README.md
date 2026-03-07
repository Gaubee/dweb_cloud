# Infra

`dwebCloud v1` 当前只保证最小自托管闭环：本地文件系统 backend + `WebDAV` + `challenge/signature -> token`。

## 运行方式

### 1. 本地 Rust 运行

启动服务：

```bash
cargo run -p dweb-cloud-server -- --http 127.0.0.1:9080 --data-dir ./.data
```

签发 `gaubee-2fa` 使用的 WebDAV 凭据：

```bash
cargo run -p dweb-cloud-cli -- token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

### 2. Docker Compose 运行

启动服务：

```bash
docker compose up -d --build
```

查看日志：

```bash
docker compose logs -f dweb-cloud
```

签发 WebDAV 凭据：

```bash
docker compose exec dweb-cloud dweb-cloud-cli token issue \
  --server http://127.0.0.1:9080 \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

停止服务：

```bash
docker compose down
```

## 默认配置

容器默认环境变量：

- `DWEB_CLOUD_HTTP=0.0.0.0:9080`
- `DWEB_CLOUD_DATA_DIR=/var/lib/dweb-cloud`
- `DWEB_CLOUD_APP_CONFIG=/app/config/apps.json`

数据卷：

- `dweb-cloud-data`：保存 challenge、token、app 私有文件

挂载配置：

- `./config:/app/config:ro`

## 数据布局

默认本地文件结构：

- `challenges/`：待消费 challenge
- `tokens/`：token 记录
- `accounts/<public_key_hex>/apps/<app_id>/`：该 app 的私有文件空间

## 与 2FA 联调

详见：[2fa-webdav.md](./2fa-webdav.md)

## 当前非目标

`v1` 暂不在这里展开：

- TLS 终止
- 反向代理模板
- `S3` backend
- 在线授权回调页
- 多 app 管理后台

这些能力在进入下一阶段之前，仍应先回写 `specs/` 再实现。
