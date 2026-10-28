// Hava durumu: çoklu sağlayıcı desteği.
// Varsayılan sağlayıcı Open-Meteo (API anahtarı gerekmez); OpenWeatherMap
// isteğe bağlıdır (anahtar Ayarlar'dan girilir).
// İstekler engelleyici (blocking) çalışır: arka plan_executor'unda çağrılır.

use chrono::Datelike as _;
use serde_json::Value;
use std::sync::Mutex;
use std::time::Instant;

/// Uygulama genelinde hava önbelleği: sayfa açılışında tazelenir.
#[derive(Default)]
pub struct WeatherState {
    /// son başarı verisi (None = henüz çekilmedi)
    pub data: Mutex<Option<WeatherData>>,
    /// son çekim hatası
    pub error: Mutex<Option<String>>,
    /// son başarı anı (10 dk'dan eski ise yenilenir)
    pub updated: Mutex<Option<Instant>>,
}
impl gpui_kit::Global for WeatherState {}

/// Tek bir hava verisi paketi.
#[derive(Debug, Clone)]
pub struct WeatherData {
    /// çözümlenmiş yer adı (ör. "İstanbul")
    pub place: String,
    /// °C
    pub temp: f32,
    /// hissedilen °C
    pub feels: f32,
    /// km/s
    pub wind: f32,
    /// %
    pub humidity: u32,
    /// kısa durum kodu -> ikon eşlemesi için ("clear", "rain"...)
    pub code: String,
    /// gelecek saatler: (saat "14:00", sıcaklık, code)
    pub hourly: Vec<(String, f32, String)>,
    /// gelecek günler: (gün "Pzt", min, max, code)
    pub daily: Vec<(String, f32, f32, String)>,
}

/// Sağlayıcılar: (kimlik, etiket, anahtar gerektirir mi).
pub const PROVIDERS: [(&str, &str, bool); 2] = [
    ("open-meteo", "Open-Meteo (anahtar gerekmez)", false),
    ("openweathermap", "OpenWeatherMap (anahtar gerekli)", true),
];

/// Sağlayıcı kimliğini okunur etikete çevirir.
pub fn provider_label(id: &str) -> &'static str {
    PROVIDERS
        .iter()
        .find(|p| p.0 == id)
        .map(|p| p.1)
        .unwrap_or("Open-Meteo")
}

pub fn weather_icon(code: &str) -> gpui_kit_assets::IconName {
    match code {
        "clear" => gpui_kit_assets::IconName::Sun,
        "partly" => gpui_kit_assets::IconName::CloudSun,
        "cloudy" => gpui_kit_assets::IconName::Cloudy,
        "fog" => gpui_kit_assets::IconName::CloudFog,
        "drizzle" => gpui_kit_assets::IconName::CloudDrizzle,
        "rain" => gpui_kit_assets::IconName::CloudRain,
        "snow" => gpui_kit_assets::IconName::CloudSnow,
        "thunder" => gpui_kit_assets::IconName::CloudLightning,
        _ => gpui_kit_assets::IconName::Cloud,
    }
}

/// Durum kodunu insan okunur Türkçe açıklamaya çevirir.
pub fn weather_text(code: &str) -> &'static str {
    match code {
        "clear" => "Açık",
        "partly" => "Parçalı bulutlu",
        "cloudy" => "Bulutlu",
        "fog" => "Puslu",
        "drizzle" => "Sağanak",
        "rain" => "Yağmurlu",
        "snow" => "Karlı",
        "thunder" => "Gök gürültülü",
        _ => "Kapalı",
    }
}

/// Sağlayıcıdan hava verisini çeker.
pub fn fetch_weather(place: &str, provider: &str, key: &str) -> Result<WeatherData, String> {
    let place = place.trim();
    if place.is_empty() {
        return Err("Konum ayarlanmadı — Ayarlar'dan şehir gir.".into());
    }
    match provider {
        "openweathermap" => fetch_openweathermap(place, key),
        _ => fetch_open_meteo(place),
    }
}

// --- Ortak yardımcılar ---

/// HTTP GET -> JSON; hata kullanıcı okunabilir.
fn get_json(url: &str) -> Result<Value, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("personal-dashboard/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .get(url)
        .send()
        .map_err(|e| format!("bağlantı: {e}"))?;
    let status = res.status();
    let text = res.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "HTTP {status}: {}",
            text.chars().take(160).collect::<String>()
        ));
    }
    serde_json::from_str(&text).map_err(|e| format!("JSON çözümleme: {e}"))
}

/// JSON değerini f32'ye çevirir (yoksa 0).
fn f(v: &Value) -> f32 {
    v.as_f64().unwrap_or(0.) as f32
}

fn urlencode(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// "2026-10-29" gün listesinde i=0 -> "Bugün", 1 -> "Yarın", sonrası kısa gün adı.
fn day_label(day: &str, i: usize) -> String {
    match i {
        0 => return "Bugün".into(),
        1 => return "Yarın".into(),
        _ => {}
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d") {
        const DAYS: [&str; 7] = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];
        return DAYS[d.weekday().num_days_from_monday() as usize].into();
    }
    day.get(5..10).unwrap_or(day).to_string()
}

// --- Open-Meteo (anahtarsız) ---

fn fetch_open_meteo(place: &str) -> Result<WeatherData, String> {
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=tr&format=json",
        urlencode(place)
    );
    let geo = get_json(&geo_url)?;
    let hit = geo["results"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| format!("\"{place}\" bulunamadı"))?;
    let lat = f(&hit["latitude"]);
    let lon = f(&hit["longitude"]);
    let city = hit["name"].as_str().unwrap_or(place).to_string();

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}\
         &current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code\
         &hourly=temperature_2m,weather_code&daily=temperature_2m_max,temperature_2m_min,weather_code\
         &timezone=auto&forecast_days=5"
    );
    let w = get_json(&url)?;

    // "şu an" saatinin saatlik dizideki indeksini bulur
    let now_key = w["current"]["time"]
        .as_str()
        .and_then(|t| t.get(0..13))
        .map(String::from);
    let times: Vec<Option<String>> = w["hourly"]["time"]
        .as_array()
        .map(|a| a.iter().map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let now_ix = now_key
        .and_then(|key| {
            times
                .iter()
                .position(|t| t.as_deref().and_then(|s| s.get(0..13)) == Some(key.as_str()))
        })
        .unwrap_or(0);

    let hourly = (0..8)
        .filter_map(|i| {
            let ix = now_ix + i;
            let t = w["hourly"]["time"]
                .as_array()?
                .get(ix)?
                .as_str()?
                .get(11..16)?
                .to_string();
            Some((
                t,
                f(&w["hourly"]["temperature_2m"][ix]),
                wmo_code(f(&w["hourly"]["weather_code"][ix]) as i64),
            ))
        })
        .collect();

    let daily = (0..5)
        .filter_map(|i| {
            let day = w["daily"]["time"].as_array()?.get(i)?.as_str()?.to_string();
            Some((
                day_label(&day, i),
                f(&w["daily"]["temperature_2m_min"][i]),
                f(&w["daily"]["temperature_2m_max"][i]),
                wmo_code(f(&w["daily"]["weather_code"][i]) as i64),
            ))
        })
        .collect();

    let cur = &w["current"];
    Ok(WeatherData {
        place: city,
        temp: f(&cur["temperature_2m"]),
        feels: f(&cur["apparent_temperature"]),
        wind: f(&cur["wind_speed_10m"]),
        humidity: f(&cur["relative_humidity_2m"]) as u32,
        code: wmo_code(f(&cur["weather_code"]) as i64),
        hourly,
        daily,
    })
}

/// WMO kodunu kısa durum koduna çevirir.
fn wmo_code(code: i64) -> String {
    let name: &'static str = match code {
        0 => "clear",
        1 | 2 => "partly",
        3 => "cloudy",
        45 | 48 => "fog",
        51..=57 => "drizzle",
        61..=67 => "rain",
        71..=77 => "snow",
        80..=82 => "rain",
        85 | 86 => "snow",
        95..=99 => "thunder",
        _ => "cloud",
    };
    name.into()
}

// --- OpenWeatherMap (anahtarlı) ---

fn fetch_openweathermap(place: &str, key: &str) -> Result<WeatherData, String> {
    if key.trim().is_empty() {
        return Err("OpenWeatherMap için API anahtarı gerekli — Ayarlar'dan gir.".into());
    }
    let key = key.trim();
    let geo_url = format!(
        "https://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={key}",
        urlencode(place)
    );
    let geo = get_json(&geo_url)?;
    let hit = geo
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| format!("\"{place}\" bulunamadı"))?;
    let lat = f(&hit["lat"]);
    let lon = f(&hit["lon"]);
    let city = hit["name"].as_str().unwrap_or(place).to_string();

    let cur = get_json(&format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={lat}&lon={lon}&units=metric&lang=tr&appid={key}"
    ))?;
    let fc = get_json(&format!(
        "https://api.openweathermap.org/data/2.5/forecast?lat={lat}&lon={lon}&units=metric&appid={key}"
    ))?;

    let hourly = (0..8)
        .filter_map(|i| {
            let item = fc["list"].as_array()?.get(i)?;
            let t = item["dt_txt"].as_str()?.get(11..16)?.to_string();
            Some((
                t,
                f(&item["main"]["temp"]),
                owm_id(f(&item["weather"][0]["id"]) as i64),
            ))
        })
        .collect();

    // 3 saatlik öğeleri güne göre gruplayıp min/max çıkarır
    // (önce raw tarih anahtarıyla gruplanır, etiket sonda verilir)
    let mut daily: Vec<(String, f32, f32, String)> = Vec::new();
    for item in fc["list"].as_array().unwrap_or(&Vec::new()) {
        let day = item["dt_txt"]
            .as_str()
            .and_then(|d| d.get(0..10))
            .unwrap_or_default()
            .to_string();
        let temp = f(&item["main"]["temp"]);
        let code = owm_id(f(&item["weather"][0]["id"]) as i64);
        if let Some(slot) = daily.iter_mut().find(|d| d.0 == day) {
            slot.1 = slot.1.min(temp);
            slot.2 = slot.2.max(temp);
        } else if daily.len() < 5 {
            daily.push((day, temp, temp, code));
        }
    }
    let daily = daily
        .into_iter()
        .enumerate()
        .map(|(i, (day, min, max, code))| (day_label(&day, i), min, max, code))
        .collect::<Vec<_>>();

    Ok(WeatherData {
        place: city,
        temp: f(&cur["main"]["temp"]),
        feels: f(&cur["main"]["feels_like"]),
        wind: f(&cur["wind"]["speed"]) * 3.6,
        humidity: f(&cur["main"]["humidity"]) as u32,
        code: owm_id(f(&cur["weather"][0]["id"]) as i64),
        hourly,
        daily,
    })
}

/// OWM durum kimliğini kısa koda çevirir.
fn owm_id(id: i64) -> String {
    let name: &'static str = match id {
        800 => "clear",
        801 | 802 => "partly",
        803 | 804 => "cloudy",
        700..=799 => "fog",
        300..=321 => "drizzle",
        200..=299 => "thunder",
        500..=599 => "rain",
        600..=699 => "snow",
        _ => "cloud",
    };
    name.into()
}
