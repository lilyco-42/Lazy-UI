# lazy-ply — Agent 引导

Rust GUI 组件框架：ply-engine 的「约定大于配置」上层封装。
**本仓库是唯一事实源**（GitHub: `lilyco-42/lazy-ply`）。改这里 → commit → push → 其它项目经 junction 自动同步。

## 仓库结构

```
src/
├── lib.rs                 # 库入口（with_ui 帧内上下文、公共导出）
├── immediate.rs           # 中文即时模式核心（current_ui）
├── theme.rs               # M3 主题 + 视觉常量（HOVER_PRIMARY 等）
├── fonts.rs               # 字体加载（嵌入式，无 fontconfig）
├── main.rs                # 默认 binary 入口
└── components/
    ├── zh.rs              # 中文即时模式组件（按钮/复选框/滑块/…）
    ├── button.rs          # 含 ButtonKind/variant_palette/平滑高亮
    ├── chat_panel.rs      # 聊天气泡 + 快捷问题 + 输入框
    ├── background.rs      # 和风直绘背景（pet_background）
    ├── component.rs       # Component trait + ComponentTree（有状态组件）
    ├── page_layout.rs     # flex 布局原语（按 toml 摆放子组件）
    ├── imgui_kit.rs      # 移植 imgui 风格（im_window/collapsing_header/drag_float…）
    ├── gpui_kit.rs        # 移植 gpui 风格（div/kbd/chip/badge/avatar/code）
    ├── eui_neo_kit.rs     # 移植 eui-neo 风格（segmented/stepper/card/dialog/data_table/toast）
    └── …                  # 基础组件：button/checkbox/combo/listbox/slider/tabs…
src/bin/
├── demo_menu.rs           # 菜单 demo
├── demo_todo_list.rs      # 待办清单 demo
└── demo_components.rs     # 全组件 showcase（cargo run --bin demo_components）
assets/
├── components/*.toml      # 每个组件的主题 toml（颜色/缩放）
├── fonts/                 # lexend + LXGWWenKai（嵌入）
├── i18n/{zh-CN,en}.toml   # rust-i18n 文案目录
├── app_layout.toml        # 布局推断
└── theme.toml             # 全局主题
docs/
├── journal/               # 历史会话/提示词（归档，不参与构建）
├── maintenance.md         # ★ 分叉地图 + 归并清单（维护第一入口）
└── android-*/apk-*        # 平台构建备忘
tools/cargo-docx/          # 文档小工具
.github/workflows/build.yml # CI（triage/test/wasm/apk）
```

## 常用命令

```bash
cargo build                                # 编译
cargo run --bin demo_components            # 全组件 showcase
cargo run --bin demo_menu                  # 菜单 demo
cargo check --all-targets                  # 快速全量检查
cargo test --lib                           # headless 单元测试（component.rs 内嵌测试）
```

## 铁律

1. **组件 = rs + toml**：`xxx.rs` 管逻辑/响应，`xxx.toml` 管颜色/缩放；只接收数据，不写死样式。
2. **新增组件流程**：建 `src/components/xxx.rs` + `assets/components/xxx.toml` → `mod.rs` 注册并 `pub use` → 中文即时模式在 `zh.rs` 加同名包装 → Showcase 加一区 → 本地 `cargo run --bin demo_components` 验证。
3. **三套风格 kit 改造**：改 `imgui/gpui/eui_neo_kit.rs` 时保持「接收数据、返回事件/新值」约定，禁止引入对应框架的运行时依赖。
4. **i18n**：文案进 `assets/i18n/*.toml`，用 rust-i18n 宏；组件名/zh.rs API 除外（本来就是中文）。
5. **API 破坏性变更**：下游消费方（见 `docs/maintenance.md`）用 path 依赖 + junction 直连本仓库，签名一变它们全崩。改公共 API 前先 `cargo check` 并更新 showcase。
6. **跨平台**：fontdb 已关 fontconfig（HarmonyOS/ohos 无 -lX11/pango）；新依赖不得引入要系统库的 crate。
7. **提交规范**：`feat(组件): 一句话` 或 `fix(组件): 一句话`；改动必须可复现（demo 可跑）。

## 会话记录存档

`docs/journal/` 下的 `.md` 是历史提示词/会话转储，**不具备参考价值就删**，别让它们回根目录。