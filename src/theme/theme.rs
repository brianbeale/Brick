use super::types::Color;

/// All visual design tokens for a Brick theme.
/// Const-constructible — `DEFAULT_THEME` and `DARK_THEME` are `const`.
#[derive(Clone, Copy)]
pub struct Theme {
    // Accent palette
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_fg: Color,
    pub accent_alpha: f32,

    // Semantic colors
    pub danger: Color,
    pub success: Color,
    pub warning: Color,

    // Surfaces
    pub bg: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub border: Color,

    // Text
    pub text: Color,
    pub text_muted: Color,

    // Code block
    pub code_bg: Color,
    pub code_fg: Color,

    // Typography
    pub font_sans: &'static str,
    pub font_mono: &'static str,

    // Spacing scale (rem)
    pub space_xs: f32,
    pub space_sm: f32,
    pub space_md: f32,
    pub space_lg: f32,
    pub space_xl: f32,

    // Border radius (rem)
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,

    // Typography scale (rem) — used by .text_xs() … .text_2xl() builders
    pub text_xs: f32,
    pub text_sm: f32,
    pub text_md: f32,
    pub text_lg: f32,
    pub text_xl: f32,
    pub text_2xl: f32,
}

impl Theme {
    pub fn to_css_vars(&self) -> String {
        let faint = self.accent.to_rgba(self.accent_alpha);
        let mut v = String::from(":root {\n");
        let mut p = |name: &str, val: String| {
            v.push_str("  --brick-");
            v.push_str(name);
            v.push(':');
            v.push(' ');
            v.push_str(&val);
            v.push_str(";\n");
        };
        p("accent", self.accent.to_hex());
        p("accent-hover", self.accent_hover.to_hex());
        p("accent-fg", self.accent_fg.to_hex());
        p("accent-faint", faint);
        p("danger", self.danger.to_hex());
        p("success", self.success.to_hex());
        p("warning", self.warning.to_hex());
        p("bg", self.bg.to_hex());
        p("surface", self.surface.to_hex());
        p("surface-raised", self.surface_raised.to_hex());
        p("border", self.border.to_hex());
        p("text", self.text.to_hex());
        p("text-muted", self.text_muted.to_hex());
        p("code-bg", self.code_bg.to_hex());
        p("code-fg", self.code_fg.to_hex());
        p("font-sans", self.font_sans.to_string());
        p("font-mono", self.font_mono.to_string());
        p("space-xs", format!("{}rem", self.space_xs));
        p("space-sm", format!("{}rem", self.space_sm));
        p("space-md", format!("{}rem", self.space_md));
        p("space-lg", format!("{}rem", self.space_lg));
        p("space-xl", format!("{}rem", self.space_xl));
        p("radius-sm", format!("{}rem", self.radius_sm));
        p("radius-md", format!("{}rem", self.radius_md));
        p("radius-lg", format!("{}rem", self.radius_lg));
        p("text-xs", format!("{}rem", self.text_xs));
        p("text-sm", format!("{}rem", self.text_sm));
        p("text-md", format!("{}rem", self.text_md));
        p("text-lg", format!("{}rem", self.text_lg));
        p("text-xl", format!("{}rem", self.text_xl));
        p("text-2xl", format!("{}rem", self.text_2xl));
        v.push('}');
        v
    }
}

/// A light + optional dark theme pair. Call `.inject()` once at startup.
///
/// Set `prefer_dark: true` to make the dark theme the default (shown unless
/// the user explicitly prefers light). Useful during development.
pub struct ThemeSet {
    pub light: Theme,
    pub dark: Option<Theme>,
    /// When `true`, the dark theme is the `:root` default and the light theme
    /// is only applied under `@media (prefers-color-scheme: light)`.
    pub prefer_dark: bool,
}

impl ThemeSet {
    pub fn to_css(&self) -> String {
        let mut css = String::new();

        css.push_str("@layer brick.reset, brick.theme, brick.components, brick.utilities;\n\n");

        // CSS custom properties live outside any @layer so they are unconditional —
        // unlayered styles always beat layered ones regardless of specificity.
        if self.prefer_dark {
            // Force dark unconditionally — no media query, system preference ignored.
            let dark = self.dark.as_ref().unwrap_or(&self.light);
            css.push_str(&dark.to_css_vars());
            css.push('\n');
        } else {
            css.push_str(&self.light.to_css_vars());
            css.push('\n');
            if let Some(ref dark) = self.dark {
                css.push_str("@media (prefers-color-scheme: dark) {\n");
                css.push_str(&dark.to_css_vars());
                css.push_str("\n}\n");
            }
        }

        css.push_str("\n@layer brick.components {\n");
        css.push_str(SEMANTIC_CSS);
        css.push_str("}\n");

        css
    }

    #[cfg(brick_dom)]
    pub fn inject(&self) {
        use wasm_bindgen::JsCast;
        let document = web_sys::window().unwrap().document().unwrap();
        let style_el = document.create_element("style").unwrap();
        style_el.set_inner_html(&self.to_css());
        document
            .body()
            .unwrap()
            .append_child(style_el.unchecked_ref())
            .unwrap();
    }
}

// ── Semantic component class definitions ─────────────────────────────────────

const SEMANTIC_CSS: &str = r#"
/* Variants */
.brick-primary {
  background: var(--brick-accent);
  color: var(--brick-accent-fg);
  border: none;
  border-radius: var(--brick-radius-sm);
  padding: 0.4rem 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
}
.brick-primary:hover  { background: var(--brick-accent-hover); }
.brick-primary:active { transform: scale(0.97); }

.brick-tonal {
  background: var(--brick-accent-faint);
  color: var(--brick-accent);
  border: none;
  border-radius: var(--brick-radius-sm);
  padding: 0.4rem 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: filter 0.15s, transform 0.1s;
}
.brick-tonal:hover  { filter: brightness(0.93); }
.brick-tonal:active { transform: scale(0.97); }

.brick-secondary {
  background: transparent;
  color: var(--brick-accent);
  border: 1.5px solid var(--brick-accent);
  border-radius: var(--brick-radius-sm);
  padding: calc(0.4rem - 1.5px) calc(1rem - 1.5px);
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
}
.brick-secondary:hover  { background: var(--brick-accent-faint); }
.brick-secondary:active { transform: scale(0.97); }

.brick-ghost {
  background: transparent;
  color: var(--brick-text-muted);
  border: none;
  border-radius: var(--brick-radius-sm);
  padding: 0.4rem 1rem;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.brick-ghost:hover { background: var(--brick-surface-raised); color: var(--brick-text); }

.brick-danger {
  background: var(--brick-danger);
  color: #fff;
  border: none;
  border-radius: var(--brick-radius-sm);
  padding: 0.4rem 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s, transform 0.1s;
}
.brick-danger:hover  { opacity: 0.88; }
.brick-danger:active { transform: scale(0.97); }

.brick-success {
  background: var(--brick-success);
  color: #fff;
  border: none;
  border-radius: var(--brick-radius-sm);
  padding: 0.4rem 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s, transform 0.1s;
}
.brick-success:hover  { opacity: 0.88; }
.brick-success:active { transform: scale(0.97); }

.brick-muted {
  color: var(--brick-text-muted);
  font-size: 0.875rem;
}

/* Typography roles */
.brick-title {
  font-size: var(--brick-text-xl);
  font-weight: 600;
  line-height: 1.3;
}
.brick-caption {
  font-size: var(--brick-text-sm);
  color: var(--brick-text-muted);
}
.brick-label {
  font-size: var(--brick-text-xs);
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--brick-text-muted);
}

/* Sizes */
.brick-sm { font-size: 0.8rem; }
.brick-lg { font-size: 1rem; }
button.brick-sm { padding: 0.25rem 0.65rem; }
button.brick-lg { padding: 0.5rem 1.4rem; }

/* Typography scale */
.brick-text-xs  { font-size: var(--brick-text-xs);  }
.brick-text-sm  { font-size: var(--brick-text-sm);  }
.brick-text-md  { font-size: var(--brick-text-md);  }
.brick-text-lg  { font-size: var(--brick-text-lg);  }
.brick-text-xl  { font-size: var(--brick-text-xl);  }
.brick-text-2xl { font-size: var(--brick-text-2xl); }

/* Layout */
.brick-full-width { width: 100%; display: block; }
.brick-row {
  display: flex;
  flex-direction: row;
  gap: var(--brick-space-sm);
  align-items: center;
  margin-top: 0.75rem;
}
.brick-row > button { margin-top: 0; }
.brick-row > p     { margin-top: 0; }
.brick-col { display: flex; flex-direction: column; gap: var(--brick-space-sm); }
.brick-center { display: flex; align-items: center; justify-content: center; }

/* Typography */
.brick-bold   { font-weight: 700; }
.brick-italic { font-style: italic; }
.brick-mono   { font-family: var(--brick-font-mono); }
.brick-truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
"#;

// ── Slate light theme — neutral warm off-white ────────────────────────────────

pub const SLATE_THEME: Theme = Theme {
    accent: Color::rgb(172, 58, 38),
    accent_hover: Color::rgb(144, 44, 26),
    accent_fg: Color::rgb(255, 255, 255),
    accent_alpha: 0.12,

    danger: Color::rgb(196, 36, 36),
    success: Color::rgb(36, 138, 70),
    warning: Color::rgb(204, 136, 16),

    bg: Color::rgb(247, 242, 238),
    surface: Color::rgb(255, 252, 248),
    surface_raised: Color::rgb(250, 246, 241),
    border: Color::rgb(220, 208, 198),

    text: Color::rgb(28, 18, 14),
    text_muted: Color::rgb(118, 94, 82),

    code_bg: Color::rgb(20, 16, 14),
    code_fg: Color::rgb(220, 210, 202),

    font_sans: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    font_mono: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace",

    space_xs: 0.25,
    space_sm: 0.5,
    space_md: 1.0,
    space_lg: 1.5,
    space_xl: 2.5,
    radius_sm: 0.15,
    radius_md: 0.4,
    radius_lg: 0.65,
    text_xs: 0.75,
    text_sm: 0.875,
    text_md: 1.0,
    text_lg: 1.125,
    text_xl: 1.25,
    text_2xl: 1.5,
};

// ── Slate dark theme — near-neutral dark, warmth from accent not bg ───────────

pub const SLATE_DARK_THEME: Theme = Theme {
    accent: Color::rgb(196, 72, 54),
    accent_hover: Color::rgb(218, 96, 76),
    accent_fg: Color::rgb(255, 255, 255),
    accent_alpha: 0.16,

    danger: Color::rgb(220, 86, 78),
    success: Color::rgb(68, 186, 106),
    warning: Color::rgb(224, 164, 52),

    bg: Color::rgb(14, 12, 11), // R-B delta ≤ 3 — reads neutral
    surface: Color::rgb(22, 19, 18),
    surface_raised: Color::rgb(32, 28, 26),
    border: Color::rgb(58, 47, 43),

    text: Color::rgb(236, 224, 214),
    text_muted: Color::rgb(146, 118, 106),

    code_bg: Color::rgb(10, 8, 7),
    code_fg: Color::rgb(220, 210, 202),

    font_sans: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    font_mono: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace",

    space_xs: 0.25,
    space_sm: 0.5,
    space_md: 1.0,
    space_lg: 1.5,
    space_xl: 2.5,
    radius_sm: 0.15,
    radius_md: 0.4,
    radius_lg: 0.65,
    text_xs: 0.75,
    text_sm: 0.875,
    text_md: 1.0,
    text_lg: 1.125,
    text_xl: 1.25,
    text_2xl: 1.5,
};

// ── Brick light theme ─────────────────────────────────────────────────────────
//
// The whole palette is built on the brick red hue (≈12°).
// Light bg is washed terracotta; surfaces are warm parchment.

pub const BRICK_THEME: Theme = Theme {
    accent: Color::rgb(158, 40, 26), // brick red, G-B≈14 — red not orange
    accent_hover: Color::rgb(126, 24, 14),
    accent_fg: Color::rgb(255, 255, 255),
    accent_alpha: 0.14,

    danger: Color::rgb(186, 28, 28),
    success: Color::rgb(30, 130, 62),
    warning: Color::rgb(190, 124, 10),

    bg: Color::rgb(226, 186, 180), // hue≈7° — red not orange
    surface: Color::rgb(246, 218, 214),
    surface_raised: Color::rgb(236, 204, 199),
    border: Color::rgb(182, 132, 124),

    text: Color::rgb(22, 6, 4),
    text_muted: Color::rgb(98, 52, 46),

    code_bg: Color::rgb(26, 8, 6),
    code_fg: Color::rgb(230, 200, 192),

    font_sans: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    font_mono: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace",

    space_xs: 0.25,
    space_sm: 0.5,
    space_md: 1.0,
    space_lg: 1.5,
    space_xl: 2.5,
    radius_sm: 0.15,
    radius_md: 0.4,
    radius_lg: 0.65,
    text_xs: 0.75,
    text_sm: 0.875,
    text_md: 1.0,
    text_lg: 1.125,
    text_xl: 1.25,
    text_2xl: 1.5,
};

// ── Brick dark theme ──────────────────────────────────────────────────────────
//
// Backgrounds are dark brick red — the same hue, deeply darkened.
// Think of a brick wall at night, not a neutral dark with a red accent.

pub const BRICK_DARK_THEME: Theme = Theme {
    accent: Color::rgb(214, 66, 54), // hue≈7° — distinctly red, not orange
    accent_hover: Color::rgb(230, 84, 72),
    accent_fg: Color::rgb(255, 255, 255),
    accent_alpha: 0.18,

    danger: Color::rgb(216, 72, 66),
    success: Color::rgb(62, 180, 100),
    warning: Color::rgb(218, 158, 46),

    bg: Color::rgb(26, 8, 6), // hue≈8° dark brick — not orange
    surface: Color::rgb(42, 13, 10),
    surface_raised: Color::rgb(58, 19, 15),
    border: Color::rgb(92, 34, 28),

    text: Color::rgb(240, 216, 212), // warm white, hue≈10° not 22°
    text_muted: Color::rgb(164, 116, 108),

    code_bg: Color::rgb(14, 4, 3),
    code_fg: Color::rgb(230, 200, 192),

    font_sans: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    font_mono: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace",

    space_xs: 0.25,
    space_sm: 0.5,
    space_md: 1.0,
    space_lg: 1.5,
    space_xl: 2.5,
    radius_sm: 0.15,
    radius_md: 0.4,
    radius_lg: 0.65,
    text_xs: 0.75,
    text_sm: 0.875,
    text_md: 1.0,
    text_lg: 1.125,
    text_xl: 1.25,
    text_2xl: 1.5,
};

// Backward-compat aliases — DEFAULT_THEME is now the brick light variant
pub const DEFAULT_THEME: Theme = BRICK_THEME;
pub const DARK_THEME: Theme = BRICK_DARK_THEME;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Style;

    #[test]
    fn css_vars_contains_accent() {
        let css = BRICK_THEME.to_css_vars();
        assert!(
            css.contains("--brick-accent: #9e281a;"),
            "missing brick accent var"
        );
        assert!(
            css.contains("--brick-text-muted:"),
            "missing text-muted var"
        );
        assert!(css.starts_with(":root {"), "should start with :root");
        assert!(css.ends_with('}'), "should end with closing brace");
    }

    #[test]
    fn theme_set_css_includes_layer_and_dark() {
        let ts = ThemeSet {
            light: SLATE_THEME,
            dark: Some(SLATE_DARK_THEME),
            prefer_dark: false,
        };
        let css = ts.to_css();
        assert!(
            css.contains("@layer brick.reset"),
            "missing layer declaration"
        );
        assert!(
            css.contains("@layer brick.components"),
            "missing components layer"
        );
        assert!(
            css.contains("prefers-color-scheme: dark"),
            "missing dark media query"
        );
        assert!(
            css.contains(".brick-primary"),
            "missing .brick-primary class"
        );
        assert!(
            !css.contains("@layer brick.theme"),
            "vars must not be wrapped in a layer"
        );
    }

    #[test]
    fn prefer_dark_forces_dark_unconditionally() {
        let ts = ThemeSet {
            light: BRICK_THEME,
            dark: Some(BRICK_DARK_THEME),
            prefer_dark: true,
        };
        let css = ts.to_css();
        assert!(
            !css.contains("prefers-color-scheme"),
            "prefer_dark must not use any media query"
        );
        assert!(
            css.contains("--brick-accent: #d64236;"),
            "brick dark accent should be injected"
        );
    }

    #[test]
    fn color_to_hex() {
        assert_eq!(Color::rgb(158, 40, 26).to_hex(), "#9e281a");
        assert_eq!(Color::rgb(0, 0, 0).to_hex(), "#000000");
        assert_eq!(Color::rgb(255, 255, 255).to_hex(), "#ffffff");
    }

    #[test]
    fn text_scale_vars_emitted() {
        let css = DEFAULT_THEME.to_css_vars();
        assert!(css.contains("--brick-text-xs:"), "missing text-xs");
        assert!(css.contains("--brick-text-2xl:"), "missing text-2xl");
    }

    #[test]
    fn css_macro_literal_form() {
        let s = Style::from_str("margin-right: 0.5rem");
        assert_eq!(s.as_ref(), "margin-right: 0.5rem");
    }

    #[test]
    fn css_macro_brace_form() {
        let s = Style::from_str(crate::__brick_css_str!(margin-right: 0.5rem));
        assert_eq!(s.as_ref(), "margin-right: 0.5rem");
    }

    #[test]
    fn css_macro_multi_property() {
        let s = Style::from_str(crate::__brick_css_str!(padding: 1rem; color: red));
        assert_eq!(s.as_ref(), "padding: 1rem; color: red");
    }
}
