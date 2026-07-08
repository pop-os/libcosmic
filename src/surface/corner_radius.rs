use iced::Rectangle;
use iced_runtime::platform_specific::wayland::CornerRadius;

#[must_use]
pub fn rounded_rect_strips(rect: Rectangle<f32>, radius: CornerRadius) -> Vec<Rectangle<f32>> {
    let mut out = Vec::new();

    let w = rect.width.max(0.0);
    let h = rect.height.max(0.0);

    if w <= 0.0 || h <= 0.0 {
        return out;
    }

    let tl = radius.top_left as i32;
    let tr = radius.top_right as i32;
    let bl = radius.bottom_left as i32;
    let br = radius.bottom_right as i32;

    let max_top = tl.max(tr);
    let max_bottom = bl.max(br);

    let center_y = rect.y + max_top as f32;
    let center_h = (h as i32 - max_top - max_bottom).max(0) as f32;

    if center_h > 0.0 {
        out.push(Rectangle {
            x: rect.x,
            y: center_y,
            width: w,
            height: center_h,
        });
    }

    for y in 0..max_top {
        let left = if y < tl { circle_inset(tl, y) } else { 0 };

        let right = if y < tr { circle_inset(tr, y) } else { 0 };

        let strip_x = rect.x + left as f32;
        let strip_w = (w as i32 - left - right).max(0) as f32;

        if strip_w > 0.0 {
            out.push(Rectangle {
                x: strip_x,
                y: rect.y + y as f32,
                width: strip_w,
                height: if h > 1. { 2. } else { 1. },
            });
        }
    }

    for y in 0..max_bottom {
        let cy = max_bottom - 1 - y;
        let left = if cy < bl { circle_inset(bl, cy) } else { 0 };

        let right = if cy < br { circle_inset(br, cy) } else { 0 };

        let strip_x = rect.x + left as f32;
        let strip_w = (w as i32 - left - right).max(0) as f32;

        if strip_w > 0.0 {
            out.push(Rectangle {
                x: strip_x,
                y: rect.y + h - max_bottom as f32 + y as f32 - if h > 1. { 1. } else { 0. },
                width: strip_w,
                height: if h > 1. { 2. } else { 1. },
            });
        }
    }

    out
}

fn circle_inset(radius: i32, y: i32) -> i32 {
    if radius <= 0 {
        return 0;
    }

    let fy = y as f32 + 0.5;
    let r = radius as f32;

    let x = r - (r * r - (r - fy) * (r - fy)).sqrt();

    x.floor() as i32
}
