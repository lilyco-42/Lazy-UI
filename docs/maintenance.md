# 维护手册 — 分叉地图与归并清单

维护第一入口。lazy-ply 曾在 `D:\Code` 下存在 7 处副本/分叉，本仓库（`D:\Code\lazy-ply`）
是不可争议的唯一事实源。本文记录：各处副本去往何处、哪些实验性改动待归并、发布流程。

## 1. 分叉地图（2026-08 盘点后状态）

| 路径 | git? | 版本基线 | 角色 | 当前处置 |
|------|------|----------|------|----------|
| **本仓库** `D:\Code\lazy-ply` | ✅ → GitHub main | 最新 | **唯一事实源** | 所有开发/提交/push 都在这里 |
| `D:\Code\test\lazy-ply` | ✅(整目录已归档) | 曾停 f5b3a74 | test/demo 的 path 依赖 | 已归档，替换为 **junction** → 本仓库 |
| `D:\Code\cute_box\lazy-ply` | ❌ 副本 | 5e0da19 时代+kits | 桌宠组件源；cute_box/pet 的 path 依赖 | 已归档，替换为 **junction** → 本仓库 |
| `D:\Code\rust\helo\lazy-ply` | ✅ 别处克隆 | f2b9b80+本地 analyzer | 二进制分析实验（capstone/goblin） | **未动** — 待处理（见 §3） |
| `D:\Code\rust\plyx_demo\demo` | ✅ 别处克隆 | f2b9b80+大量未提交实验 | M3 系列实验（themes/dsl/中文组件…） | **未动** — 待处理（见 §3） |
| `D:\Code\rust\oh-my-meme-rs` | ✅ 独立 fork | 独立进化 | 表情包网格应用（chat_panel+meme_grid） | **独立项目**，保留 |
| `D:\Code\test\demo` | ✅ | 依赖 test/lazy-ply | 引擎测试台 | 通过 junction 直连本仓库 |

## 2. 已归并进本仓库的资产（回收的本地改进）

| 来源 | 内容 | 落点 |
|------|------|------|
| test\lazy-ply 本地改动 | button 平滑高亮（thread_local BTN_HL + lerp_srgb 十字渐变）、`ButtonKind`/`variant_palette`/`variant_state`/`button_id_kind`、无障碍 focus 环 | `src/components/button.rs` |
| test\lazy-ply 本地改动 | `按钮()` 改走 `ButtonKind::Filled`（实心强调） | `src/components/zh.rs` |
| cute_box\lazy-ply | 全组件 showcase `demo_components.rs`（523 行、19 个交互状态） | `src/bin/demo_components.rs` |
| cute_box\lazy-ply | 主题组件 toml 批量（avatar/badge/card/chat_panel/chip/kbd…） | `assets/components/`（cute_box 与 GitHub 仅行尾差异，已天然一致） |
| plyx_demo\demo | **copy_button**（剪贴板按钮，固定宽度防跳变） | `src/components/copy_button.rs` + `copy_button.toml` |
| plyx_demo\demo | **log_panel**（有状态日志面板，Component trait 演示） | `src/components/log_panel.rs`（依赖新增 `panel_opt`） |
| plyx_demo\demo | **蓝色按钮**（M3 蓝实心按钮，center_x_shift 校准） | `src/components/蓝色按钮.rs` + `蓝色按钮.toml` |
| plyx_demo\demo | **卡片容器**（M3 卡片容器） | `src/components/卡片容器.rs` + `卡片容器.toml` |
| plyx_demo\demo | **主页面 / 关于页**（page_layout 页面组件 + 布局关系 toml） | `src/components/主页面.rs` `关于页.rs` + 布局 toml；bin `demo_pages` |
| plyx_demo\demo | **11 套主题**（catppuccin/dracula/fluent/gruvbox/nord/one_dark/solarized×2/tokyo/m3_dark/m3_light） | `assets/themes/*.toml`（新增目录；当前激活主题仍为 `theme.toml`） |
| oh-my-meme-rs | **meme_grid**（虚拟化表情网格，image 解码 + LRU 纹理缓存） | `src/components/meme_grid.rs` + `meme_grid.toml`（feature `image-grid` 启用） |

## 3. 待处理清单（需要决策）

### 3.1 `rust/helo/lazy-ply` —— analyzer 二进分析实验
本地未提交：`src/bin/analyzer.rs` + `Cargo.toml`(+capstone/goblin/anyhow) + `sample.bin`。
- 方案 A：把 `analyzer.rs` 收编为本仓库 `src/bin/analyzer.rs`，capstone/goblin 加为可选 feature。
- 方案 B：作为独立工具仓库另立（`lilyco-42/rust-bit-analyzer`）。
- 方案 C：丢弃。

### 3.2 `rust/plyx_demo/demo` —— M3 系列实验（全部未提交）
独有内容：
- `assets/themes/` 11 套主题 toml（catppuccin/dracula/fluent_light/gruvbox/nord/one_dark/solarized×2/tokyo_night/m3_dark/…）
- `src/dsl.rs`、`src/config.rs`、`src/bin/{doc_gen,lyco,showcase}.rs`
- `src/components/` 中文命名组件（主页画面.rs / 关于页.rs / 卡片容器.rs / 蓝色按钮.rs）、`copy_button.rs`、`log_panel.rs`
- `docs/components.md`、`docs/ply-engine-pitfalls.md`、`docs/lyco.md`、`CLAUDE.md`
- asset 侧 catalog.toml、主题化改造（theme.toml → themes/ 目录化）
- **注意**：其中 component.rs + page_layout + slider 重写已在 f5b3a74 归并过；以上是剩余未归并部分。
- 方案 A：逐项评审归并进本仓库（主题目录化是架构级改进，建议优先）。
- 方案 B：先在其本地 repo 里 commit 成分支保留，再逐步归并。
- 方案 C：丢弃。

### 3.3 `rust/oh-my-meme-rs` —— meme_grid 网格组件
GIF/图片虚拟化网格，依赖 `image` + `indexmap`。
- 方案 A：抽成本仓库组件（放 `examples/` 或可选 feature），保留 demo。
- 方案 B：作为独立仓库应用继续存在，需要时复制回来。

## 4. 消费者依赖关系（改公共 API 前先看这里）

| 消费者 | 依赖写法 | 连接方式 |
|--------|----------|----------|
| `D:\Code\test\demo` | `lazy-ply = { path = "../lazy-ply" }` | junction `test\lazy-ply` → 本仓库 |
| `D:\Code\cute_box\pet` | `lazy-ply = { path = "../lazy-ply" }`（原名 `demo`，已迁移） | junction `cute_box\lazy-ply` → 本仓库 |

> Junction 创建：`cmd /c mklink /J <目标目录> <本仓库>`。被替换的实体副本已移到
> `D:\Code\_wip_archive\<日期>\` 下备用（git 历史在各自的 `.git` 里，需要时直接拷回）。

## 5. 发布流程（后续可选：crates.io）

1. 版本推进：`Cargo.toml` version 0.1.0 → 0.2.0（public API 已稳定：lib + 中文即时模式 + 3 kit）。
2. `cargo package` 双检仓库内构建：`cargo build --release` + `cargo run --bin demo_components`。
3. 发布前核对 §3 待处理清单，决定哪些在这个版本收编。
4. 打 tag：`git tag v0.2.0 && git push origin v0.2.0`。

## 6. 例行维护节奏（OODA）

- **Observe**：有心提交就 `git log origin/main..HEAD`；看 §3 有没有新实验可收编。
- **Orient**：改动是否影响 §4 两个消费者 → 先 cargo check。
- **Decide/Act**：小步提交、及时 push；showcase 验证视觉。
- **Re-observe**：`cargo run --bin demo_components` 过一遍全部组件再合入。