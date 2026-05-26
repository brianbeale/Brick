/// SVG path data builder — construct `d` attribute values without writing raw path strings.
///
/// ```rust,ignore
/// PathData::new()
///     .move_to(10.0, 20.0)
///     .line_to(100.0, 20.0)
///     .arc(50.0, 50.0, 0.0, false, true, 150.0, 80.0)
///     .close()
/// ```
#[derive(Clone, Default)]
pub struct PathData {
    ops: Vec<PathOp>,
}

#[derive(Clone)]
enum PathOp {
    // Absolute
    MoveTo(f64, f64),
    LineTo(f64, f64),
    HLine(f64),
    VLine(f64),
    Cubic(f64, f64, f64, f64, f64, f64),
    SmoothCubic(f64, f64, f64, f64),
    Quad(f64, f64, f64, f64),
    SmoothQuad(f64, f64),
    Arc {
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64,
        y: f64,
    },
    // Relative
    MoveBy(f64, f64),
    LineBy(f64, f64),
    HBy(f64),
    VBy(f64),
    CubicBy(f64, f64, f64, f64, f64, f64),
    SmoothCubicBy(f64, f64, f64, f64),
    QuadBy(f64, f64, f64, f64),
    SmoothQuadBy(f64, f64),
    ArcBy {
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        dx: f64,
        dy: f64,
    },
    Close,
}

fn f(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

fn flag(b: bool) -> &'static str {
    if b { "1" } else { "0" }
}

impl PathOp {
    fn to_string(&self) -> String {
        match self {
            PathOp::MoveTo(x, y) => format!("M{},{}", f(*x), f(*y)),
            PathOp::LineTo(x, y) => format!("L{},{}", f(*x), f(*y)),
            PathOp::HLine(x) => format!("H{}", f(*x)),
            PathOp::VLine(y) => format!("V{}", f(*y)),
            PathOp::Cubic(x1, y1, x2, y2, x, y) => format!(
                "C{},{} {},{} {},{}",
                f(*x1),
                f(*y1),
                f(*x2),
                f(*y2),
                f(*x),
                f(*y)
            ),
            PathOp::SmoothCubic(x2, y2, x, y) => {
                format!("S{},{} {},{}", f(*x2), f(*y2), f(*x), f(*y))
            }
            PathOp::Quad(cx, cy, x, y) => format!("Q{},{} {},{}", f(*cx), f(*cy), f(*x), f(*y)),
            PathOp::SmoothQuad(x, y) => format!("T{},{}", f(*x), f(*y)),
            PathOp::Arc {
                rx,
                ry,
                rotation,
                large_arc,
                sweep,
                x,
                y,
            } => format!(
                "A{},{} {} {} {} {},{}",
                f(*rx),
                f(*ry),
                f(*rotation),
                flag(*large_arc),
                flag(*sweep),
                f(*x),
                f(*y)
            ),
            // Relative
            PathOp::MoveBy(dx, dy) => format!("m{},{}", f(*dx), f(*dy)),
            PathOp::LineBy(dx, dy) => format!("l{},{}", f(*dx), f(*dy)),
            PathOp::HBy(dx) => format!("h{}", f(*dx)),
            PathOp::VBy(dy) => format!("v{}", f(*dy)),
            PathOp::CubicBy(x1, y1, x2, y2, dx, dy) => format!(
                "c{},{} {},{} {},{}",
                f(*x1),
                f(*y1),
                f(*x2),
                f(*y2),
                f(*dx),
                f(*dy)
            ),
            PathOp::SmoothCubicBy(x2, y2, dx, dy) => {
                format!("s{},{} {},{}", f(*x2), f(*y2), f(*dx), f(*dy))
            }
            PathOp::QuadBy(cx, cy, dx, dy) => {
                format!("q{},{} {},{}", f(*cx), f(*cy), f(*dx), f(*dy))
            }
            PathOp::SmoothQuadBy(dx, dy) => format!("t{},{}", f(*dx), f(*dy)),
            PathOp::ArcBy {
                rx,
                ry,
                rotation,
                large_arc,
                sweep,
                dx,
                dy,
            } => format!(
                "a{},{} {} {} {} {},{}",
                f(*rx),
                f(*ry),
                f(*rotation),
                flag(*large_arc),
                flag(*sweep),
                f(*dx),
                f(*dy)
            ),
            PathOp::Close => "Z".to_string(),
        }
    }
}

impl PathData {
    pub fn new() -> Self {
        PathData { ops: Vec::new() }
    }

    // ── Absolute ─────────────────────────────────────────────────────────────

    pub fn move_to(mut self, x: f64, y: f64) -> Self {
        self.ops.push(PathOp::MoveTo(x, y));
        self
    }
    pub fn line_to(mut self, x: f64, y: f64) -> Self {
        self.ops.push(PathOp::LineTo(x, y));
        self
    }
    pub fn h(mut self, x: f64) -> Self {
        self.ops.push(PathOp::HLine(x));
        self
    }
    pub fn v(mut self, y: f64) -> Self {
        self.ops.push(PathOp::VLine(y));
        self
    }
    pub fn cubic(mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Self {
        self.ops.push(PathOp::Cubic(x1, y1, x2, y2, x, y));
        self
    }
    pub fn smooth_cubic(mut self, x2: f64, y2: f64, x: f64, y: f64) -> Self {
        self.ops.push(PathOp::SmoothCubic(x2, y2, x, y));
        self
    }
    pub fn quad(mut self, cx: f64, cy: f64, x: f64, y: f64) -> Self {
        self.ops.push(PathOp::Quad(cx, cy, x, y));
        self
    }
    pub fn smooth_quad(mut self, x: f64, y: f64) -> Self {
        self.ops.push(PathOp::SmoothQuad(x, y));
        self
    }
    pub fn arc(
        mut self,
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64,
        y: f64,
    ) -> Self {
        self.ops.push(PathOp::Arc {
            rx,
            ry,
            rotation,
            large_arc,
            sweep,
            x,
            y,
        });
        self
    }
    pub fn close(mut self) -> Self {
        self.ops.push(PathOp::Close);
        self
    }

    // ── Relative ─────────────────────────────────────────────────────────────

    pub fn move_by(mut self, dx: f64, dy: f64) -> Self {
        self.ops.push(PathOp::MoveBy(dx, dy));
        self
    }
    pub fn line_by(mut self, dx: f64, dy: f64) -> Self {
        self.ops.push(PathOp::LineBy(dx, dy));
        self
    }
    pub fn h_by(mut self, dx: f64) -> Self {
        self.ops.push(PathOp::HBy(dx));
        self
    }
    pub fn v_by(mut self, dy: f64) -> Self {
        self.ops.push(PathOp::VBy(dy));
        self
    }
    pub fn cubic_by(mut self, x1: f64, y1: f64, x2: f64, y2: f64, dx: f64, dy: f64) -> Self {
        self.ops.push(PathOp::CubicBy(x1, y1, x2, y2, dx, dy));
        self
    }
    pub fn smooth_cubic_by(mut self, x2: f64, y2: f64, dx: f64, dy: f64) -> Self {
        self.ops.push(PathOp::SmoothCubicBy(x2, y2, dx, dy));
        self
    }
    pub fn quad_by(mut self, cx: f64, cy: f64, dx: f64, dy: f64) -> Self {
        self.ops.push(PathOp::QuadBy(cx, cy, dx, dy));
        self
    }
    pub fn smooth_quad_by(mut self, dx: f64, dy: f64) -> Self {
        self.ops.push(PathOp::SmoothQuadBy(dx, dy));
        self
    }
    pub fn arc_by(
        mut self,
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        dx: f64,
        dy: f64,
    ) -> Self {
        self.ops.push(PathOp::ArcBy {
            rx,
            ry,
            rotation,
            large_arc,
            sweep,
            dx,
            dy,
        });
        self
    }
}

impl std::fmt::Display for PathData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.ops
                .iter()
                .map(|op| op.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path() {
        let d = PathData::new()
            .move_to(10.0, 20.0)
            .line_to(100.0, 20.0)
            .close();
        assert_eq!(d.to_string(), "M10,20 L100,20 Z");
    }

    #[test]
    fn arc_flags() {
        let d = PathData::new()
            .move_to(0.0, 0.0)
            .arc(50.0, 50.0, 0.0, false, true, 100.0, 0.0);
        assert_eq!(d.to_string(), "M0,0 A50,50 0 0 1 100,0");
    }

    #[test]
    fn relative_move() {
        let d = PathData::new().move_to(0.0, 0.0).move_by(10.0, 5.0);
        assert_eq!(d.to_string(), "M0,0 m10,5");
    }

    #[test]
    fn cubic_bezier() {
        let d = PathData::new()
            .move_to(0.0, 0.0)
            .cubic(20.0, 0.0, 80.0, 100.0, 100.0, 100.0);
        assert_eq!(d.to_string(), "M0,0 C20,0 80,100 100,100");
    }

    #[test]
    fn quad_bezier() {
        let d = PathData::new()
            .move_to(0.0, 0.0)
            .quad(50.0, 0.0, 100.0, 100.0);
        assert_eq!(d.to_string(), "M0,0 Q50,0 100,100");
    }

    #[test]
    fn h_v_shortcuts() {
        let d = PathData::new().move_to(0.0, 0.0).h(100.0).v(50.0);
        assert_eq!(d.to_string(), "M0,0 H100 V50");
    }

    #[test]
    fn float_values_preserved() {
        let d = PathData::new().move_to(10.5, 20.25);
        assert_eq!(d.to_string(), "M10.5,20.25");
    }
}
