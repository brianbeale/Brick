use crate::view_components::*;

#[model]
pub struct Thermometer {
    pub celsius: f64,
    #[from(celsius, |c| c * 9.0 / 5.0 + 32.0)]
    fahrenheit: f64,
    #[from(celsius, |c| c + 273.15)]
    kelvin: f64,
}

#[view(Thermometer)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("Temperature Converter"),
        input().attr("type", "number").attr("step", "0.1").bind(&my.celsius),
        p(live!("{my.celsius:.1} °C = {my.fahrenheit:.1} °F = {my.kelvin:.1} K")),
    }
}
