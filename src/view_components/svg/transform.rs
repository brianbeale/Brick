/// A composable SVG transform — build with named constructors and chain with `.then_*` methods.
///
/// ```rust,ignore
/// Transform::translate(10.0, 20.0).then_rotate(45.0).then_scale(2.0)
/// // → "translate(10,20) rotate(45) scale(2)"
///
/// Transform::rotate_around(90.0, 50.0, 50.0)
/// // → "rotate(90,50,50)"
/// ```
#[derive(Clone, Default)]
pub struct Transform {
    ops: Vec<TransformOp>,
}

#[derive(Clone)]
enum TransformOp {
    Translate(f64, f64),
    Rotate(f64),
    RotateAround(f64, f64, f64),
    Scale(f64, f64),
    SkewX(f64),
    SkewY(f64),
    Matrix(f64, f64, f64, f64, f64, f64),
}

impl TransformOp {
    fn to_string(&self) -> String {
        match self {
            TransformOp::Translate(x, y) => format!("translate({},{})", fmt_f64(*x), fmt_f64(*y)),
            TransformOp::Rotate(a) => format!("rotate({})", fmt_f64(*a)),
            TransformOp::RotateAround(a, cx, cy) => {
                format!("rotate({},{},{})", fmt_f64(*a), fmt_f64(*cx), fmt_f64(*cy))
            }
            TransformOp::Scale(x, y) if x == y => format!("scale({})", fmt_f64(*x)),
            TransformOp::Scale(x, y) => format!("scale({},{})", fmt_f64(*x), fmt_f64(*y)),
            TransformOp::SkewX(a) => format!("skewX({})", fmt_f64(*a)),
            TransformOp::SkewY(a) => format!("skewY({})", fmt_f64(*a)),
            TransformOp::Matrix(a, b, c, d, e, f) => format!(
                "matrix({},{},{},{},{},{})",
                fmt_f64(*a),
                fmt_f64(*b),
                fmt_f64(*c),
                fmt_f64(*d),
                fmt_f64(*e),
                fmt_f64(*f)
            ),
        }
    }
}

fn fmt_f64(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

impl Transform {
    pub fn translate(x: f64, y: f64) -> Self {
        Transform {
            ops: vec![TransformOp::Translate(x, y)],
        }
    }
    pub fn rotate(angle: f64) -> Self {
        Transform {
            ops: vec![TransformOp::Rotate(angle)],
        }
    }
    pub fn rotate_around(angle: f64, cx: f64, cy: f64) -> Self {
        Transform {
            ops: vec![TransformOp::RotateAround(angle, cx, cy)],
        }
    }
    pub fn scale(s: f64) -> Self {
        Transform {
            ops: vec![TransformOp::Scale(s, s)],
        }
    }
    pub fn scale_xy(sx: f64, sy: f64) -> Self {
        Transform {
            ops: vec![TransformOp::Scale(sx, sy)],
        }
    }
    pub fn skew_x(angle: f64) -> Self {
        Transform {
            ops: vec![TransformOp::SkewX(angle)],
        }
    }
    pub fn skew_y(angle: f64) -> Self {
        Transform {
            ops: vec![TransformOp::SkewY(angle)],
        }
    }
    pub fn matrix(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Transform {
            ops: vec![TransformOp::Matrix(a, b, c, d, e, f)],
        }
    }

    pub fn then_translate(mut self, x: f64, y: f64) -> Self {
        self.ops.push(TransformOp::Translate(x, y));
        self
    }
    pub fn then_rotate(mut self, angle: f64) -> Self {
        self.ops.push(TransformOp::Rotate(angle));
        self
    }
    pub fn then_rotate_around(mut self, angle: f64, cx: f64, cy: f64) -> Self {
        self.ops.push(TransformOp::RotateAround(angle, cx, cy));
        self
    }
    pub fn then_scale(mut self, s: f64) -> Self {
        self.ops.push(TransformOp::Scale(s, s));
        self
    }
    pub fn then_scale_xy(mut self, sx: f64, sy: f64) -> Self {
        self.ops.push(TransformOp::Scale(sx, sy));
        self
    }
    pub fn then_skew_x(mut self, angle: f64) -> Self {
        self.ops.push(TransformOp::SkewX(angle));
        self
    }
    pub fn then_skew_y(mut self, angle: f64) -> Self {
        self.ops.push(TransformOp::SkewY(angle));
        self
    }
    pub fn then_matrix(mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        self.ops.push(TransformOp::Matrix(a, b, c, d, e, f));
        self
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self
            .ops
            .iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{}", s)
    }
}

/// Anything that can be used as an SVG `transform` attribute value.
pub trait IntoSvgTransform {
    fn into_svg_transform(self) -> String;
}
impl IntoSvgTransform for Transform {
    fn into_svg_transform(self) -> String {
        self.to_string()
    }
}
impl IntoSvgTransform for &str {
    fn into_svg_transform(self) -> String {
        self.to_string()
    }
}
impl IntoSvgTransform for String {
    fn into_svg_transform(self) -> String {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_formats() {
        assert_eq!(
            Transform::translate(10.0, 20.0).to_string(),
            "translate(10,20)"
        );
    }

    #[test]
    fn translate_float_formats() {
        assert_eq!(
            Transform::translate(10.5, 20.0).to_string(),
            "translate(10.5,20)"
        );
    }

    #[test]
    fn rotate_formats() {
        assert_eq!(Transform::rotate(45.0).to_string(), "rotate(45)");
    }

    #[test]
    fn rotate_around_formats() {
        assert_eq!(
            Transform::rotate_around(90.0, 50.0, 50.0).to_string(),
            "rotate(90,50,50)"
        );
    }

    #[test]
    fn scale_uniform_omits_y() {
        assert_eq!(Transform::scale(2.0).to_string(), "scale(2)");
    }

    #[test]
    fn scale_non_uniform() {
        assert_eq!(Transform::scale_xy(2.0, 3.0).to_string(), "scale(2,3)");
    }

    #[test]
    fn skew_x_formats() {
        assert_eq!(Transform::skew_x(15.0).to_string(), "skewX(15)");
    }

    #[test]
    fn matrix_formats() {
        assert_eq!(
            Transform::matrix(1.0, 0.0, 0.0, 1.0, 10.0, 20.0).to_string(),
            "matrix(1,0,0,1,10,20)"
        );
    }

    #[test]
    fn chained_ops() {
        let t = Transform::translate(10.0, 0.0)
            .then_rotate(45.0)
            .then_scale(2.0);
        assert_eq!(t.to_string(), "translate(10,0) rotate(45) scale(2)");
    }

    #[test]
    fn str_passthrough() {
        assert_eq!("rotate(45)".into_svg_transform(), "rotate(45)");
    }
}
