//! Material 3 component library for Ply, in a Kotlin-Compose-like style.
//!
//! Convention over configuration: components take only data (labels, values, ids);
//! all styling comes from the M3 `Theme` (see [`crate::theme`]).

mod background;
mod button;
mod component;
mod page_layout;
mod chat_panel;
mod checkbox;
mod combo;
pub mod config;
mod container;
mod copy_button;
mod divider;
mod layout;
mod listbox;
mod log_panel;
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
#[path = "蓝色按钮.rs"] pub mod 蓝色按钮;
#[path = "卡片容器.rs"] pub mod 卡片容器;
#[path = "主页面.rs"] pub mod 主页面;
#[path = "关于页.rs"] pub mod 关于页;

// 外部框架风格移植组件集(均遵循 lazy-ply 约定)
mod imgui_kit;
mod gpui_kit;
mod eui_neo_kit;

pub use background::*;
pub use button::*;
pub use component::*;
pub use page_layout::*;
pub use chat_panel::*;
pub use checkbox::*;
pub use combo::*;
pub use container::*;
pub use copy_button::*;
pub use divider::*;
pub use layout::*;
pub use listbox::*;
pub use log_panel::*;
pub use progress::*;
pub use radio::*;
pub use selectable::*;
pub use slider::*;
pub use switch::*;
pub use tabs::*;
pub use text::*;
pub use text_field::*;
pub use tooltip::*;
pub use 蓝色按钮::*;
pub use 卡片容器::*;
pub use zh::*;
pub use imgui_kit::*;
pub use gpui_kit::*;
pub use eui_neo_kit::*;