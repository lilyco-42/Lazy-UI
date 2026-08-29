//! Material 3 component library for Ply, in a Kotlin-Compose-like style.
//!
//! Convention over configuration: components take only data (labels, values, ids);
//! all styling comes from the M3 `Theme` (see [`crate::theme`]).

mod background;
mod button;
mod chat_panel;
mod checkbox;
mod combo;
pub mod config;
mod container;
mod divider;
mod layout;
mod listbox;
mod progress;
mod radio;
mod selectable;
mod slider;
mod switch;
mod tabs;
mod text;
mod text_field;
mod tooltip;
mod zh;

// 外部框架风格移植组件集(均遵循 lazy-ply 约定)
mod imgui_kit;
mod gpui_kit;
mod eui_neo_kit;

pub use background::*;
pub use button::*;
pub use chat_panel::*;
pub use checkbox::*;
pub use combo::*;
pub use container::*;
pub use divider::*;
pub use layout::*;
pub use listbox::*;
pub use progress::*;
pub use radio::*;
pub use selectable::*;
pub use slider::*;
pub use switch::*;
pub use tabs::*;
pub use text::*;
pub use text_field::*;
pub use tooltip::*;
pub use zh::*;
pub use imgui_kit::*;
pub use gpui_kit::*;
pub use eui_neo_kit::*;