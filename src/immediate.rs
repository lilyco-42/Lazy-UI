//! Frame-scoped current [`Ui`] context — the bridge that lets components render
//! without threading `ui` through every call, immediate-mode style.
//!
//! [`with_ui`] installs the current frame's [`Ui`] for the duration of a
//! closure; components then fetch it with [`current_ui`] and render directly:
//!
//! ```ignore
//! let mut ui = ply.begin();
//!
//! ui.element()
//!   .width(grow!())
//!   .height(grow!())
//!   .layout(|l| l.align(CenterX, CenterY))
//!   .children(|ui| {
//!     with_ui(ui, || {
//!       if 按钮("点我") {
//!         println!("Hi");
//!       }
//!     });
//!   });
//!
//! ui.show(|_| {}).await;
//! ```
//!
//! # Safety
//!
//! [`current_ui`] hands out a mutable reference to the [`Ui`] created by
//! `Ply::begin()` with its borrow lifetime boosted to `'static`. This is sound
//! only because every component uses it transiently, *within* the [`with_ui`]
//! scope that installed it, and components never hold the returned reference
//! across other components or frames. Never store a [`current_ui`] reference
//! past the surrounding [`with_ui`] scope.

use ply_engine::prelude::Ui;
use std::cell::Cell;

thread_local! {
    /// Raw pointer to the frame's active `Ui`; `null` outside [`with_ui`].
    static CURRENT_UI: Cell<*mut Ui<'static, ()>> = const { Cell::new(std::ptr::null_mut()) };
}

/// Runs `f` with `ui` installed as the current frame context, so components
/// with zero-argument signatures (e.g. `按钮("点我")`) can render into it.
/// The previously active context is restored afterwards (supports nesting).
pub fn with_ui<F: FnOnce()>(ui: &mut Ui<'_, ()>, f: F) {
    let ptr = ui as *mut Ui<'_, ()> as *mut Ui<'static, ()>;
    CURRENT_UI.with(|slot| {
        let prev = slot.replace(ptr);
        f();
        slot.set(prev);
    });
}

/// The current frame's [`Ui`], for building immediate-mode components.
/// See the [module docs](self) for the safety contract before using this.
pub fn current_ui() -> &'static mut Ui<'static, ()> {
    let ptr = CURRENT_UI.with(|slot| slot.get());
    assert!(
        !ptr.is_null(),
        "immediate-mode component called outside with_ui()"
    );
    unsafe { &mut *ptr }
}