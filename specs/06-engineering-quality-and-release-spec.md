# Engineering Quality And Release Spec

## 文档状态

- Status: Active
- Scope: 工程结构、测试、发布、文档对齐要求

## 当前工程原则

- Rust workspace 优先
- 文档先行
- 后端优先复用成熟协议与现成库
- 改动 `dwebCloud` 的产品边界时必须回写 specs

## 测试要求

- `cargo test --workspace`
- 与 `2FA` 的手动联调验证
