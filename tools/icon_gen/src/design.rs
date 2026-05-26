use crate::png::Canvas;

// ── Palette ───────────────────────────────────────────────────────────────────

pub const BRICK:      [u8; 3] = [214,  66,  54]; // #D64236 — face
pub const BRICK_DARK: [u8; 3] = [140,  34,  24]; // #8C2218 — right shadow
pub const BRICK_MID:  [u8; 3] = [176,  50,  38]; // #B03226 — top face
pub const MORTAR:     [u8; 3] = [ 58,  19,  15]; // #3A130F — background / mortar
pub const EDGE:       [u8; 3] = [ 26,   8,   6]; // #1A0806 — outline (brand dark)

// ── 24×24 base grid ───────────────────────────────────────────────────────────
//
// A single isometric-ish pixel art brick viewed from slight above-left.
//
// Layout at 24×24:
//   Rows  0-2   : top face (lighter BRICK_MID) — 3 rows
//   Rows  3-20  : front face (BRICK) — 18 rows
//   Rows  21-23 : bottom shadow edge
//   Cols  0-17  : front/top face width
//   Cols 18-23  : right side face (BRICK_DARK shadow)
//
// Outer 1-px border (EDGE) defines the silhouette.
//
//   ┌──────────────────┐──────┐   ← top face (rows 0-2)
//   │     front face    │right │   ← rows 3-20
//   │                   │face  │
//   └──────────────────┴──────┘   ← rows 21-23
//
// The result is a classic rectangular brick with depth on one corner.

const W: u32 = 24;
const H: u32 = 24;

pub fn brick_canvas() -> Canvas {
    let mut c = Canvas::new(W, H, MORTAR);

    // ── Front face: rows 3-20, cols 0-17 ────────────────────────────────────
    for y in 3..=20 {
        for x in 0..=17 {
            let on_top_edge    = y == 3;
            let on_bottom_edge = y == 20;
            let on_left_edge   = x == 0;
            let on_right_edge  = x == 17;
            if on_top_edge || on_bottom_edge || on_left_edge || on_right_edge {
                c.set(x, y, EDGE);
            } else {
                c.set(x, y, BRICK);
            }
        }
    }

    // ── Top face: rows 0-2, cols 0-17 ────────────────────────────────────────
    for y in 0..=2 {
        for x in 0..=17 {
            let on_edge = y == 0 || x == 0 || x == 17;
            if on_edge {
                c.set(x, y, EDGE);
            } else {
                c.set(x, y, BRICK_MID);
            }
        }
    }

    // ── Right side face: rows 3-23, cols 18-23 ───────────────────────────────
    for y in 3..=23 {
        for x in 18..=23 {
            let on_edge = y == 3 || y == 23 || x == 18 || x == 23;
            if on_edge {
                c.set(x, y, EDGE);
            } else {
                c.set(x, y, BRICK_DARK);
            }
        }
    }

    // ── Top-right corner cap (joins top face and right face) ─────────────────
    // rows 0-2, cols 18-23 — dark top of right face
    for y in 0..=2 {
        for x in 18..=23 {
            let on_edge = y == 0 || x == 18 || x == 23;
            if on_edge {
                c.set(x, y, EDGE);
            } else {
                // slightly darker than top face to show the corner
                c.set(x, y, BRICK_DARK);
            }
        }
    }

    c
}
