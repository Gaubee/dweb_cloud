# Production Reverse Proxy Guide

本文档描述 `dwebCloud` 在公网部署时推荐的反向代理拓扑。

## 推荐拓扑

推荐只把反向代理暴露到公网，而把 `dweb-cloud-server` 绑定到本机回环地址：

- `public internet -> Caddy / Nginx -> dwebCloud(127.0.0.1:9080)`

这样做的目的是：

- 由代理层处理 `HTTPS`
- 避免直接把 `9080` 暴露到公网
- 为后续限流、访问日志、基础防护预留位置

## 方案 1：Caddy

适合：

- 单机快速部署
- 希望自动申请和续期证书

启动 `dwebCloud`：

```bash
docker run -d --name dweb-cloud \
  -p 127.0.0.1:9080:9080 \
  -e DWEB_CLOUD_HTTP=0.0.0.0:9080 \
  -e DWEB_CLOUD_DATA_DIR=/var/lib/dweb-cloud \
  -e DWEB_CLOUD_APP_CONFIG=/app/config/apps.json \
  -v dweb-cloud-data:/var/lib/dweb-cloud \
  gaubee/dweb-cloud:latest
```

复制模板并修改域名：

- 模板文件：[caddy/Caddyfile.example](./caddy/Caddyfile.example)

最小运行示例：

```bash
caddy run --config ./infra/caddy/Caddyfile.example
```

要求：

- 域名已解析到当前服务器
- `80/443` 端口可被 Caddy 访问

## 方案 2：Nginx

适合：

- 已有现成 Nginx 运维体系
- 证书由 Certbot、面板或云服务统一管理

复制模板并修改域名与证书路径：

- 模板文件：[nginx/dweb-cloud.conf.example](./nginx/dweb-cloud.conf.example)

最小运行要求：

- `ssl_certificate` 与 `ssl_certificate_key` 指向有效证书
- `proxy_pass` 指向 `http://127.0.0.1:9080`
- 保留 `proxy_request_buffering off`，避免 WebDAV 请求被不必要缓存

## 2FA 侧配置

当你通过反向代理暴露服务后，2FA 中应填写公网地址，例如：

- `WebDAV Host = https://cloud.example.com/dav/gaubee-2fa`
- `WebDAV Account = <public_key_hex>`
- `WebDAV Password = <app_scoped_token>`
- `Vault Secret = <local secret>`

签发 token 的 CLI 请求地址也应切换到公网域名，例如：

```bash
docker exec -it dweb-cloud dweb-cloud-cli token issue \
  --server https://cloud.example.com \
  --app gaubee-2fa \
  --secret "your secret" \
  --json
```

## 当前边界

本文档只覆盖：

- 单机部署
- 反向代理
- HTTPS

当前仍不覆盖：

- 多节点部署
- 对象存储 backend
- 自动扩缩容
- WAF / DDoS 专项防护
