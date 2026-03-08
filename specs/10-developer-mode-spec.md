# Developer Mode Spec

## 文档状态

- Status: Active
- Scope: 面向 app builder、集成测试与未来 dweb_chain 基础设施的开发者模式

## 开启方式

状态：`Implemented`

- 启动参数：`--developer-mode`
- 环境变量：`DWEB_CLOUD_DEVELOPER_MODE=true`

## 当前能力

- `GET /api/v1/dev/meta`
- 返回：apps、plans、store stats、management scope
- 配合 CLI：`dweb-cloud-cli developer meta`

## 目标边界

- 用于本地调试、集成测试与未来 `dweb_chain` app onboarding 基础能力
- 默认不应在公网生产环境中开启
- 当前不提供 app 写入注册、动态 capability 下发与多租户开发者控制台
