# Storage And WebDAV Spec

## 文档状态

- Status: Active
- Scope: backend 存储布局、WebDAV 暴露方式

## v1 backend

状态：`Implemented`

- backend 只实现本地文件系统
- 目录结构按 `account/app` 隔离

## WebDAV

状态：`Implemented / In Progress`

- 路径：`/dav/:app_id`
- 用户通过 Basic Auth 访问
- token 必须和 `app_id` 强绑定
- 仅允许访问该 app 的私有空间

## 后续规划

状态：`Planned`

- `S3` backend
- 更多协议适配层
