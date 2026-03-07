# Deployment And Operations Spec

## 文档状态

- Status: Active
- Scope: `dwebCloud v1` 的自托管部署、运行与运维边界

## v1 部署目标

状态：`Implemented / Ready for Acceptance`

- 提供本地 Rust 运行方式
- 提供 `Dockerfile + compose.yml` 的最小自托管闭环
- 提供 Docker Hub 镜像命名与发布链路
- 提供 2FA 对接手册
- 让没有 Rust 环境的部署者也能跑起服务

## v1 运行约束

- backend 仍只实现本地文件系统
- 数据目录必须可持久化
- app 配置通过 `config/apps.json` 挂载
- 提供 Caddy 与 Nginx 的最小反向代理模板
- 监控、告警与更复杂的网关编排仍不在 v1 范围内
- Docker Hub 镜像以 `gaubee/dweb-cloud` 作为默认命名

## 运维最小要求

- 部署者需要持久化 `data_dir`
- 生产环境推荐仅暴露反向代理，不直接暴露 `9080`
- 升级前应先备份 `config/` 与数据目录
- token 丢失不会导致明文泄漏，但会影响客户端继续写入

## 当前非目标

- 多节点同步
- 自动计费
- 在线控制台
- 完整 IAM
