# Roadmap

`ROADMAP.md` 是本仓库的执行总控视图，负责把“愿景、阶段目标、待验收事项、阻塞项、阶段优先级”集中到一个文件中。

如果内容发生冲突，优先级为：

1. 用户最新明确要求
2. `specs/` 中对应模块 spec
3. `ROADMAP.md`
4. `AGENTS.md`
5. 当前代码实现

## 1. 愿景摘要

`dwebCloud` 的目标不是只服务 `2FA`，而是提供一个可复用的应用存储云内核：

- 使用密钥派生身份，而不是传统账号密码。
- 为每个 app 提供隔离的私有存储空间。
- 对客户端暴露简单稳定的协议入口，当前优先 `WebDAV`。
- 支持本地文件系统、自托管对象存储、未来官方托管。
- 让 `2FA` 等产品保持纯前端化，不再强绑定自有后端。

## 2. 读取顺序

1. `README.md`
2. `ROADMAP.md`
3. `specs/README.md`
4. 对应模块 spec
5. `AGENTS.md`
6. 相关代码与测试

## 3. 状态标记

- `Implemented`: 已有代码落地。
- `In Progress`: 已开始实现但未闭环。
- `Planned`: 已明确进入路线图，但尚未开发。
- `Exploration`: 仍需技术决策或验证样机。
- `Ready for Acceptance`: 已具备阶段性交付物，应进入人工验收。
- `Blocked`: 当前存在环境或外部条件阻塞。

## 4. 阶段总览

| 阶段 | 主题 | 当前状态 | 主要验收门槛 |
| --- | --- | --- | --- |
| Phase 1 | 文档与骨架 | `Implemented` | 文档真源齐备、工作区可构建 |
| Phase 2 | 本地 WebDAV 存储 | `In Progress` | 本地 FS + WebDAV + token 闭环 |
| Phase 3 | 2FA 集成验证 | `Ready for Acceptance` | 2FA 可通过 WebDAV 完成 push/pull |
| Phase 4 | 自托管部署闭环 | `Implemented / Ready for Acceptance` | Docker 镜像发布 + 运行文档 + 2FA 接入手册 |
| Phase 5 | 对象存储与托管化 | `Planned` | S3 backend、在线授权、计费策略 |
| Phase 6 | 通用应用存储云 | `Planned` | 多 app 管理、网盘、协议扩展 |

## 5. 当前优先级

### P0. 文档真源与工作流

状态：`Implemented`

- [x] 建立 `specs/`
- [x] 建立 `ROADMAP.md`
- [x] 建立 `AGENTS.md`
- [x] 建立 `CHAT.md`

### P1. 本地 WebDAV 最小闭环

状态：`In Progress`

- [x] challenge 接口
- [x] token 签发接口
- [x] app 隔离目录模型
- [x] 本地文件系统存储
- [x] WebDAV 基础读写
- [ ] token 撤销命令
- [ ] 更完善的 app 注册配置

### P2. 与 2FA 集成

状态：`Ready for Acceptance`

- [x] 约定 `gaubee-2fa` app
- [x] 2FA 手动 WebDAV 配置
- [x] 旧 Self Provider 移除
- [ ] 双设备 push/pull 验证

### P3. 自托管部署

状态：`Ready for Acceptance`

- [x] `Dockerfile`
- [x] `compose.yml`
- [x] `infra/README.md` 运行说明
- [x] `infra/2fa-webdav.md` 对接手册
- [x] Docker Hub 发布脚本与工作流
- [ ] 反向代理与 TLS 模板

### P4. 后续扩展

状态：`Planned`

- [ ] `S3` backend
- [ ] `Connect dwebCloud` 授权页
- [ ] 在线网盘 UI
- [ ] 更通用的 app/token 管理
