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
/// stylesheet `base`. The CSS cascade: attributes > toml > theme.
pub fn effective<T: Copy>(attrs: Option<T>, base: &T, merge: impl FnOnce(T, T) -> T) -> T {
    attrs.map_or(*base, |a| merge(a, *base))
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

component_config! {
    ListboxConfig {
        item_height: Option<f32>,
        radius: Option<f32>,
        border: Option<u32>,
        font_size: Option<u16>,
    },
    "../../assets/components/listbox.toml"
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

component_config! {
    ProgressConfig {
        track_height: Option<f32>,
        radius: Option<f32>,
        track_color: Option<u32>,
        fill_color: Option<u32>,
    },
    "../../assets/components/progress.toml"
}

component_config! {
    DividerConfig {
        thickness: Option<f32>,
        color: Option<u32>,
    },
    "../../assets/components/divider.toml"
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
}
