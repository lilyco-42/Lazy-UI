//! Convention-over-configuration component configs.
//!
//! Every component ships with an optimal built-in default. An optional
//! sidecar `assets/components/<name>.toml` overrides only the fields you set —
//! everything else falls back to the M3 default. No toml = optimal defaults.

use serde::Deserialize;
use std::sync::OnceLock;

/// Generates a serde config struct with built-in defaults + a lazy loader
/// that reads its sidecar toml (`include_str!`) and falls back on parse errors.
macro_rules! component_config {
    ($name:ident { $($field:ident: $ty:ty = $default:expr),+ $(,)? }, $toml:literal) => {
        #[derive(Debug, Clone, Copy, Deserialize)]
        #[serde(default)]
        pub struct $name {
            $(pub $field: $ty,)+
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $($field: $default,)+ }
            }
        }

        impl $name {
            /// Loads `<name>.toml` once; falls back to built-in optimal defaults.
            pub fn get() -> &'static Self {
                static CONFIG: OnceLock<$name> = OnceLock::new();
                CONFIG.get_or_init(|| toml::from_str(include_str!($toml)).unwrap_or_default())
            }
        }
    };
}

component_config! {
    SidebarConfig {
        width: f32 = 240.0,
        gap: f32 = 4.0,
        padding: f32 = 12.0,
        scroll: bool = true,
    },
    "../../assets/components/sidebar.toml"
}

component_config! {
    PanelConfig {
        gap: f32 = 8.0,
        padding: f32 = 16.0,
        scroll: bool = true,
    },
    "../../assets/components/panel.toml"
}

component_config! {
    StatusBarConfig {
        height: f32 = 32.0,
        gap: f32 = 8.0,
        padding: f32 = 12.0,
    },
    "../../assets/components/status_bar.toml"
}

component_config! {
    LogProgressConfig {
        track_height: f32 = 6.0,
        gap: f32 = 4.0,
        padding: f32 = 8.0,
    },
    "../../assets/components/log_progress.toml"
}
