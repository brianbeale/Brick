use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::Signal;

static SPRING_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ── Presets (tension, friction) — from react-spring ──────────────────────────
const DEFAULT: (f64, f64) = (170.0, 26.0);
const GENTLE: (f64, f64) = (120.0, 14.0);
const WOBBLY: (f64, f64) = (180.0, 12.0);
const STIFF: (f64, f64) = (210.0, 20.0);
const SLOW: (f64, f64) = (37.0, 4.0);

// ── SpringBuilder ─────────────────────────────────────────────────────────────

pub struct SpringBuilder {
    source: Signal<f64>,
    tension: f64,
    friction: f64,
}

impl SpringBuilder {
    pub fn tension(mut self, v: f64) -> Self {
        self.tension = v;
        self
    }
    pub fn friction(mut self, v: f64) -> Self {
        self.friction = v;
        self
    }

    pub fn gentle(self) -> Signal<f64> {
        self.tension(GENTLE.0).friction(GENTLE.1).build()
    }
    pub fn wobbly(self) -> Signal<f64> {
        self.tension(WOBBLY.0).friction(WOBBLY.1).build()
    }
    pub fn stiff(self) -> Signal<f64> {
        self.tension(STIFF.0).friction(STIFF.1).build()
    }
    pub fn slow(self) -> Signal<f64> {
        self.tension(SLOW.0).friction(SLOW.1).build()
    }

    pub fn build(self) -> Signal<f64> {
        build_spring(self.source, self.tension, self.friction)
    }
}

/// Construct a [`SpringBuilder`] that follows `source` with physics-based smoothing.
pub fn spring(source: Signal<f64>) -> SpringBuilder {
    SpringBuilder {
        source,
        tension: DEFAULT.0,
        friction: DEFAULT.1,
    }
}

// ── Physics ───────────────────────────────────────────────────────────────────

pub(crate) struct SpringState {
    pub(crate) position: f64,
    pub(crate) velocity: f64,
    pub(crate) target: f64,
    pub(crate) tension: f64,
    pub(crate) friction: f64,
    pub(crate) running: bool,
    pub(crate) last_time: Option<f64>,
}

impl SpringState {
    pub(crate) fn step(&mut self, dt: f64) {
        let dx = self.position - self.target;
        let acc = -self.tension * dx - self.friction * self.velocity;
        self.velocity += acc * dt;
        self.position += self.velocity * dt;

        let settled = (self.position - self.target).abs() < 0.001 && self.velocity.abs() < 0.001;
        if settled {
            self.position = self.target;
            self.velocity = 0.0;
            self.running = false;
            self.last_time = None;
        }
    }
}

// ── Build ─────────────────────────────────────────────────────────────────────

#[cfg(brick_dom)]
fn build_spring(source: Signal<f64>, tension: f64, friction: f64) -> Signal<f64> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    let initial = source.read();
    let output = Signal::new(initial);

    let state = Rc::new(RefCell::new(SpringState {
        position: initial,
        velocity: 0.0,
        target: initial,
        tension,
        friction,
        running: false,
        last_time: None,
    }));

    // Self-referential RAF closure — receives the browser-provided timestamp (ms).
    let holder: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let holder_inner = Rc::clone(&holder);
    let state_for_raf = Rc::clone(&state);
    let output_for_raf = output.clone();

    *holder.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let (position, still_running) = {
            let mut s = state_for_raf.borrow_mut();
            let dt = match s.last_time {
                Some(last) => ((timestamp - last) / 1000.0).clamp(0.001, 0.064),
                None => 1.0 / 60.0,
            };
            s.last_time = Some(timestamp);
            s.step(dt);
            (s.position, s.running)
        };

        output_for_raf.set(position);

        if still_running {
            web_sys::window()
                .unwrap()
                .request_animation_frame(
                    holder_inner
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
        }
    }) as Box<dyn FnMut(f64)>));

    // Kick off the RAF whenever the source signal changes.
    let n = SPRING_COUNTER.fetch_add(1, Ordering::SeqCst);
    let state_for_obs = Rc::clone(&state);
    let holder_for_obs = Rc::clone(&holder);

    source.add_observer(
        &format!("spring-{}", n),
        Box::new(super::observers::Effect::new(move |&target: &f64| {
            let mut s = state_for_obs.borrow_mut();
            s.target = target;
            if !s.running {
                s.running = true;
                s.last_time = None;
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(
                        holder_for_obs
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .unwrap();
            }
        })),
    );

    std::mem::forget(holder);
    output
}

#[cfg(not(brick_dom))]
fn build_spring(source: Signal<f64>, _tension: f64, _friction: f64) -> Signal<f64> {
    Signal::new(source.read())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(pos: f64, target: f64, tension: f64, friction: f64) -> SpringState {
        SpringState {
            position: pos,
            velocity: 0.0,
            target,
            tension,
            friction,
            running: true,
            last_time: None,
        }
    }

    #[test]
    fn converges_to_target() {
        let mut s = make_state(0.0, 100.0, DEFAULT.0, DEFAULT.1);
        for _ in 0..300 {
            s.step(1.0 / 60.0);
        }
        assert!((s.position - 100.0).abs() < 0.01, "position={}", s.position);
        assert!(!s.running);
    }

    #[test]
    fn wobbly_overshoots_before_settling() {
        let mut s = make_state(0.0, 100.0, WOBBLY.0, WOBBLY.1);
        let mut peak = 0.0_f64;
        for _ in 0..120 {
            s.step(1.0 / 60.0);
            peak = peak.max(s.position);
        }
        assert!(peak > 100.0, "wobbly should overshoot; peak={}", peak);
        for _ in 0..300 {
            s.step(1.0 / 60.0);
        }
        assert!((s.position - 100.0).abs() < 0.01);
    }

    #[test]
    fn stiff_settles_faster_than_gentle() {
        let dt = 1.0 / 60.0;
        let mut stiff = make_state(0.0, 100.0, STIFF.0, STIFF.1);
        let mut gentle = make_state(0.0, 100.0, GENTLE.0, GENTLE.1);

        let stiff_frames = (0..600)
            .find(|_| {
                stiff.step(dt);
                !stiff.running
            })
            .unwrap_or(600);
        let gentle_frames = (0..600)
            .find(|_| {
                gentle.step(dt);
                !gentle.running
            })
            .unwrap_or(600);

        assert!(
            stiff_frames < gentle_frames,
            "stiff={} gentle={}",
            stiff_frames,
            gentle_frames
        );
    }

    #[test]
    fn step_at_rest_does_not_move() {
        let mut s = make_state(100.0, 100.0, DEFAULT.0, DEFAULT.1);
        s.step(1.0 / 60.0);
        assert_eq!(s.position, 100.0);
        assert!(!s.running);
    }

    #[test]
    fn builder_default_returns_source_value_in_tests() {
        let source = Signal::new(42.0_f64);
        let out = spring(source).build();
        assert_eq!(out.read(), 42.0);
    }

    #[test]
    fn builder_presets_compile() {
        let _ = spring(Signal::new(0.0_f64)).gentle();
        let _ = spring(Signal::new(0.0_f64)).wobbly();
        let _ = spring(Signal::new(0.0_f64)).stiff();
        let _ = spring(Signal::new(0.0_f64)).slow();
        let _ = spring(Signal::new(0.0_f64))
            .tension(200.0)
            .friction(18.0)
            .build();
    }
}
