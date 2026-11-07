use gpui_kit::{AnyElement, IntoElement, ParentElement, Styled, div, px, rgb};
use qrcode::{Color, QrCode};

const MODULE: f32 = 4.0;
const QUIET_ZONE: f32 = 16.0;

/// Draws a QR code as dark rows on a white plate, so it scans in both themes.
pub fn code(data: &str) -> Option<AnyElement> {
    let qr = QrCode::new(data).ok()?;
    let width = qr.width();
    let colors = qr.to_colors();
    let rows = colors.chunks(width).map(|row| {
        div().flex().children(row.iter().map(|color| {
            let module = div().size(px(MODULE));
            if *color == Color::Dark { module.bg(rgb(0x0000_0000)) } else { module }
        }))
    });
    Some(div().p(px(QUIET_ZONE)).rounded(px(12.)).bg(rgb(0x00ff_ffff)).children(rows).into_any_element())
}
