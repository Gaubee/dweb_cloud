把 `2FA` 的 self-host 后端独立成 `dwebCloud` 项目，长期为更多产品提供通用服务。
---
`dwebCloud` 使用 Web3 密钥系统作为账号系统。
---
`dwebCloud` 暴露出 `WebDAV` 等接口，同时支持未来在线网盘。
---
`2FA` 的 Provider 变成 `Github-Gist / Google-Drive / Custom-WebDAV`，其中 `dwebCloud` 作为一个 WebDAV 服务提供商。
---
`2FA` 本身保持纯前端、免费化运营，不再依赖自己的后端服务。
---
本轮优先：
1. 在 `~/Dev/GitHub/dweb_cloud` 展开项目，参考 `2FA` 的项目结构
2. 先编写 `specs/AGENTS/ROADMAP`
3. 实现本地化的 `WebDAV`
4. 接入到 `2FA` 验证可行性
5. 简化并剔除 `2FA` 里的 server 部分
