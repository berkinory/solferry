use gpui_kit::{
    AnyElement, Bounds, IntoElement, ParentElement, Styled, canvas, div, fill, point, px, rgb, size,
};
use qrcode::{Color, QrCode};

const MODULE: f32 = 3.0;
const QUIET_ZONE: f32 = 12.0;

/// Paints a QR code on a white plate, so it scans in both themes.
pub fn code(data: &str) -> Option<AnyElement> {
    let qr = QrCode::new(data).ok()?;
    let width = qr.width();
    let colors = qr.to_colors();
    let side = px(MODULE * f32::from(u16::try_from(width).ok()?));
    let painted = canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let cell = size(px(MODULE), px(MODULE));
            for (index, color) in colors.iter().enumerate() {
                if *color != Color::Dark {
                    continue;
                }
                let (Ok(row), Ok(column)) =
                    (u16::try_from(index / width), u16::try_from(index % width))
                else {
                    continue;
                };
                let offset = point(px(MODULE * f32::from(column)), px(MODULE * f32::from(row)));
                let module = Bounds::new(bounds.origin + offset, cell);
                window.paint_quad(fill(module, rgb(0x0000_0000)));
            }
        },
    )
    .size(side);
    Some(
        div()
            .p(px(QUIET_ZONE))
            .rounded(px(12.))
            .bg(rgb(0x00ff_ffff))
            .child(painted)
            .into_any_element(),
    )
}
