// Hava sağlayıcılarının hızlı doğrulaması: `cargo run --example weather_check`
// include! ile src/weather.rs modülünü derler (kütüphane hedefi yok).
#![allow(dead_code, unused_imports)]

include!("../src/weather.rs");

fn main() {
    for (provider, key) in [("open-meteo", ""), ("openweathermap", "")] {
        match fetch_weather("İstanbul", provider, key) {
            Ok(w) => println!(
                "OK [{provider}] {} {}° hiss={} rüzg={:.0} nem={} saatlık={} günlük={}",
                w.place,
                w.temp,
                w.feels,
                w.wind,
                w.humidity,
                w.hourly.len(),
                w.daily.len()
            ),
            Err(e) => println!("ERR [{provider}] {e}"),
        }
    }
}
