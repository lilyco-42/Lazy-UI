//! Convention-over-configuration component configs.
//!
//! Every component ships with a same-named sidecar stylesheet,
//! `assets/components/<name>.toml` — think of it as the component's CSS. The
//! TOML declares only the fields you want to change; every unset field is
//! `None` and falls back to the M3 theme (or a built-in literal) inside the
//! component. No TOML = optimal M3 defaults.
//!
//! Runtime per-call UI attributes work like a CSS cascade: components rendered
//! inside [`Style::with`] merge the given [`Attrs`] over their stylesheet.

use serde::Deserialize;
use std::cell::RefCell;
use std::sync::OnceLock;

use crate::theme;

// ---------------------------------------------------------------------------
// Size scaling — stylesheets declare logical px; every read multiplies by the
// effective scale (DPR × user zoom) so `set_zoom`/HiDPI take effect on the
// next frame. Colors, counts and ratios (e.g. `bubble_width = 0.72`,
// `speed`, `moon_x_ratio`) are NOT sizes and stay untouched.
// ---------------------------------------------------------------------------

#[inline]
fn px_f(v: Option<f32>, s: f32) -> Option<f32> {
    v.map(|x| x * s)
}

#[inline]
fn px_u(v: Option<u16>, s: f32) -> Option<u16> {
    v.map(|x| (x as f32 * s).round() as u16)
}

/// Stylesheet size fields (logical px) to multiply by the effective scale.
pub trait ScaleSizes {
    fn scale_sizes(&mut self, s: f32);
}

/// Generates a serde config struct (all fields `Option`, so unset = fall
/// back to the theme) plus a lazy loader that reads its same-named sidecar
/// toml and a `merged` CSS-cascade method.
macro_rules! component_config {
    ($name:ident { $($field:ident: $ty:ty),+ $(,)? }, $toml:literal) => {
        #[derive(Debug, Clone, Copy, Default, Deserialize)]
        #[serde(default)]
        pub struct $name {
            $(pub $field: $ty,)+
        }

        impl $name {
            /// Loads `<name>.toml` once; absent fields stay `None` and fall
            /// back to the M3 theme or a built-in literal in the component.
            pub fn get() -> &'static Self {
                static CONFIG: OnceLock<$name> = OnceLock::new();
                CONFIG.get_or_init(|| toml::from_str(include_str!($toml)).unwrap_or_default())
            }

            /// CSS-cascade merge: `self` (higher priority) wins over `base`.
            pub fn merged(self, base: Self) -> Self {
                Self {
                    $($field: self.$field.or(base.$field),)+
                }
            }
        }
    };
}

/// Merge `attrs` (per-call overrides, `None` = not set) over a component's
/// stylesheet `base`. The CSS cascade: attributes > toml > theme. The merged
/// result's size fields are scaled by the effective scale at read time, so
/// runtime overrides, stylesheets and theme fallbacks all share one semantics:
/// authors always write logical px.
pub fn effective<T: Copy + ScaleSizes>(
    attrs: Option<T>,
    base: &T,
    merge: impl FnOnce(T, T) -> T,
) -> T {
    let mut cfg = attrs.map_or(*base, |a| merge(a, *base));
    cfg.scale_sizes(theme::effective_scale());
    cfg
}

// ---------------------------------------------------------------------------
// Containers (container.rs)
// ---------------------------------------------------------------------------

component_config! {
    SidebarConfig {
        width: Option<f32>,
        gap: Option<f32>,
        padding: Option<f32>,
        scroll: Option<bool>,
        background: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/sidebar.toml"
}

impl ScaleSizes for SidebarConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.width = px_f(self.width, s);
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
    }
}

component_config! {
    PanelConfig {
        gap: Option<f32>,
        padding: Option<f32>,
        scroll: Option<bool>,
        background: Option<u32>,
        radius: Option<f32>,
    },
    "../../assets/components/panel.toml"
}

impl ScaleSizes for PanelConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
        self.radius = px_f(self.radius, s);
    }
}

component_config! {
    StatusBarConfig {
        height: Option<f32>,
        gap: Option<f32>,
        padding: Option<f32>,
        background: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/status_bar.toml"
}

impl ScaleSizes for StatusBarConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
    }
}

component_config! {
    LogProgressConfig {
        track_height: Option<f32>,
        gap: Option<f32>,
        padding: Option<f32>,
        track_color: Option<u32>,
        fill_color: Option<u32>,
        radius: Option<f32>,
    },
    "../../assets/components/log_progress.toml"
}

impl ScaleSizes for LogProgressConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.track_height = px_f(self.track_height, s);
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
        self.radius = px_f(self.radius, s);
    }
}

// ---------------------------------------------------------------------------
// Button (button.rs) — one palette per M3 variant, like a CSS class.
// ---------------------------------------------------------------------------

/// A single button variant's palette: every field optional, hex colors.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct ButtonStateConfig {
    pub background: Option<u32>,
    pub hover: Option<u32>,
    pub pressed: Option<u32>,
    pub foreground: Option<u32>,
    pub border: Option<u32>,
}

impl ButtonStateConfig {
    /// CSS-cascade merge: `self` (higher priority) wins over `base`.
    pub fn merged(self, base: Self) -> Self {
        Self {
            background: self.background.or(base.background),
            hover: self.hover.or(base.hover),
            pressed: self.pressed.or(base.pressed),
            foreground: self.foreground.or(base.foreground),
            border: self.border.or(base.border),
        }
    }
}

component_config! {
    ButtonConfig {
        height: Option<f32>,
        font_size: Option<u16>,
        pad_x: Option<f32>,
        radius: Option<f32>,
        filled: Option<ButtonStateConfig>,
        tonal: Option<ButtonStateConfig>,
        outlined: Option<ButtonStateConfig>,
        text: Option<ButtonStateConfig>,
    },
    "../../assets/components/button.toml"
}

impl ScaleSizes for ButtonConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.pad_x = px_f(self.pad_x, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// Form controls
// ---------------------------------------------------------------------------

component_config! {
    CheckboxConfig {
        box_size: Option<f32>,
        radius: Option<f32>,
        gap: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        checked_color: Option<u32>,
        check_color: Option<u32>,
        border_color: Option<u32>,
    },
    "../../assets/components/checkbox.toml"
}

impl ScaleSizes for CheckboxConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.box_size = px_f(self.box_size, s);
        self.radius = px_f(self.radius, s);
        self.gap = px_f(self.gap, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    SwitchConfig {
        width: Option<f32>,
        height: Option<f32>,
        handle_size: Option<f32>,
        gap: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        on_color: Option<u32>,
        on_handle: Option<u32>,
        off_track: Option<u32>,
        off_border: Option<u32>,
        off_handle: Option<u32>,
    },
    "../../assets/components/switch.toml"
}

impl ScaleSizes for SwitchConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.width = px_f(self.width, s);
        self.height = px_f(self.height, s);
        self.handle_size = px_f(self.handle_size, s);
        self.gap = px_f(self.gap, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    RadioConfig {
        size: Option<f32>,
        dot_size: Option<f32>,
        gap: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        selected_color: Option<u32>,
        border_color: Option<u32>,
    },
    "../../assets/components/radio.toml"
}

impl ScaleSizes for RadioConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.size = px_f(self.size, s);
        self.dot_size = px_f(self.dot_size, s);
        self.gap = px_f(self.gap, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    SliderConfig {
        height: Option<f32>,
        track_height: Option<f32>,
        handle_size: Option<f32>,
        radius: Option<f32>,
        gap: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        track_color: Option<u32>,
        fill_color: Option<u32>,
        handle_color: Option<u32>,
        handle_border: Option<u32>,
        label_color: Option<u32>,
    },
    "../../assets/components/slider.toml"
}

impl ScaleSizes for SliderConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.track_height = px_f(self.track_height, s);
        self.handle_size = px_f(self.handle_size, s);
        self.radius = px_f(self.radius, s);
        self.gap = px_f(self.gap, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    TextFieldConfig {
        height: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        placeholder_color: Option<u32>,
        cursor_color: Option<u32>,
        selection_color: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/text_field.toml"
}

impl ScaleSizes for TextFieldConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    TabsConfig {
        height: Option<f32>,
        font_size: Option<u16>,
        pad_x: Option<f32>,
        active_color: Option<u32>,
        inactive_color: Option<u32>,
        indicator_color: Option<u32>,
        indicator_height: Option<f32>,
    },
    "../../assets/components/tabs.toml"
}

impl ScaleSizes for TabsConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.pad_x = px_f(self.pad_x, s);
        self.indicator_height = px_f(self.indicator_height, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// Selection & display
// ---------------------------------------------------------------------------

component_config! {
    ComboConfig {
        height: Option<f32>,
        radius: Option<f32>,
        gap: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        item_height: Option<f32>,
        background: Option<u32>,
        text_color: Option<u32>,
        arrow_color: Option<u32>,
        menu_bg: Option<u32>,
        menu_radius: Option<f32>,
        menu_border: Option<u32>,
        selected_bg: Option<u32>,
        selected_fg: Option<u32>,
    },
    "../../assets/components/combo.toml"
}

impl ScaleSizes for ComboConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.gap = px_f(self.gap, s);
        self.pad_x = px_f(self.pad_x, s);
        self.item_height = px_f(self.item_height, s);
        self.menu_radius = px_f(self.menu_radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    ListboxConfig {
        item_height: Option<f32>,
        radius: Option<f32>,
        border: Option<u32>,
        font_size: Option<u16>,
    },
    "../../assets/components/listbox.toml"
}

impl ScaleSizes for ListboxConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.item_height = px_f(self.item_height, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    SelectableConfig {
        height: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        selected_bg: Option<u32>,
        selected_fg: Option<u32>,
        text_color: Option<u32>,
    },
    "../../assets/components/selectable.toml"
}

impl ScaleSizes for SelectableConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    ProgressConfig {
        track_height: Option<f32>,
        radius: Option<f32>,
        track_color: Option<u32>,
        fill_color: Option<u32>,
    },
    "../../assets/components/progress.toml"
}

impl ScaleSizes for ProgressConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.track_height = px_f(self.track_height, s);
        self.radius = px_f(self.radius, s);
    }
}

component_config! {
    DividerConfig {
        thickness: Option<f32>,
        color: Option<u32>,
    },
    "../../assets/components/divider.toml"
}

impl ScaleSizes for DividerConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.thickness = px_f(self.thickness, s);
    }
}

component_config! {
    TextConfig {
        headline_size: Option<u16>,
        title_size: Option<u16>,
        body_size: Option<u16>,
        label_size: Option<u16>,
        headline_color: Option<u32>,
        title_color: Option<u32>,
        body_color: Option<u32>,
        label_color: Option<u32>,
    },
    "../../assets/components/text.toml"
}

impl ScaleSizes for TextConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.headline_size = px_u(self.headline_size, s);
        self.title_size = px_u(self.title_size, s);
        self.body_size = px_u(self.body_size, s);
        self.label_size = px_u(self.label_size, s);
    }
}

component_config! {
    ChatPanelConfig {
        background: Option<u32>,
        gap: Option<f32>,
        padding: Option<f32>,
        bubble_gap: Option<f32>,
        bubble_font_size: Option<u16>, // 气泡文字字号(移动端需要放大)
        bubble_radius: Option<f32>,
        bubble_width: Option<f32>,
        bubble_pad_x: Option<f32>,
        bubble_pad_y: Option<f32>,
        user_background: Option<u32>,
        user_foreground: Option<u32>,
        pet_background: Option<u32>,
        pet_foreground: Option<u32>,
        quick_gap: Option<f32>,
        quick_columns: Option<u32>, // 每行按钮数(>1 分行, 移动端大按钮用)
        input_gap: Option<f32>,
        max_bubbles: Option<u32>,
    },
    "../../assets/components/chat_panel.toml"
}

impl ScaleSizes for ChatPanelConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
        self.bubble_gap = px_f(self.bubble_gap, s);
        self.bubble_radius = px_f(self.bubble_radius, s);
        self.bubble_pad_x = px_f(self.bubble_pad_x, s);
        self.bubble_pad_y = px_f(self.bubble_pad_y, s);
        self.quick_gap = px_f(self.quick_gap, s);
        self.input_gap = px_f(self.input_gap, s);
        self.bubble_font_size = px_u(self.bubble_font_size, s);
    }
}

component_config! {
    PetBackgroundConfig {
        gradient_top: Option<u32>,
        gradient_mid: Option<u32>,
        gradient_bot: Option<u32>,
        moon_color: Option<u32>,
        moon_glow: Option<u32>,
        moon_x_ratio: Option<f32>,
        moon_y_ratio: Option<f32>,
        moon_radius: Option<f32>,
        moon_glow_radius: Option<f32>,
        cloud_color: Option<u32>,
        cloud_count: Option<u32>,
        cloud_enabled: Option<bool>,
        petal_color: Option<u32>,
        petal_count: Option<u32>,
        petal_enabled: Option<bool>,
        gradient_bands: Option<u32>,
    },
    "../../assets/components/background.toml"
}

impl ScaleSizes for PetBackgroundConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.moon_radius = px_f(self.moon_radius, s);
        self.moon_glow_radius = px_f(self.moon_glow_radius, s);
    }
}

component_config! {
    TooltipConfig {
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        radius: Option<f32>,
        pad_x: Option<f32>,
        offset: Option<f32>,
    },
    "../../assets/components/tooltip.toml"
}

impl ScaleSizes for TooltipConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.offset = px_f(self.offset, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// imgui_kit — Dear ImGui 风格组件集
// ---------------------------------------------------------------------------

component_config! {
    ImWindowConfig {
        background: Option<u32>,
        header_background: Option<u32>,
        title_color: Option<u32>,
        border: Option<u32>,
        radius: Option<f32>,
        gap: Option<f32>,
        padding: Option<f32>,
        header_height: Option<f32>,
    },
    "../../assets/components/im_window.toml"
}

impl ScaleSizes for ImWindowConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
        self.header_height = px_f(self.header_height, s);
    }
}

component_config! {
    CollapsingHeaderConfig {
        height: Option<f32>,
        font_size: Option<u16>,
        gap: Option<f32>,
        pad_x: Option<f32>,
        color: Option<u32>,
        hover: Option<u32>,
        arrow_color: Option<u32>,
        body_gap: Option<f32>,
    },
    "../../assets/components/collapsing_header.toml"
}

impl ScaleSizes for CollapsingHeaderConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.gap = px_f(self.gap, s);
        self.pad_x = px_f(self.pad_x, s);
        self.body_gap = px_f(self.body_gap, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    DragFloatConfig {
        height: Option<f32>,
        radius: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        label_color: Option<u32>,
        value_color: Option<u32>,
        border: Option<u32>,
        speed: Option<f32>,
    },
    "../../assets/components/drag_float.toml"
}

impl ScaleSizes for DragFloatConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    PlotLinesConfig {
        width: Option<f32>,
        height: Option<f32>,
        background: Option<u32>,
        line_color: Option<u32>,
        fill_color: Option<u32>,
        grid_color: Option<u32>,
        border: Option<u32>,
        radius: Option<f32>,
        thickness: Option<f32>,
    },
    "../../assets/components/plot_lines.toml"
}

impl ScaleSizes for PlotLinesConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.width = px_f(self.width, s);
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.thickness = px_f(self.thickness, s);
    }
}

component_config! {
    ImProgressBarConfig {
        track_height: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        track_color: Option<u32>,
        fill_color: Option<u32>,
        text_color: Option<u32>,
    },
    "../../assets/components/im_progress_bar.toml"
}

impl ScaleSizes for ImProgressBarConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.track_height = px_f(self.track_height, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// gpui_kit — GPUI/Zed tailwind 风格组件集
// ---------------------------------------------------------------------------

component_config! {
    DivConfig {
        background: Option<u32>,
        radius: Option<f32>,
        padding: Option<f32>,
        border: Option<u32>,
        border_width: Option<f32>,
        gap: Option<f32>,
    },
    "../../assets/components/div.toml"
}

impl ScaleSizes for DivConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.padding = px_f(self.padding, s);
        self.border_width = px_f(self.border_width, s);
        self.gap = px_f(self.gap, s);
    }
}

component_config! {
    KbdConfig {
        height: Option<f32>,
        pad_x: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/kbd.toml"
}

impl ScaleSizes for KbdConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.pad_x = px_f(self.pad_x, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    ChipConfig {
        height: Option<f32>,
        radius: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        selected_bg: Option<u32>,
        text_color: Option<u32>,
        selected_fg: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/chip.toml"
}

impl ScaleSizes for ChipConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    BadgeConfig {
        radius: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        dot: Option<bool>,
    },
    "../../assets/components/badge.toml"
}

impl ScaleSizes for BadgeConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    AvatarConfig {
        size: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
    },
    "../../assets/components/avatar.toml"
}

impl ScaleSizes for AvatarConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.size = px_f(self.size, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    CodeConfig {
        pad_x: Option<f32>,
        pad_y: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/code.toml"
}

impl ScaleSizes for CodeConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.pad_x = px_f(self.pad_x, s);
        self.pad_y = px_f(self.pad_y, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// eui_neo_kit — EUI-NEO 声明式 DSL 风格组件集
// ---------------------------------------------------------------------------

component_config! {
    SegmentedConfig {
        height: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        pad_x: Option<f32>,
        background: Option<u32>,
        selected_bg: Option<u32>,
        selected_fg: Option<u32>,
        text_color: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/segmented.toml"
}

impl ScaleSizes for SegmentedConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    StepperConfig {
        height: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        button_bg: Option<u32>,
        button_fg: Option<u32>,
        border: Option<u32>,
        step: Option<i32>,
    },
    "../../assets/components/stepper.toml"
}

impl ScaleSizes for StepperConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.height = px_f(self.height, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    CardConfig {
        radius: Option<f32>,
        padding: Option<f32>,
        gap: Option<f32>,
        background: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/card.toml"
}

impl ScaleSizes for CardConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.padding = px_f(self.padding, s);
        self.gap = px_f(self.gap, s);
    }
}

component_config! {
    DialogConfig {
        width: Option<f32>,
        radius: Option<f32>,
        padding: Option<f32>,
        gap: Option<f32>,
        background: Option<u32>,
        title_color: Option<u32>,
        scrim: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/dialog.toml"
}

impl ScaleSizes for DialogConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.width = px_f(self.width, s);
        self.radius = px_f(self.radius, s);
        self.padding = px_f(self.padding, s);
        self.gap = px_f(self.gap, s);
    }
}

component_config! {
    DataTableConfig {
        row_height: Option<f32>,
        header_height: Option<f32>,
        radius: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        header_bg: Option<u32>,
        header_color: Option<u32>,
        row_color: Option<u32>,
        selected_bg: Option<u32>,
        selected_fg: Option<u32>,
        border: Option<u32>,
    },
    "../../assets/components/data_table.toml"
}

impl ScaleSizes for DataTableConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.row_height = px_f(self.row_height, s);
        self.header_height = px_f(self.header_height, s);
        self.radius = px_f(self.radius, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    ToastConfig {
        radius: Option<f32>,
        pad_x: Option<f32>,
        pad_y: Option<f32>,
        font_size: Option<u16>,
        background: Option<u32>,
        text_color: Option<u32>,
        offset_y: Option<f32>,
    },
    "../../assets/components/toast.toml"
}

impl ScaleSizes for ToastConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.pad_y = px_f(self.pad_y, s);
        self.offset_y = px_f(self.offset_y, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// Collected components (from plyx_demo / oh-my-meme prototypes)
// ---------------------------------------------------------------------------

component_config! {
    CopyButtonConfig {
        gap: Option<f32>,
    },
    "../../assets/components/copy_button.toml"
}

impl ScaleSizes for CopyButtonConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.gap = px_f(self.gap, s);
    }
}

component_config! {
    MemeGridConfig {
        cell_size: Option<f32>,
        gap: Option<f32>,
        padding: Option<f32>,
        radius: Option<f32>,
        buffer_rows: Option<usize>,
        thumb_px: Option<u32>,
        cache_capacity: Option<usize>,
        background: Option<u32>,
        placeholder: Option<u32>,
        selected_border: Option<u32>,
    },
    "../../assets/components/meme_grid.toml"
}

impl ScaleSizes for MemeGridConfig {
    fn scale_sizes(&mut self, s: f32) {
        self.cell_size = px_f(self.cell_size, s);
        self.gap = px_f(self.gap, s);
        self.padding = px_f(self.padding, s);
        self.radius = px_f(self.radius, s);
    }
}

component_config! {
    卡片容器Config {
        background: Option<u32>,
        foreground: Option<u32>,
        radius: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
    },
    "../../assets/components/卡片容器.toml"
}

impl ScaleSizes for 卡片容器Config {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.font_size = px_u(self.font_size, s);
    }
}

component_config! {
    蓝色按钮Config {
        background: Option<u32>,
        foreground: Option<u32>,
        radius: Option<f32>,
        pad_x: Option<f32>,
        font_size: Option<u16>,
        center_x: Option<bool>,
        center_x_shift: Option<f32>,
    },
    "../../assets/components/蓝色按钮.toml"
}

impl ScaleSizes for 蓝色按钮Config {
    fn scale_sizes(&mut self, s: f32) {
        self.radius = px_f(self.radius, s);
        self.pad_x = px_f(self.pad_x, s);
        self.center_x_shift = px_f(self.center_x_shift, s);
        self.font_size = px_u(self.font_size, s);
    }
}

// ---------------------------------------------------------------------------
// Per-call UI attributes — Compose-style, CSS-cascade semantics.
//
// ```rust
// let _g = Style::with(Attrs {
//     button: Some(ButtonConfig { height: Some(56.0), radius: Some(28.0), ..Default::default() }),
//     ..Default::default()
// }, || {
//     button(ui, "Save", || save());
// });
// ```
// ---------------------------------------------------------------------------

/// Per-call UI attributes for every component. Each field overrides only the
/// stylesheet fields you set (field-level `Option`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Attrs {
    pub button: Option<ButtonConfig>,
    pub checkbox: Option<CheckboxConfig>,
    pub combo: Option<ComboConfig>,
    pub divider: Option<DividerConfig>,
    pub listbox: Option<ListboxConfig>,
    pub progress: Option<ProgressConfig>,
    pub radio: Option<RadioConfig>,
    pub selectable: Option<SelectableConfig>,
    pub slider: Option<SliderConfig>,
    pub switch: Option<SwitchConfig>,
    pub tabs: Option<TabsConfig>,
    pub text: Option<TextConfig>,
    pub text_field: Option<TextFieldConfig>,
    pub tooltip: Option<TooltipConfig>,
    pub sidebar: Option<SidebarConfig>,
    pub panel: Option<PanelConfig>,
    pub status_bar: Option<StatusBarConfig>,
    pub log_progress: Option<LogProgressConfig>,
    pub chat_panel: Option<ChatPanelConfig>,
    pub im_window: Option<ImWindowConfig>,
    pub collapsing_header: Option<CollapsingHeaderConfig>,
    pub drag_float: Option<DragFloatConfig>,
    pub plot_lines: Option<PlotLinesConfig>,
    pub im_progress_bar: Option<ImProgressBarConfig>,
    pub div: Option<DivConfig>,
    pub kbd: Option<KbdConfig>,
    pub chip: Option<ChipConfig>,
    pub badge: Option<BadgeConfig>,
    pub avatar: Option<AvatarConfig>,
    pub code: Option<CodeConfig>,
    pub segmented: Option<SegmentedConfig>,
    pub stepper: Option<StepperConfig>,
    pub card: Option<CardConfig>,
    pub dialog: Option<DialogConfig>,
    pub data_table: Option<DataTableConfig>,
    pub toast: Option<ToastConfig>,
    pub copy_button: Option<CopyButtonConfig>,
    pub meme_grid: Option<MemeGridConfig>,
    pub 卡片容器: Option<卡片容器Config>,
    pub 蓝色按钮: Option<蓝色按钮Config>,
}

thread_local! {
    /// Stack of active attribute scopes (CSS cascade). The topmost wins.
    static ATTRS: RefCell<Vec<Attrs>> = const { RefCell::new(Vec::new()) };
}

/// Pops the topmost attribute scope on drop (RAII) — safe on early return.
#[must_use]
pub struct StyleGuard;

impl Drop for StyleGuard {
    fn drop(&mut self) {
        ATTRS.with(|s| s.borrow_mut().pop());
    }
}

/// Runtime UI-attribute cascade. Components rendered inside [`Style::with`]
/// merge the given [`Attrs`] over their `<name>.toml` stylesheet.
pub struct Style;

impl Style {
    /// Applies `attrs` to every component rendered by `f`, then pops the scope
    /// when the returned guard is dropped. Use `let _g = Style::with(...)`.
    pub fn with(attrs: Attrs, f: impl FnOnce()) -> StyleGuard {
        ATTRS.with(|s| s.borrow_mut().push(attrs));
        f();
        StyleGuard
    }

    /// The active attribute scope (topmost wins, `Default` when none active).
    pub fn current() -> Attrs {
        ATTRS.with(|s| s.borrow().last().copied().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every component stylesheet must parse strictly — the runtime falls back
    /// to the theme on parse errors, so this test is the only thing that
    /// catches a malformed `<name>.toml`.
    #[test]
    fn all_component_stylesheets_parse() {
        macro_rules! check {
            ($name:ident, $toml:literal) => {
                let raw = include_str!($toml);
                assert!(
                    toml::from_str::<$name>(raw).is_ok(),
                    "{} fails to parse",
                    $toml
                );
            };
        }
        check!(SidebarConfig, "../../assets/components/sidebar.toml");
        check!(PanelConfig, "../../assets/components/panel.toml");
        check!(StatusBarConfig, "../../assets/components/status_bar.toml");
        check!(LogProgressConfig, "../../assets/components/log_progress.toml");
        check!(ButtonConfig, "../../assets/components/button.toml");
        check!(CheckboxConfig, "../../assets/components/checkbox.toml");
        check!(SwitchConfig, "../../assets/components/switch.toml");
        check!(RadioConfig, "../../assets/components/radio.toml");
        check!(SliderConfig, "../../assets/components/slider.toml");
        check!(TextFieldConfig, "../../assets/components/text_field.toml");
        check!(TabsConfig, "../../assets/components/tabs.toml");
        check!(ComboConfig, "../../assets/components/combo.toml");
        check!(ListboxConfig, "../../assets/components/listbox.toml");
        check!(SelectableConfig, "../../assets/components/selectable.toml");
        check!(ProgressConfig, "../../assets/components/progress.toml");
        check!(DividerConfig, "../../assets/components/divider.toml");
        check!(TextConfig, "../../assets/components/text.toml");
        check!(TooltipConfig, "../../assets/components/tooltip.toml");
        check!(ChatPanelConfig, "../../assets/components/chat_panel.toml");
        check!(PetBackgroundConfig, "../../assets/components/background.toml");
        check!(ImWindowConfig, "../../assets/components/im_window.toml");
        check!(CollapsingHeaderConfig, "../../assets/components/collapsing_header.toml");
        check!(DragFloatConfig, "../../assets/components/drag_float.toml");
        check!(PlotLinesConfig, "../../assets/components/plot_lines.toml");
        check!(ImProgressBarConfig, "../../assets/components/im_progress_bar.toml");
        check!(DivConfig, "../../assets/components/div.toml");
        check!(KbdConfig, "../../assets/components/kbd.toml");
        check!(ChipConfig, "../../assets/components/chip.toml");
        check!(BadgeConfig, "../../assets/components/badge.toml");
        check!(AvatarConfig, "../../assets/components/avatar.toml");
        check!(CodeConfig, "../../assets/components/code.toml");
        check!(SegmentedConfig, "../../assets/components/segmented.toml");
        check!(StepperConfig, "../../assets/components/stepper.toml");
        check!(CardConfig, "../../assets/components/card.toml");
        check!(DialogConfig, "../../assets/components/dialog.toml");
        check!(DataTableConfig, "../../assets/components/data_table.toml");
        check!(ToastConfig, "../../assets/components/toast.toml");
        check!(CopyButtonConfig, "../../assets/components/copy_button.toml");
        check!(MemeGridConfig, "../../assets/components/meme_grid.toml");
        check!(卡片容器Config, "../../assets/components/卡片容器.toml");
        check!(蓝色按钮Config, "../../assets/components/蓝色按钮.toml");
    }

    /// The CSS cascade: per-call attrs win over the stylesheet.
    #[test]
    fn attrs_override_stylesheet() {
        let base = ButtonConfig {
            height: Some(40.0),
            filled: Some(ButtonStateConfig { background: Some(0x6750A4), ..Default::default() }),
            ..Default::default()
        };
        let attrs = ButtonConfig {
            height: Some(56.0),
            ..Default::default()
        };
        let merged = attrs.merged(base);
        assert_eq!(merged.height, Some(56.0));
        assert_eq!(merged.filled.and_then(|s| s.background), Some(0x6750A4));
        assert_eq!(merged.pad_x, None);
    }

    /// Style::with pushes/restores the cascade (RAII guard pops on drop).
    #[test]
    fn style_scope_is_raii() {
        assert!(Style::current().button.is_none());
        {
            let _g = Style::with(
                Attrs { button: Some(ButtonConfig::default()), ..Default::default() },
                || assert!(Style::current().button.is_some()),
            );
            assert!(Style::current().button.is_some());
        }
        assert!(Style::current().button.is_none());
    }

    /// `scale_sizes` multiplies logical-px size fields (u16 rounds)…
    #[test]
    fn scale_sizes_multiplies_size_fields() {
        let mut b = ButtonConfig {
            height: Some(40.0),
            font_size: Some(14), // 14 × 1.25 = 17.5 → 18
            radius: Some(20.0),
            ..Default::default()
        };
        b.scale_sizes(1.25);
        assert_eq!(b.height, Some(50.0));
        assert_eq!(b.font_size, Some(18));
        assert_eq!(b.radius, Some(25.0));

        let mut t = TextConfig {
            body_size: Some(16),
            label_size: Some(14),
            ..Default::default()
        };
        t.scale_sizes(2.0);
        assert_eq!(t.body_size, Some(32));
        assert_eq!(t.label_size, Some(28));
    }

    /// …while colors, counts, ratios and sensitivities stay untouched:
    /// `bubble_width` is a width *fraction* (0.72 = 72%), `speed` a drag
    /// multiplier, `thumb_px` a texture budget — none are logical px.
    #[test]
    fn scale_sizes_skips_non_size_fields() {
        let mut c = ChatPanelConfig {
            bubble_width: Some(0.72),
            gap: Some(8.0),
            max_bubbles: Some(3),
            ..Default::default()
        };
        c.scale_sizes(1.25);
        assert_eq!(c.bubble_width, Some(0.72));
        assert_eq!(c.gap, Some(10.0));
        assert_eq!(c.max_bubbles, Some(3));

        let mut d = DragFloatConfig {
            speed: Some(0.5),
            height: Some(24.0),
            ..Default::default()
        };
        d.scale_sizes(1.25);
        assert_eq!(d.speed, Some(0.5));
        assert_eq!(d.height, Some(30.0));

        let mut m = MemeGridConfig {
            cell_size: Some(96.0),
            thumb_px: Some(256),
            buffer_rows: Some(2),
            ..Default::default()
        };
        m.scale_sizes(1.25);
        assert_eq!(m.cell_size, Some(120.0));
        assert_eq!(m.thumb_px, Some(256));
        assert_eq!(m.buffer_rows, Some(2));

        let mut p = PetBackgroundConfig {
            moon_x_ratio: Some(0.8),
            moon_radius: Some(40.0),
            ..Default::default()
        };
        p.scale_sizes(1.25);
        assert_eq!(p.moon_x_ratio, Some(0.8));
        assert_eq!(p.moon_radius, Some(50.0));
    }

    /// `effective` still resolves the cascade (attrs > toml) and applies the
    /// effective scale to the merged result — at test identity scale, values
    /// pass through unchanged.
    #[test]
    fn effective_merges_then_scales() {
        let base = ButtonConfig { height: Some(40.0), ..Default::default() };
        let attrs = ButtonConfig { pad_x: Some(16.0), ..Default::default() };
        let cfg = effective::<ButtonConfig>(Some(attrs), &base, ButtonConfig::merged);
        assert_eq!(cfg.height, Some(40.0));
        assert_eq!(cfg.pad_x, Some(16.0));
    }
}
