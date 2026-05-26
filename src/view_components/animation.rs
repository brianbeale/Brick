use super::Brick;
use crate::theme::Ms;
use crate::view_components::keyframes::KeyframeName;

// ── Easing ────────────────────────────────────────────────────────��───────────

pub enum Easing {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    StepStart,
    StepEnd,
    Custom(&'static str),
}

impl Easing {
    fn as_str(&self) -> &str {
        match self {
            Easing::Linear => "linear",
            Easing::Ease => "ease",
            Easing::EaseIn => "ease-in",
            Easing::EaseOut => "ease-out",
            Easing::EaseInOut => "ease-in-out",
            Easing::StepStart => "step-start",
            Easing::StepEnd => "step-end",
            Easing::Custom(s) => s,
        }
    }
}

// ── FillMode ──────────────────────���───────────────────────────────────────────

pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

impl FillMode {
    fn as_str(&self) -> &str {
        match self {
            FillMode::None => "none",
            FillMode::Forwards => "forwards",
            FillMode::Backwards => "backwards",
            FillMode::Both => "both",
        }
    }
}

// ── AnimationBuilder ─────────────────────────���────────────────────────────────

/// Wraps any [`Brick`] and applies a CSS animation to it.
/// Returned by `.animate()` on any element; implements [`ViewComponent`] directly.
pub struct AnimationBuilder {
    inner: Box<dyn Brick>,
    name: KeyframeName,
    duration: Ms,
    delay: Ms,
    easing: Easing,
    fill: FillMode,
    iteration_count: String,
}

impl AnimationBuilder {
    fn new(inner: Box<dyn Brick>, name: &KeyframeName) -> Self {
        Self {
            inner,
            name: name.clone(),
            duration: Ms(300.0),
            delay: Ms(0.0),
            easing: Easing::Ease,
            fill: FillMode::Both,
            iteration_count: "1".to_string(),
        }
    }

    pub fn duration(mut self: Box<Self>, d: Ms) -> Box<Self> {
        self.duration = d;
        self
    }
    pub fn delay(mut self: Box<Self>, d: Ms) -> Box<Self> {
        self.delay = d;
        self
    }
    pub fn iterations(mut self: Box<Self>, n: f32) -> Box<Self> {
        self.iteration_count = n.to_string();
        self
    }
    pub fn infinite(mut self: Box<Self>) -> Box<Self> {
        self.iteration_count = "infinite".to_string();
        self
    }

    // ── Easing presets ────────────────────────────────────────────────────────
    pub fn linear(mut self: Box<Self>) -> Box<Self> {
        self.easing = Easing::Linear;
        self
    }
    pub fn ease_in(mut self: Box<Self>) -> Box<Self> {
        self.easing = Easing::EaseIn;
        self
    }
    pub fn ease_out(mut self: Box<Self>) -> Box<Self> {
        self.easing = Easing::EaseOut;
        self
    }
    pub fn ease_in_out(mut self: Box<Self>) -> Box<Self> {
        self.easing = Easing::EaseInOut;
        self
    }
    pub fn easing(mut self: Box<Self>, e: Easing) -> Box<Self> {
        self.easing = e;
        self
    }

    // ── Fill presets ──────────────────────────────────────────────────────────
    pub fn fill_both(mut self: Box<Self>) -> Box<Self> {
        self.fill = FillMode::Both;
        self
    }
    pub fn fill_forwards(mut self: Box<Self>) -> Box<Self> {
        self.fill = FillMode::Forwards;
        self
    }
    pub fn fill_backwards(mut self: Box<Self>) -> Box<Self> {
        self.fill = FillMode::Backwards;
        self
    }
    pub fn fill(mut self: Box<Self>, f: FillMode) -> Box<Self> {
        self.fill = f;
        self
    }

    fn animation_css(&self) -> String {
        format!(
            "animation: {} {}ms {}ms {} {} {}",
            self.name.as_str(),
            self.duration.0,
            self.delay.0,
            self.easing.as_str(),
            self.fill.as_str(),
            self.iteration_count,
        )
    }
}

impl Brick for AnimationBuilder {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let wrapper = Renderer::element("div");
        Renderer::set_attr(&wrapper, "style", &self.animation_css());
        self.inner.render_into(&wrapper);
        Renderer::append(parent, &wrapper);
    }

    fn detach(&self) {
        self.inner.detach();
    }
}

// ── AnimateExt ──────────────────────────────────────────────────────��─────────

/// Adds `.animate()` to any boxed [`Brick`].
pub trait AnimateExt: Brick + Sized + 'static {
    fn animate(self: Box<Self>, name: &KeyframeName) -> Box<AnimationBuilder> {
        Box::new(AnimationBuilder::new(self, name))
    }
}

impl<T: Brick + Sized + 'static> AnimateExt for T {}

// ── stagger ───────────────────────��───────────────────────────────��───────────

/// Compute the animation delay for a list item at `index` given a per-item `step`.
///
/// ```rust,ignore
/// .delay(stagger(index, Ms(50.0)))  // item 0: 0ms, item 1: 50ms, item 2: 100ms, …
/// ```
pub fn stagger(index: usize, step: Ms) -> Ms {
    Ms(index as f32 * step.0)
}

// ── Tests ────────────────────────��────────────────────────────��───────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::{Brick, leafs::p};

    fn kf() -> KeyframeName {
        KeyframeName::generate()
    }
    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn animation_css_default() {
        let name = kf();
        let b = Box::new(AnimationBuilder::new(p("x"), &name));
        let css = b.animation_css();
        assert!(css.contains(name.as_str()));
        assert!(css.contains("300ms"));
        assert!(css.contains("0ms"));
        assert!(css.contains("ease"));
        assert!(css.contains("both"));
    }

    #[test]
    fn builder_chain_sets_params() {
        let name = kf();
        let b = Box::new(AnimationBuilder::new(p("x"), &name))
            .duration(Ms(500.0))
            .delay(Ms(100.0))
            .ease_out()
            .fill_forwards();
        let css = b.animation_css();
        assert!(css.contains("500ms"));
        assert!(css.contains("100ms"));
        assert!(css.contains("ease-out"));
        assert!(css.contains("forwards"));
    }

    #[test]
    fn html_wraps_inner_in_div() {
        let name = kf();
        let html = rh(&*Box::new(AnimationBuilder::new(p("hello"), &name)));
        assert!(html.starts_with("<div style=\"animation:"));
        assert!(html.contains("<p>hello</p>"));
        assert!(html.ends_with("</div>"));
    }

    #[test]
    fn animate_ext_available_on_any_element() {
        let name = kf();
        let html = rh(&*p("hi").animate(&name).duration(Ms(200.0)));
        assert!(html.contains("<p>hi</p>"));
        assert!(html.contains("200ms"));
    }

    #[test]
    fn stagger_computes_delay() {
        assert_eq!(stagger(0, Ms(50.0)), Ms(0.0));
        assert_eq!(stagger(1, Ms(50.0)), Ms(50.0));
        assert_eq!(stagger(3, Ms(50.0)), Ms(150.0));
    }

    #[test]
    fn stagger_zero_step() {
        assert_eq!(stagger(5, Ms(0.0)), Ms(0.0));
    }

    #[test]
    fn stagger_with_index_zero_returns_zero_delay() {
        assert_eq!(stagger(0, Ms(100.0)), Ms(0.0));
    }

    #[test]
    fn stagger_sequential_delays_increase_by_step() {
        let step = Ms(75.0);
        for i in 0..5 {
            assert_eq!(stagger(i, step), Ms(i as f32 * 75.0));
        }
    }

    #[test]
    fn infinite_iterations() {
        let name = kf();
        let css = Box::new(AnimationBuilder::new(p("x"), &name))
            .infinite()
            .animation_css();
        assert!(css.contains("infinite"));
    }
}
