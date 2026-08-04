# `plyx apk` 构建失败分析与修复记录

## 现象

执行 `plyx apk --auto`（Docker 模式）时，`cargo check` 通过，但 Docker 内构建失败：

```
error: invalid inline table
expected `}`
  --> Cargo.toml:21:15
   |
21 | ply-engine = {
   |               ^
```

## 根因

plyx 在构建 APK 前会生成一个覆盖版 `Cargo.toml`（见 plyx 源码
`src/commands/apk.rs` 的 `generate_overlay_cargo_toml`），其中：

- 删除 `[build-dependencies]`
- 注入 `[package.metadata.android]`（默认 `assets = "assets/"`、`build_targets`、`activity_attributes` 等）
- 通过 Docker `-v` 挂载覆盖进容器

本地 `cargo check` 能通过是因为本机 cargo 使用的 toml 解析器（toml_edit）支持
**多行 inline table**（TOML 1.1 特性）：

```toml
ply-engine = {
  version = "1.1",
  features = [ ... ]
}
```

而 Docker 镜像内 `cargo quad-apk` 使用的 toml 解析器（toml v0.8 / 严格 TOML 1.0）
**不允许 inline table 跨多行**，于是在 `ply-engine = {` 这一行直接报
`invalid inline table, expected '}'`。

注意：本仓库的 `Cargo.toml` 是手工/旧版模板生成的，把 `ply-engine` 写成了多行
inline table；而 plyx 官方模板（`src/templates.rs`）生成的是单行 inline table，
所以官方模板不会触发此问题。

## 修复

将 `Cargo.toml` 中的 `ply-engine` 依赖改为单行 inline table：

```toml
ply-engine = { version = "1.1", features = ["audio", "built-in-shaders", "net", "net-json", "storage", "text-styling", "tinyvg"] }
```

## 验证

修复后重新运行 `plyx apk --auto`，构建成功，产物位于：

```
target/android-artifacts/release/apk/demo.apk
```

## 附注（未修复、与本失败无关）

`C:\Users\liuqi\.cargo\config.toml` 存在重复的 `[unstable]` 表（第 10 行与第 35 行），
会导致从 `C:\Users\liuqi` 路径下运行的任意 cargo 命令报
`duplicate key [unstable]`。本仓库位于 `D:\` 下所以不受影响。如需在其他位置
执行 cargo，需合并这两个 `[unstable]` 表。
