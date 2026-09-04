//! Material 3 theme.
//!
//! Convention over configuration: sensible M3 baseline defaults are baked into
//! `Theme::default()`; `assets/theme.toml` can override any of them.

use serde::Deserialize;

/// State overlay colors (M3 tonal approximations) used for hover/pressed feedback.
pub const HOVER_PRIMARY: u32 = 0x7E6ABE; // tone 50
pub const PRESSED_PRIMARY: u32 = 0x4F378B; // tone 30
pub const HOVER_TONAL: u32 = 0xD6CCE8;
pub const PRESSED_TONAL: u32 = 0xC6B9D9;
pub const HOVER_PRIMARY_CONTAINER: u32 = 0xD6C6F5;
pub const PRESSED_PRIMARY_CONTAINER: u32 = 0xC8B4F0;
pub const HOVER_OUTLINED: (u8, u8, u8, u8) = (103, 80, 164, 20);
pub const PRESSED_OUTLINED: (u8, u8, u8, u8) = (103, 80, 164, 31);
pub const HOVER_TEXT: (u8, u8, u8, u8) = (103, 80, 164, 12);
pub const PRESSED_TEXT: (u8, u8, u8, u8) = (103, 80, 164, 20);

pub const TRANSPARENT: (u8, u8, u8, u8) = (0, 0, 0, 0);

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub colors: Colors,
    pub shapes: Shapes,
    pub text: TextTheme,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: Colors::default(),
            shapes: Shapes::default(),
            text: TextTheme::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub primary: u32,
    pub on_primary: u32,
    pub primary_container: u32,
    pub on_primary_container: u32,
    pub secondary: u32,
    pub on_secondary: u32,
    pub secondary_container: u32,
    pub on_secondary_container: u32,
    pub tertiary: u32,
    pub on_tertiary: u32,
    pub tertiary_container: u32,
    pub on_tertiary_container: u32,
    pub error: u32,
    pub on_error: u32,
    pub error_container: u32,
    pub on_error_container: u32,
    pub surface: u32,
    pub on_surface: u32,
    pub surface_variant: u32,
    pub on_surface_variant: u32,
    pub outline: u32,
    pub outline_variant: u32,
    pub surface_container_lowest: u32,
    pub surface_container_low: u32,
    pub surface_container: u32,
    pub surface_container_high: u32,
    pub surface_container_highest: u32,
    pub inverse_surface: u32,
    pub inverse_on_surface: u32,
    pub inverse_primary: u32,
    pub scrim: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Shapes {
    pub touch_target: f32,
    pub button_height: f32,
    pub field_height: f32,
    pub tab_height: f32,
    pub item_height: f32,
    pub track_height: f32,
    pub handle_size: f32,
    pub radius_xs: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct TextTheme {
    pub label_size: u16,
    pub body_size: u16,
    pub title_size: u16,
    pub headline_size: u16,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            primary: 0x6750A4,
            on_primary: 0xFFFFFF,
            primary_container: 0xEADDFF,
            on_primary_container: 0x21005D,
            secondary: 0x625B71,
            on_secondary: 0xFFFFFF,
            secondary_container: 0xE8DEF8,
            on_secondary_container: 0x1D192B,
            tertiary: 0x7D5260,
            on_tertiary: 0xFFFFFF,
            tertiary_container: 0xFFD8E4,
            on_tertiary_container: 0x31111D,
            error: 0xB3261E,
            on_error: 0xFFFFFF,
            error_container: 0xF9DEDC,
            on_error_container: 0x410E0B,
            surface: 0xFEF7FF,
            on_surface: 0x1D1B20,
            surface_variant: 0xE7E0EC,
            on_surface_variant: 0x49454F,
            outline: 0x79747E,
            outline_variant: 0xCAC4D0,
            surface_container_lowest: 0xFFFFFF,
            surface_container_low: 0xF7F2FA,
            surface_container: 0xF3EDF7,
            surface_container_high: 0xECE6F0,
            surface_container_highest: 0xE6E0E9,
            inverse_surface: 0x322F35,
            inverse_on_surface: 0xF5EFF7,
            inverse_primary: 0xD0BCFF,
            scrim: 0x000000,
        }
    }
}

impl Default for Shapes {
    fn default() -> Self {
        Self {
            touch_target: 48.0,
            button_height: 40.0,
            field_height: 56.0,
            tab_height: 48.0,
            item_height: 48.0,
            track_height: 4.0,
            handle_size: 20.0,
            radius_xs: 4.0,
            radius_sm: 8.0,
            radius_md: 12.0,
            radius_lg: 16.0,
        }
    }
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            label_size: 14,
            body_size: 16,
            title_size: 22,
            headline_size: 28,
        }
    }
}

/// Global theme, loaded once from `assets/theme.toml` (falls back to M3 defaults).
/// Sizes are multiplied by the effective scale (DPR × user zoom) at load time.
/// Reloaded in place by [`set_zoom`] — the leaked old copy stays valid so any
/// `&'static Theme` handed out mid-frame remains sound until the next frame.
pub fn theme() -> &'static Theme {
    let p = THEME_PTR.load(std::sync::atomic::Ordering::Acquire);
    if !p.is_null() {
        // SAFETY: only ever set by `reload_theme` from `Box::into_raw` and
        // never freed, so the pointee lives for `'static`.
        return unsafe { &*p };
    }
    reload_theme()
}

static THEME_PTR: std::sync::atomic::AtomicPtr<Theme> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

fn reload_theme() -> &'static Theme {
    let mut t: Theme = toml::from_str(include_str!("../assets/theme.toml")).unwrap_or_default();
    t.scale_sizes(effective_scale());
    let leaked = Box::into_raw(Box::new(t));
    THEME_PTR.store(leaked, std::sync::atomic::Ordering::Release);
    // SAFETY: `leaked` was produced by `Box::into_raw` above and never freed.
    unsafe { &*leaked }
}

/// 全局 UI 缩放基线(= 设备 DPR)。宿主必须在首帧前调用 [`set_scale`]
/// (或一步到位的 [`init_scale`]),之后 `theme()` 首次加载时把所有尺寸字段
/// 乘上去 —— 手机上按物理像素渲染的布局从此与桌面观感一致
/// (不缩放时 16px 正文在 2.6x 屏上只有 ~6pt)。
static UI_SCALE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// 用户缩放倍率(Readme 的"缩放按钮"),与 DPR 相乘后生效。0 = 1.0。
static ZOOM: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn bits_to_f32(bits: u32) -> f32 {
    if bits == 0 {
        1.0
    } else {
        f32::from_bits(bits)
    }
}

/// 设置全局 UI 缩放基线(每进程一次;重复调用以首次为准,值下限 1.0).
pub fn set_scale(v: f32) {
    let v = v.max(1.0);
    let _ = UI_SCALE.compare_exchange(
        0,
        v.to_bits(),
        std::sync::atomic::Ordering::AcqRel,
        std::sync::atomic::Ordering::Acquire,
    );
}

/// 当前全局 UI 缩放基线(未设置时 1.0).
pub fn scale() -> f32 {
    bits_to_f32(UI_SCALE.load(std::sync::atomic::Ordering::Acquire))
}

/// 当前用户缩放倍率(未设置时 1.0).
pub fn zoom() -> f32 {
    bits_to_f32(ZOOM.load(std::sync::atomic::Ordering::Acquire))
}

/// 实际生效的缩放 = DPR 基线 × 用户倍率。组件样式表(`*.toml`)的尺寸字段
/// 在每帧读取时乘上它,所以 [`set_zoom`] 重载后下一帧即生效。
pub fn effective_scale() -> f32 {
    scale() * zoom()
}

/// 运行时缩放按钮:以 `v` 作为用户倍率(与设备 DPR 相乘)重载全局主题,
/// 下一帧立即生效。重复调用无需重启;传 1.0 恢复默认。
pub fn set_zoom(v: f32) -> &'static Theme {
    ZOOM.store(v.max(1.0).to_bits(), std::sync::atomic::Ordering::Release);
    reload_theme()
}

/// 宿主一步接入:读取平台 DPR(`high_dpi` 窗口下的 HiDPI/移动端密度)
/// 并应用为全局缩放。在首帧前调用一次即可,重复调用以首次为准。
pub fn init_scale() {
    let dpr = ply_engine::prelude::macroquad::window::screen_dpi_scale();
    set_scale(if dpr.is_finite() && dpr > 0.0 { dpr } else { 1.0 });
}

impl Theme {
    /// 把全部尺寸字段乘以 `s`(DPR 感知渲染用).
    pub fn scale_sizes(&mut self, s: f32) {
        self.shapes.touch_target *= s;
        self.shapes.button_height *= s;
        self.shapes.field_height *= s;
        self.shapes.tab_height *= s;
        self.shapes.item_height *= s;
        self.shapes.track_height *= s;
        self.shapes.handle_size *= s;
        self.shapes.radius_xs *= s;
        self.shapes.radius_sm *= s;
        self.shapes.radius_md *= s;
        self.shapes.radius_lg *= s;
        let t = &mut self.text;
        t.label_size = (t.label_size as f32 * s).round() as u16;
        t.body_size = (t.body_size as f32 * s).round() as u16;
        t.title_size = (t.title_size as f32 * s).round() as u16;
        t.headline_size = (t.headline_size as f32 * s).round() as u16;
    }
}

/// UI-scale-aware px helper. Components that ship their own stylesheets should
/// route sizes through here so they follow DPR and the user zoom (缩放按钮)
/// without touching every component.
pub fn px(v: f32) -> f32 {
    v * effective_scale()
}

/// `px` cast to `u16` (layout padding units).
pub fn pxu(v: f32) -> u16 {
    px(v) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pure transform only: `set_scale`/`set_zoom` write process-global state,
    // so exercising them here would leak the factor into every other test.
    #[test]
    fn scale_sizes_multiplies_every_dimension() {
        let mut t = Theme::default();
        let body = t.text.body_size;
        let btn_h = t.shapes.button_height;
        let radius = t.shapes.radius_md;
        t.scale_sizes(2.0);
        assert_eq!(t.text.body_size, body * 2);
        assert!((t.shapes.button_height - btn_h * 2.0).abs() < f32::EPSILON);
        assert!((t.shapes.radius_md - radius * 2.0).abs() < f32::EPSILON);
        t.scale_sizes(0.5);
        assert_eq!(t.text.body_size, body);
    }

    #[test]
    fn reload_theme_scales_shape_and_text_fields() {
        // Never calls set_scale/set_zoom: reload at identity must equal the
        // on-disk toml (multiplying by 1.0 is exact for f32 and u16 rounding).
        let t = theme();
        assert!(t.shapes.button_height > 0.0);
        assert!(t.text.body_size > 0);
    }
}
