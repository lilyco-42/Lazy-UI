//! 和风背景组件: 渐变 + 圆月 + 云 + 飘落花瓣(仿 web 版 HTML/CSS)。
//!
//! 直绘组件(不占 UI 流): 由调用方在主循环底层调用 `pet_background(now, w, h)`,
//! 例如在 clear_background 之后、立绘之前。
//! 配色从 `assets/components/background.toml` 读取, 未设置字段用内置默认。

use ply_engine::prelude::*;

use crate::components::config::PetBackgroundConfig;

fn cfg() -> &'static PetBackgroundConfig {
    PetBackgroundConfig::get()
}

/// 解包 0xRRGGBB 为 (r, g, b) 0..1 分量。
fn unpack(hex: u32) -> (f32, f32, f32) {
    (
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

/// 三段渐变插值: t in [0,1] → top→mid(0..0.5) / mid→bot(0.5..1)
fn gradient_color(c: &PetBackgroundConfig, t: f32) -> MacroquadColor {
    let top = unpack(c.gradient_top.unwrap_or(0xFDF0F6));
    let mid = unpack(c.gradient_mid.unwrap_or(0xF7E3EE));
    let bot = unpack(c.gradient_bot.unwrap_or(0xDCD0EC));
    let (a, b) = if t < 0.5 {
        (top, mid)
    } else {
        (mid, bot)
    };
    let k = if t < 0.5 { t * 2.0 } else { (t - 0.5) * 2.0 };
    MacroquadColor::new(
        a.0 + (b.0 - a.0) * k,
        a.1 + (b.1 - a.1) * k,
        a.2 + (b.2 - a.2) * k,
        1.0,
    )
}

/// 和风背景: 渐变底 + 右上圆月 + 漂移云朵 + 飘落花瓣。
pub fn pet_background(now: f32, w: f32, h: f32) {
    let c = cfg();

    // 1) 纵向渐变(分段矩形近似)
    let bands = c.gradient_bands.unwrap_or(64).max(8) as i32;
    for i in 0..bands {
        let t = i as f32 / (bands - 1) as f32;
        let y = h * t;
        draw_rectangle(0.0, y, w, h / bands as f32 + 1.0, gradient_color(&c, t));
    }

    // 2) 圆月(右上) + 光晕
    let mx = w * c.moon_x_ratio.unwrap_or(0.78);
    let my = h * c.moon_y_ratio.unwrap_or(0.12);
    let r = c.moon_radius.unwrap_or(56.0);
    let glow_r = c.moon_glow_radius.unwrap_or(90.0);
    let glow = unpack(c.moon_glow.unwrap_or(0xFFF3D8));
    draw_circle(mx, my, glow_r, MacroquadColor::new(glow.0, glow.1, glow.2, 0.28));
    let m = unpack(c.moon_color.unwrap_or(0xFCF0D2));
    draw_circle(mx, my, r, MacroquadColor::new(m.0, m.1, m.2, 1.0));

    // 3) 云朵(半透明白椭圆组, 缓慢漂移) — 默认关闭, 验证圆形绘制后开启
    if c.cloud_enabled.unwrap_or(false) {
        let cloud = unpack(c.cloud_color.unwrap_or(0xFFFFFF));
        let cloud_count = c.cloud_count.unwrap_or(3).min(6);
        for i in 0..cloud_count {
            let seed = i as f32 * 0.61803;
            let speed = 0.006 + seed.fract() * 0.01;
            let sy = 0.06 + seed.fract() * 0.26;
            let alpha = 0.6 + seed.fract() * 0.3;
            let off = ((now * speed + seed * 1.7).sin() + 1.0) * 0.5;
            let cx = off * (w + 320.0) - 160.0;
            let cy = h * sy + (now * 0.3 + seed * 3.0).sin() * 6.0;
            let cc = MacroquadColor::new(cloud.0, cloud.1, cloud.2, alpha);
            let s = 0.7 + seed.fract() * 0.6;
            draw_ellipse(cx, cy, 90.0 * s, 26.0 * s, 0.0, cc);
            draw_ellipse(cx - 46.0 * s, cy + 4.0 * s, 40.0 * s, 18.0 * s, 0.0, cc);
            draw_ellipse(cx + 42.0 * s, cy + 6.0 * s, 34.0 * s, 16.0 * s, 0.0, cc);
        }
    }

    // 4) 花瓣飘落(小椭圆旋转下落) — 默认关闭
    if c.petal_enabled.unwrap_or(false) {
        let petal = unpack(c.petal_color.unwrap_or(0xE896BE));
        let petal_count = c.petal_count.unwrap_or(14).min(30);
        for i in 0..petal_count {
            let seed = i as f32 * 0.61803;
            let speed = 0.045 + (seed * 7.0).fract() * 0.05;
            let y = ((now * speed + seed * 3.7).fract() * 1.08 - 0.04) * h;
            let x = (seed * 17.0 + (now * 0.02 + seed).sin() * 40.0).fract() * w;
            let rot = now * (0.5 + seed) + seed * 10.0;
            let s = 4.0 + seed.fract() * 3.0;
            draw_ellipse(x, y, s, s * 0.7, rot, MacroquadColor::new(petal.0, petal.1, petal.2, 0.5));
        }
    }
}
