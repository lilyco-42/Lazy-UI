//! Stateful component abstraction: lifecycle + cross-frame state on top of
//! ply-engine's immediate mode. Ported from the plyx_demo framework user set.
//!
//! The stateless helpers (`button`, `panel`, ...) draw and hand values back to
//! the caller. A [`Component`] instead **owns** its internal state and renders
//! every frame; a [`ComponentTree`] keeps instances alive across frames and
//! drives lifecycle (`on_mount` / `on_unmount`). This mirrors the Flutter
//! Widget / React Component model while staying true to Rust's ownership rules:
//! state lives in `&mut self`, never in shared globals.

use ply_engine::prelude::*;
use std::collections::HashMap;

/// A stateful UI component. `render` runs every frame; internal state lives in
/// `&mut self` and persists between frames (the immediate-mode bridge).
pub trait Component {
    /// Draw the component into `ui`. Called once per frame.
    fn render(&mut self, ui: &mut Ui<'_, ()>);
    /// Called once, on the first frame the component appears — the place for
    /// init side effects (starting a job/timer, seeding state).
    fn on_mount(&mut self, _ui: &mut Ui<'_, ()>) {}
    /// Called once when the component is removed from the tree (cleanup).
    fn on_unmount(&mut self) {}
}

struct Slot {
    mounted: bool,
    comp: Box<dyn Component>,
}

/// Persistent, id-keyed store for stateful components. Immediate mode rebuilds
/// the widget tree every frame, so instances that must keep state live here.
#[derive(Default)]
pub struct ComponentTree {
    slots: HashMap<u32, Slot>,
}

impl ComponentTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures a component exists at `id` (built via `init` on first
    /// appearance), mounts it once, then renders it every frame.
    ///
    /// ```rust,ignore
    /// components.show(ui, "log_panel", LogPanel::new);
    /// ```
    pub fn show<C: Component + 'static>(
        &mut self,
        ui: &mut Ui<'_, ()>,
        id: impl Into<Id>,
        init: impl FnOnce() -> C,
    ) {
        let key = id.into().id;
        let slot = self.slots.entry(key).or_insert_with(|| Slot {
            mounted: false,
            comp: Box::new(init()),
        });
        if !slot.mounted {
            slot.comp.on_mount(ui);
            slot.mounted = true;
        }
        slot.comp.render(ui);
    }

    /// Unmounts and drops the component at `id` (if present).
    pub fn hide(&mut self, id: impl Into<Id>) {
        let key = id.into().id;
        if let Some(mut slot) = self.slots.remove(&key) {
            if slot.mounted {
                slot.comp.on_unmount();
            }
        }
    }

    /// Whether a component is currently alive at `id`.
    pub fn is_alive(&self, id: impl Into<Id>) -> bool {
        self.slots.contains_key(&id.into().id)
    }

    /// Number of live components.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct Counts {
        renders: u32,
        mounts: u32,
        unmounts: u32,
    }

    struct TestWidget {
        c: Rc<RefCell<Counts>>,
    }

    impl Component for TestWidget {
        fn render(&mut self, _ui: &mut Ui<'_, ()>) {
            self.c.borrow_mut().renders += 1;
        }
        fn on_mount(&mut self, _ui: &mut Ui<'_, ()>) {
            self.c.borrow_mut().mounts += 1;
        }
        fn on_unmount(&mut self) {
            self.c.borrow_mut().unmounts += 1;
        }
    }

    fn headless_ui() -> (Ply<()>, impl Fn() -> Rc<RefCell<Counts>>) {
        let mut ply = Ply::<()>::new_headless(ply_engine::math::Dimensions::new(800.0, 600.0));
        ply.set_measure_text_function(|_, _| ply_engine::math::Dimensions::new(100.0, 24.0));
        (ply, || Rc::new(RefCell::new(Counts::default())))
    }

    /// `show` on an existing id does not rebuild the component: mounts exactly
    /// once, renders every call.
    #[test]
    fn mounts_once_renders_each_frame() {
        let (mut ply, counts) = headless_ui();
        let c = counts();
        let mut tree = ComponentTree::new();
        {
            let mut ui = ply.begin();
            let c1 = c.clone();
            tree.show(&mut ui, "w", || TestWidget { c: c1.clone() });
            // Same id again: `init` must NOT run; the existing instance renders.
            let c2 = c.clone();
            tree.show(&mut ui, "w", || TestWidget { c: c2 });
        }
        let s = c.borrow();
        assert_eq!(s.mounts, 1, "mount once");
        assert_eq!(s.renders, 2, "render every show");
        assert_eq!(s.unmounts, 0);
    }

    /// Distinct ids keep distinct instances and each mounts once.
    #[test]
    fn distinct_ids_are_separate() {
        let (mut ply, counts) = headless_ui();
        let c1 = counts();
        let c2 = counts();
        let mut tree = ComponentTree::new();
        {
            let mut ui = ply.begin();
            let a = c1.clone();
            tree.show(&mut ui, "a", || TestWidget { c: a });
            let b = c2.clone();
            tree.show(&mut ui, "b", || TestWidget { c: b });
        }
        assert_eq!(c1.borrow().mounts, 1);
        assert_eq!(c2.borrow().mounts, 1);
        assert_eq!(tree.len(), 2);
    }

    /// `hide` unmounts and drops; a later `show` remounts fresh.
    #[test]
    fn hide_unmounts() {
        let (mut ply, counts) = headless_ui();
        let c = counts();
        let mut tree = ComponentTree::new();
        {
            let mut ui = ply.begin();
            let c1 = c.clone();
            tree.show(&mut ui, "w", || TestWidget { c: c1.clone() });
            tree.hide("w");
        }
        let s = c.borrow();
        assert_eq!(s.mounts, 1);
        assert_eq!(s.unmounts, 1);
        assert!(tree.is_empty());
        assert!(!tree.is_alive("w"));
    }
}