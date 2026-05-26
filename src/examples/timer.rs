use crate::view_components::*;

#[model]
pub struct Timer {
    #[default(0.0f64)]
    pub elapsed: f64,
    #[default(15.0f64)]
    pub duration: f64,
}

#[controller]
impl Timer {
    #[on(interval, 100)]
    pub fn tick(&mut self) {
        let next = (self.elapsed.read() + 0.1).min(self.duration.read());
        self.elapsed.set(next);
    }

    #[on(input, f64)]
    pub fn set_duration(&mut self, v: f64) {
        self.duration.set(v);
    }

    pub fn reset(&mut self) {
        self.elapsed.set(0.0);
    }
}

#[view(Timer)]
fn render() -> Box<ViewComposite> {
    style! {
        .timer-wrap { display: flex; flex-direction: column; gap: 0.75rem; max-width: 300px; }
        .timer-bar-track { height: 1rem; background: var(--brick-border); border-radius: 0.5rem; overflow: hidden; }
        .timer-label { font-size: 1.5rem; font-variant-numeric: tabular-nums; }
        .timer-row { display: flex; align-items: center; gap: 0.5rem; font-size: 0.9rem; }
        .timer-row label { white-space: nowrap; color: var(--brick-muted); }
        .timer-row input[type=range] { flex: 1; }
    }
    children! {
        h2("Timer"),
        div {
            class("timer-wrap"),
            div {
                class("timer-bar-track"),
                div("").c("timer-bar-fill").style(css!(
                    "height:100%;background:var(--brick-accent);transition:width 0.1s linear;width:{(my.elapsed / my.duration.max(0.001) * 100.0).min(100.0):.1}%"
                )),
            },
            p(live!("{my.elapsed:.1}s / {my.duration:.0}s")).c("timer-label"),
            div {
                class("timer-row"),
                label("Duration:"),
                input()
                    .attr("type", "range")
                    .attr("min", "1")
                    .attr("max", "60")
                    .attr("step", "0.5")
                    .attr("value", "15")
                    .trigger(&my.set_duration),
            },
            button("Reset").secondary().trigger(&my.reset),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    #[test]
    fn renders_heading() {
        let t = Timer { ..cascade() };
        assert!(crate::view_components::render_to_html(&*t.into_component()).contains("Timer"));
    }

    #[test]
    fn starts_at_zero() {
        let t = Timer { ..cascade() };
        assert_eq!(t.elapsed.read(), 0.0);
    }

    #[test]
    fn tick_increments_elapsed() {
        let mut t = Timer { ..cascade() };
        t.tick();
        assert!((t.elapsed.read() - 0.1).abs() < 1e-10);
    }

    #[test]
    fn tick_clamps_at_duration() {
        let mut t = Timer { ..cascade() };
        t.elapsed.set(14.95);
        t.tick();
        // should clamp to exactly duration (15.0)
        assert_eq!(t.elapsed.read(), 15.0);
    }

    #[test]
    fn reset_zeroes_elapsed() {
        let mut t = Timer { ..cascade() };
        t.elapsed.set(7.5);
        t.reset();
        assert_eq!(t.elapsed.read(), 0.0);
    }

    #[test]
    fn set_duration_updates_duration() {
        let mut t = Timer { ..cascade() };
        t.set_duration(30.0);
        assert_eq!(t.duration.read(), 30.0);
    }

    #[test]
    fn tick_respects_new_duration() {
        let mut t = Timer { ..cascade() };
        t.set_duration(0.05);
        t.tick();
        assert_eq!(t.elapsed.read(), 0.05);
        t.tick();
        assert_eq!(t.elapsed.read(), 0.05); // stays at new duration
    }
}
