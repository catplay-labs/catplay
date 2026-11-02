use crate::DirtyRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px(f32),
    Dp(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,

    CenterLeft,
    Center,
    CenterRight,

    BottomLeft,
    BottomCenter,
    BottomRight,

    // For text
    BaselineLeft,
    BaselineCenter,
    BaselineRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRect {
    pub x: Unit,
    pub y: Unit,
    pub w: Unit,
    pub h: Unit,
    pub anchor: Anchor,
}

pub fn resolve_unit(u: Unit, total: i32, dpi: f32) -> u32 {
    match u {
        Unit::Px(v) => v as u32,
        Unit::Dp(v) => (v * dpi / 160.0) as u32,
        Unit::Percent(p) => (p * total as f32) as u32,
    }
}

/// Resolves `r` to pixels and clips it to the `w` x `h` screen.
///
/// Anchoring is done in signed arithmetic: a centered or right/bottom-anchored rect that is larger
/// than the screen used to underflow `u32` (a panic in debug builds, a wrapped, off-screen rect in
/// release builds). Rects that fit on the screen come out unchanged.
pub fn resolve_rect(r: UiRect, w: i32, h: i32, dpi: f32) -> DirtyRect {
    let rw = resolve_unit(r.w, w, dpi) as i64;
    let rh = resolve_unit(r.h, h, dpi) as i64;

    let mut x = resolve_unit(r.x, w, dpi) as i64;
    let mut y = resolve_unit(r.y, h, dpi) as i64;

    match r.anchor {
        Anchor::TopLeft => {}
        Anchor::TopCenter => {
            x -= rw / 2;
        }
        Anchor::TopRight => {
            x -= rw;
        }

        Anchor::CenterLeft => {
            y -= rh / 2;
        }
        Anchor::Center => {
            x -= rw / 2;
            y -= rh / 2;
        }
        Anchor::CenterRight => {
            x -= rw;
            y -= rh / 2;
        }

        Anchor::BottomLeft => {
            y -= rh;
        }
        Anchor::BottomCenter => {
            x -= rw / 2;
            y -= rh;
        }
        Anchor::BottomRight => {
            x -= rw;
            y -= rh;
        }

        Anchor::BaselineLeft => {}
        Anchor::BaselineCenter => {
            x -= rw / 2;
        }
        Anchor::BaselineRight => {
            x -= rw;
        }
    }

    let (screen_w, screen_h) = (w.max(0) as i64, h.max(0) as i64);
    let (x0, y0) = (x.clamp(0, screen_w), y.clamp(0, screen_h));
    let (x1, y1) = ((x + rw).clamp(0, screen_w), (y + rh).clamp(0, screen_h));

    DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: Unit, h: Unit, anchor: Anchor) -> UiRect {
        UiRect {
            x: Unit::Percent(0.5),
            y: Unit::Percent(0.5),
            w,
            h,
            anchor,
        }
    }

    #[test]
    fn rect_inside_the_screen_is_unchanged() {
        let r = resolve_rect(rect(Unit::Dp(300.0), Unit::Dp(300.0), Anchor::Center), 1280, 720, 160.0);
        assert_eq!((r.x, r.y, r.w, r.h), (490, 210, 300, 300));
    }

    #[test]
    fn centered_rect_wider_than_the_screen_is_clipped_and_stays_centered() {
        // The 700 dp text box of the C2A overlay on a narrow, dense screen: 1225 px on 320 px.
        let r = resolve_rect(rect(Unit::Dp(700.0), Unit::Dp(0.0), Anchor::BaselineCenter), 320, 240, 280.0);
        assert_eq!((r.x, r.w), (0, 320));
        assert_eq!(r.x + r.w / 2, 160, "center must not move");
    }

    #[test]
    fn right_and_bottom_anchors_larger_than_the_screen_do_not_underflow() {
        let r = resolve_rect(rect(Unit::Px(1000.0), Unit::Px(1000.0), Anchor::BottomRight), 320, 240, 160.0);
        assert_eq!((r.x, r.y, r.w, r.h), (0, 0, 160, 120));
    }
}
