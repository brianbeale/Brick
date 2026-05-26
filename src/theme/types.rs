#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_rgba(&self, alpha: f32) -> String {
        format!("rgba({},{},{},{:.2})", self.r, self.g, self.b, alpha)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Px(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rem(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Em(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Pct(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ms(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Deg(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rad(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Turn(pub f32);

pub enum Space {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Custom(Rem),
}

impl Space {
    pub fn to_css(&self) -> String {
        match self {
            Space::None => "0".to_string(),
            Space::Xs => "var(--brick-space-xs)".to_string(),
            Space::Sm => "var(--brick-space-sm)".to_string(),
            Space::Md => "var(--brick-space-md)".to_string(),
            Space::Lg => "var(--brick-space-lg)".to_string(),
            Space::Xl => "var(--brick-space-xl)".to_string(),
            Space::Custom(Rem(v)) => format!("{v}rem"),
        }
    }
}

/// An inline CSS style value. Accepted by `.style()` on any element builder.
/// Const-constructible via [`Style::from_str`] — suitable for `const` positions.
///
/// ```rust,no_run
/// # use brick::theme::Style;
/// const PUSH: Style = Style::from_str("margin-right: 0.5rem");
/// ```
pub struct Style(pub &'static str);

impl Style {
    pub const fn from_str(s: &'static str) -> Self {
        Style(s)
    }
}

impl AsRef<str> for Style {
    fn as_ref(&self) -> &str {
        self.0
    }
}
