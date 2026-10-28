use gpui_kit::base::StyledExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme, Sizable as _};
use gpui_kit::{div, px, AnyElement, Context, IntoElement as _, ParentElement as _, Styled as _};

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::DashboardData;
use crate::weather::{weather_icon, weather_text, WeatherState};

pub fn weather_page(_view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    let (data, error) = {
        let state = cx.global::<WeatherState>();
        (
            state.data.lock().ok().and_then(|d| d.clone()),
            state.error.lock().ok().and_then(|e| e.clone()),
        )
    };
    let loc = cx.global::<DashboardData>().weather_location.clone();

    let mut page = v_flex()
        .gap_4()
        .w_full()
        .max_w(px(820.))
        .child(section_title("Hava Durumu"));

    if loc.trim().is_empty() {
        page = page.child(
            card(cx).child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Konum ayarlanmadı: Ayarlar sayfasından şehir gir (ör. İstanbul). \
                         Sağlayıcı varsayılanı Open-Meteo'dur; API anahtarı gerekmez.",
                    ),
            ),
        );
        return page.into_any_element();
    }

    if let Some(err) = error {
        page = page.child(
            card(cx)
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .text_color(cx.theme().danger)
                        .child(gpui_kit::component::IconName::CircleAlert)
                        .child(err.clone()),
                )
                .child(refresh_button(cx)),
        );
    }

    match data {
        None => page = page.child(card(cx).child(div().text_sm().child("Yükleniyor…"))),
        Some(w) => {
            page = page.child(current_card(&w, cx));
            let row = |a: gpui_kit::Div, b: gpui_kit::Div| {
                h_flex()
                    .gap_4()
                    .items_stretch()
                    .child(a.flex_1().min_w_0())
                    .child(b.flex_1().min_w_0())
            };
            page = page.child(row(hourly_card(&w, cx), daily_card(&w, cx)));
        }
    }

    page.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(format!(
                "Sağlayıcı: {} — konum: {loc}",
                crate::weather::provider_label(&cx.global::<DashboardData>().weather_provider),
            )),
    )
    .into_any_element()
}

/// "Yenile" düğmesi: önbelleği öldürüp arka plan çekimi tetikler.
fn refresh_button(_cx: &mut Context<DashboardView>) -> Button {
    Button::new("weather-refresh")
        .small()
        .icon(gpui_kit_assets::IconName::RefreshCw)
        .label("Yenile")
        .on_click(|_, _, cx| {
            {
                let state = cx.global_mut::<WeatherState>();
                if let Ok(mut u) = state.updated.lock() {
                    *u = None;
                }
                if let Ok(mut e) = state.error.lock() {
                    *e = None;
                }
            }
            cx.refresh_windows();
        })
}

/// Anlık durum kartı: büyük sıcaklık + ikon + ayrıntılar.
fn current_card(w: &crate::weather::WeatherData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    card(cx)
        .gap_3()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_color(cx.theme().accent)
                                .child(weather_icon(&w.code)),
                        )
                        .child(
                            v_flex()
                                .child(
                                    div()
                                        .text_3xl()
                                        .font_semibold()
                                        .child(format!("{:.0}°", w.temp)),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(weather_text(&w.code)),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(w.place.clone()),
                ),
        )
        .child(
            h_flex()
                .gap_4()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("Hissedilen {:.0}°", w.feels))
                .child(format!("Nem {}%", w.humidity))
                .child(format!("Rüzgâr {:.0} km/s", w.wind)),
        )
        .child(refresh_button(cx))
}

/// Saatlik sıcaklık kartı: tek satırda 8 saat.
fn hourly_card(w: &crate::weather::WeatherData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let mut card =
        card(cx)
            .gap_3()
            .child(card_head(cx, gpui_kit_assets::IconName::Clock, "Saatlik"));
    for (hour, temp, code) in w.hourly.iter().take(8) {
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .text_sm()
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .text_color(cx.theme().muted_foreground)
                        .child(hour.clone()),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_color(cx.theme().accent)
                                .child(weather_icon(code)),
                        )
                        .child(format!("{:.0}°", temp)),
                ),
        );
    }
    card
}

/// 5 günlük tahmin kartı: min/max barlı satırlar.
fn daily_card(w: &crate::weather::WeatherData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let mut card = card(cx).gap_3().child(card_head(
        cx,
        gpui_kit_assets::IconName::Calendar,
        "5 günlük",
    ));
    for (day, min, max, code) in &w.daily {
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .text_sm()
                .child(day.clone())
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_color(cx.theme().accent)
                                .child(weather_icon(code)),
                        )
                        .child(format!("{:.0}° / {:.0}°", min, max)),
                ),
        );
    }
    card
}

fn card_head(
    _cx: &mut Context<DashboardView>,
    icon: gpui_kit_assets::IconName,
    label: &'static str,
) -> gpui_kit::Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .font_medium()
        .text_sm()
        .child(icon)
        .child(label)
}
