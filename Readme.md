# 如你所见 这是一个基于[plyx](https://github.com/TheRedDeveloper/ply-engine/)的简单上层封装
我们的目的是实现这样的ui书写

```
 侧边栏({
        启动()
        设置()
        关于()
        作者()
    })
    日志面板()
    日志进度条(默认 = nvim dialog 样式)
```
- 思路: 约定大于配置.我们约定 
 每一个组件 Button.rs 对应一个Button.toml 
    - *.rs文件
        负责数据处理和响应操作 
    - *.toml
        实现颜色,ui缩放等不影响原有功能的实现
- 为了易用性,我们确保
    1.全局config.toml 必须实现跨平台ui渲染一致,必须要要有缩放按钮.
    2.每个组件开发者应提供良好的布局 
    - 比如开发者1 开发Button.rs 和 Button.toml,Button.toml 必须是该作者默认设置的最优显示,在toml内标注适用的系统分辨率等.
-

---

## 中文(汉化)组件 — 即时模式

原版组件(如 `button`)全部保留;额外提供中文名的「即时模式」包装:
调用时无需显式传 `ui`,配合 [`with_ui`] 填好的帧内上下文即可书写零样板 UI。

```rust
with_ui(&mut ui, || {
    if 按钮("点我") { println!("Hi"); }
    自动保存 = 复选框("auto", 自动保存, "自动保存");
    音量 = 滑块("vol", "音量", 音量, 0.0, 100.0);
});
```

即时模式组件见 `src/components/zh.rs`,命名与参数与原版一一对应:

| 原版 | 中文名 | 返回 | 原版 | 中文名 | 返回 |
|---|---|---|---|---|---|
| `button_id` | `按钮` | 是否按下 | `listbox` | `列表框` | 新选中下标 |
| `headline / title / body / label` | `大标题 / 标题 / 正文 / 标签` | — | `combo` | `下拉框` | 新选中下标 |
| `checkbox` | `复选框` | 切换后状态 | `tabs` | `选项卡` | 新选中下标 |
| `switch` | `开关` | 切换后状态 | `progress` | `进度条` | — |
| `radio` | `单选` | 是否选中 | `divider` | `分割线` | — |
| `radio_group` | `单选组` | 新选中下标 | `text_field` | `输入框` | — |
| `selectable` | `列表项` | 是否激活 | `text_field_outlined` | `描边输入框` | — |
| `slider` | `滑块` | 拖动后值 | `tooltip` | `提示` | — |
| `sidebar / panel / status_bar` | `侧边栏 / 面板 / 状态栏` | — | `log_progress` | `日志进度条` | — |
| `chat_panel` | `聊天面板` | 事件 | | | |

## 移植组件集 — 中文即时模式

从桌宠项目(`cute_box/lazy-ply`)移植的三种外部框架风格组件集,外加一个直绘背景;
同样提供中文即时模式包装(原版英文名保留):

| 集合 | 中文名 | 原版 | 返回 |
|---|---|---|---|
| imgui_kit | `浮动窗口` | `im_window` | — |
| imgui_kit | `折叠标题` | `collapsing_header` | 是否展开 |
| imgui_kit | `拖拽数值` | `drag_float` | 新值 |
| imgui_kit | `迷你折线图` | `plot_lines` | — |
| imgui_kit | `复古进度条` | `im_progress_bar` | — |
| imgui_kit | `项目符号` | `bullet_text` | — |
| gpui_kit | `容器` | `div` | — |
| gpui_kit | `键盘键` | `kbd` | — |
| gpui_kit | `过滤芯片` | `chip` | 是否点击 |
| gpui_kit | `徽标` | `badge` | — |
| gpui_kit | `头像` | `avatar` | — |
| gpui_kit | `行内代码` | `code` | — |
| eui_neo_kit | `分段选择` | `segmented` | 新选中下标 |
| eui_neo_kit | `步进器` | `stepper` | 新值 |
| eui_neo_kit | `卡片` | `card` | — |
| eui_neo_kit | `对话框` | `dialog` | 是否仍打开 |
| eui_neo_kit | `数据表格` | `data_table` | 新选中行 |
| eui_neo_kit | `通知` | `toast` | 是否仍显示 |
| background | `和风背景` | `pet_background` | —(直绘,不占 UI 流) |

`chat_panel` 从桌宠版同步了 `llm_hint`(输入框下方提示行)、气泡 `wrap_mode` 与
默认快捷问题列表;`Cargo.toml` 同步禁用了 fontconfig(兼容 HarmonyOS 交叉链接)。
