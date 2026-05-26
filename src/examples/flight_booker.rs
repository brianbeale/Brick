use crate::view_components::*;

fn is_valid_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    s[..4].parse::<u16>().is_ok()
        && s[5..7].parse::<u8>().map_or(false, |m| m >= 1 && m <= 12)
        && s[8..10].parse::<u8>().map_or(false, |d| d >= 1 && d <= 31)
}

#[model]
pub struct FlightBooker {
    #[default(false)]
    pub is_return: bool,
    #[default(String::new())]
    pub depart: String,
    #[default(String::new())]
    pub ret_date: String,
    #[default(false)]
    pub can_book: bool,
    #[default(false)]
    pub booked: bool,
    #[default(String::new())]
    pub message: String,
}

impl FlightBooker {
    fn validate(&mut self) {
        let d = self.depart.read();
        let r = self.ret_date.read();
        let is_ret = self.is_return.read();
        let depart_ok = is_valid_date(&d);
        let can = if is_ret {
            depart_ok && is_valid_date(&r) && r >= d
        } else {
            depart_ok
        };
        self.can_book.set(can);
    }
}

#[controller]
impl FlightBooker {
    pub fn one_way(&mut self) {
        self.is_return.set(false);
        self.validate();
    }

    pub fn return_trip(&mut self) {
        self.is_return.set(true);
        self.validate();
    }

    #[on(input, String)]
    pub fn update_depart(&mut self, v: String) {
        set!(self.depart => v);
        self.validate();
    }

    #[on(input, String)]
    pub fn update_ret_date(&mut self, v: String) {
        set!(self.ret_date => v);
        self.validate();
    }

    pub fn book(&mut self) {
        if !self.can_book.read() {
            return;
        }
        let d = self.depart.read();
        let r = self.ret_date.read();
        let msg = if self.is_return.read() {
            format!("Booked return flight: {} → {}.", d, r)
        } else {
            format!("Booked one-way flight on {}.", d)
        };
        self.message.set(msg);
        self.can_book.set(false);
        set!(self.booked => true);
    }
}

#[view(FlightBooker)]
fn render() -> Box<ViewComposite> {
    style! {
        .booker { display: flex; flex-direction: column; gap: 0.75rem; max-width: 300px; }
        .btn-row { display: flex; gap: 0.5rem; }
        .btn-row button { flex: 1; }
        .date-group { display: flex; flex-direction: column; gap: 0.25rem; }
        .date-group label { font-size: 0.85rem; color: var(--brick-muted); }
        .date-group input { margin-bottom: 0; }
        .booked-msg { padding: 0.6rem 0.75rem; background: color-mix(in srgb, var(--brick-accent) 15%, transparent); border-left: 3px solid var(--brick-accent); border-radius: 0.25rem; }
    }
    children! {
        h2("Flight Booker"),
        div {
            class("booker"),
            div {
                class("btn-row"),
                button("One-Way").trigger(&my.one_way).secondary(),
                button("Return").trigger(&my.return_trip).secondary(),
            },
            div {
                class("date-group"),
                label("Departure"),
                input().attr("type", "date").trigger(&my.update_depart),
            },
            when!(my.is_return,
                div {
                    class("date-group"),
                    label("Return"),
                    input().attr("type", "date").trigger(&my.update_ret_date),
                }
            ),
            when!(my.can_book,
                button("Book Flight").primary().trigger(&my.book),
            ),
            when!(my.booked,
                p(live!("{my.message}")).c("booked-msg"),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    #[test]
    fn renders_title() {
        let fb = FlightBooker { ..cascade() };
        assert!(crate::view_components::render_to_html(&*fb.into_component()).contains("Flight Booker"));
    }

    #[test]
    fn initially_one_way_mode() {
        let fb = FlightBooker { ..cascade() };
        assert!(!fb.is_return.read());
    }

    #[test]
    fn can_book_false_initially() {
        let fb = FlightBooker { ..cascade() };
        assert!(!fb.can_book.read());
    }

    #[test]
    fn booked_false_initially() {
        let fb = FlightBooker { ..cascade() };
        assert!(!fb.booked.read());
    }

    #[test]
    fn valid_date_yyyy_mm_dd() {
        assert!(is_valid_date("2026-06-15"));
        assert!(is_valid_date("2000-01-01"));
    }

    #[test]
    fn invalid_date_wrong_format() {
        assert!(!is_valid_date("2026-6-5"));
        assert!(!is_valid_date(""));
        assert!(!is_valid_date("06/15/2026"));
        assert!(!is_valid_date("20260615"));
    }

    #[test]
    fn validate_one_way_valid_date() {
        let mut fb = FlightBooker { ..cascade() };
        fb.depart.set("2026-06-15".to_string());
        fb.validate();
        assert!(fb.can_book.read());
    }

    #[test]
    fn validate_one_way_empty_date() {
        let mut fb = FlightBooker { ..cascade() };
        fb.validate();
        assert!(!fb.can_book.read());
    }

    #[test]
    fn validate_return_both_valid_depart_before_ret() {
        let mut fb = FlightBooker { ..cascade() };
        fb.is_return.set(true);
        fb.depart.set("2026-06-15".to_string());
        fb.ret_date.set("2026-06-20".to_string());
        fb.validate();
        assert!(fb.can_book.read());
    }

    #[test]
    fn validate_return_ret_before_depart() {
        let mut fb = FlightBooker { ..cascade() };
        fb.is_return.set(true);
        fb.depart.set("2026-06-20".to_string());
        fb.ret_date.set("2026-06-15".to_string());
        fb.validate();
        assert!(!fb.can_book.read());
    }

    #[test]
    fn book_sets_message_and_booked() {
        let mut fb = FlightBooker { ..cascade() };
        fb.depart.set("2026-06-15".to_string());
        fb.can_book.set(true);
        fb.book();
        assert!(fb.booked.read());
        assert!(fb.message.read().contains("2026-06-15"));
        assert!(!fb.can_book.read());
    }
}
