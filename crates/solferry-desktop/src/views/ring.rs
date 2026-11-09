use std::f32::consts::{FRAC_PI_2, TAU};

use gpui_kit::{Bounds, Hsla, IntoElement, PathBuilder, Pixels, Styled, Window, canvas, point, px};

fn arc(bounds: Bounds<Pixels>, sweep: f32, tone: Hsla, window: &mut Window) {
    let center = bounds.center();
    let radius = f32::from(bounds.size.width) / 2. - 1.;
    let at = |angle: f32| {
        let turned = angle - FRAC_PI_2;
        point(center.x + px(turned.cos() * radius), center.y + px(turned.sin() * radius))
    };
    let mut path = PathBuilder::stroke(px(2.));
    path.move_to(at(0.));
    for step in 1..=48u8 {
        path.line_to(at(sweep * f32::from(step) / 48.));
    }
    if let Ok(built) = path.build() {
        window.paint_path(built, tone);
    }
}

/// How far along a transfer is, drawn by hand because only paths can curve.
pub fn ring(fraction: f32, tone: Hsla, size: f32) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            arc(bounds, TAU, tone.opacity(0.25), window);
            if fraction > 0. {
                arc(bounds, TAU * fraction.min(1.), tone, window);
            }
        },
    )
    .flex_none()
    .size(px(size))
}
