use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    radio::RadioGroup,
    v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::{
    div, px, AnyElement, Context, Div, InteractiveElement as _, IntoElement as _,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
};

use super::card;
use crate::app::DashboardView;
use crate::data::DashboardData;

pub fn settings_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(720.))
        .child(div().font_semibold().text_lg().child("Ayarlar"))
        .child(profile_card(view, cx))
        .child(theme_card(cx))
        .child(pomodoro_card(cx))
        .child(visualizer_card(cx))
        .child(peaclock_card(cx))
        .child(home_layout_card(cx))
        .into_any_element()
}

pub const CARD_LABELS: [(&str, &str); 16] = [
    ("schedule", "Bugünün programı"),
    ("habits", "Alışkanlıklar"),
    ("routines", "Rutinler"),
    ("notes", "Notlar & hatırlatıcılar"),
    ("pomodoro", "Pomodoro"),
    ("system", "Sistem özeti"),
    ("cpu", "CPU"),
    ("ram", "RAM"),
    ("gpu", "GPU"),
    ("disks", "Diskler"),
    ("temps", "Sıcaklıklar"),
    ("general", "Bilgisayar"),
    ("network", "Ağ"),
    ("media", "Medya"),
    ("visualizer", "Görselleştirici"),
    ("peaclock", "Büyük saat"),
];

fn card_label(key: &str) -> &'static str {
    for (k, l) in CARD_LABELS {
        if k == key {
            return l;
        }
    }
    "Sistem"
}

/// Kullanıcının anasayfa ızgarasını özelleştirdiği kart koleksiyonu:
/// kart ekle/çıkar, sıralama ve genişlik (1 = yarım sütun, 2 = tam sütun).
fn home_layout_card(cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>().clone();
    let mut card = card(cx)
        .gap_3()
        .child(div().text_sm().font_medium().child("Anasayfa kartları"));

    // sıradaki kartlar: genişlik + sıra + çıkar
    card = card.children(data.home_order.iter().map(|key| {
        let key = key.clone();
        let span = data.home_card_span(&key);
        h_flex()
            .gap_2()
            .items_center()
            .py_1()
            .child(
                Button::new(SharedString::from(format!("home-vis-{}", key)))
                    .xsmall()
                    .ghost()
                    .icon(IconName::Close)
                    .tooltip("Kaldır")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().remove_home_card(&key);
                            cx.notify();
                        }
                    })),
            )
            .child(div().flex_1().text_sm().child(format!(
                "{}{}",
                card_label(&key),
                match span {
                    2 => " (2 sütun)",
                    3 => " (tam satır)",
                    _ => "",
                }
            )))
            .child(
                Button::new(SharedString::from(format!("home-span-{}", key)))
                    .xsmall()
                    .ghost()
                    .label(match span {
                        2 => "2 sütun",
                        3 => "3 sütun",
                        _ => "1 sütun",
                    })
                    .tooltip("Genişliği değiştir")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            let span = cx.global::<DashboardData>().home_card_span(&key);
                            let next = match span {
                                1 => 2,
                                2 => 3,
                                _ => 1,
                            };
                            cx.global_mut::<DashboardData>().set_home_span(&key, next);
                            cx.notify();
                        }
                    })),
            )
            .child(
                Button::new(SharedString::from(format!("home-up-{}", key)))
                    .xsmall()
                    .ghost()
                    .icon(IconName::ArrowUp)
                    .tooltip("Yukarı")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().move_home_card(&key, true);
                            cx.notify();
                        }
                    })),
            )
            .child(
                Button::new(SharedString::from(format!("home-down-{}", key)))
                    .xsmall()
                    .ghost()
                    .icon(IconName::ArrowDown)
                    .tooltip("Aşağı")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().move_home_card(&key, false);
                            cx.notify();
                        }
                    })),
            )
    }));

    // koleksiyondan eklenebilir kartlar: canlı önizleme, tıkla-ekle
    let available = data.home_cards_available();
    if !available.is_empty() {
        card = card.child(
            div()
                .mt_2()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Eklenebilir kartlar (eklemek için kartın üzerine tıkla):"),
        );
        // 3 sütunlu ızgara: önizleme kutuları sabit genişlikte sarar
        let weak = cx.entity().downgrade();
        card = card.child(h_flex().gap_2().mt_1().flex_wrap().items_start().children(
            available.into_iter().map(|key| {
                let key_str = key.to_string();
                let weak = weak.clone();
                v_flex()
                    .w(px(220.))
                    .gap_1()
                    .child(
                        div()
                            .id(SharedString::from(format!("home-add-{}", key)))
                            .cursor_pointer()
                            .h(px(170.))
                            .overflow_hidden()
                            .rounded_md()
                            .flex_grow(1.)
                            .child(crate::pages::card_body(key, &data, cx).h_full())
                            .on_click(move |_, _, cx| {
                                let _ = weak.update(cx, |_, cx| {
                                    cx.global_mut::<DashboardData>().add_home_card(key);
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("+ {} ekle", card_label(&key_str))),
                    )
            }),
        ));
    }

    card.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child("Geniş 2 sütunun tamamını kaplar; 1 sütun kartlar yan yana dizilir. Her satırdaki kartlar aynı uzunlukta uzar."),
    )
}

fn profile_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>().clone();
    card(cx)
        .gap_3()
        .child(div().text_sm().font_medium().child("Profil"))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Input::new(&view.name_input).flex_1())
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("şu an: {}", data.name)),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Selamlamada kullanılır. Değiştirmek için yaz ve uygula."),
        )
        .child(
            Button::new("apply-name")
                .small()
                .label("Uygula")
                .on_click(cx.listener(|this, _, _, cx| {
                    let name = this.name_input.read(cx).value().trim().to_string();
                    let data = cx.global::<DashboardData>().clone();
                    cx.global_mut::<DashboardData>().set_settings(
                        name,
                        data.work_min,
                        data.break_min,
                    );
                    cx.notify();
                })),
        )
}

fn theme_card(cx: &mut Context<DashboardView>) -> Div {
    let dark = cx.global::<DashboardData>().dark_mode;
    card(cx)
        .gap_3()
        .child(div().text_sm().font_medium().child("Tema"))
        .child(
            RadioGroup::new("theme-mode")
                .children(["Açık", "Koyu"])
                .selected_index(Some(if dark { 1 } else { 0 }))
                .on_change(cx.listener(|_, index, window, cx| {
                    let dark = *index == 1;
                    cx.global_mut::<DashboardData>().set_dark_mode(dark);
                    let mode = if dark {
                        gpui_kit::component::ThemeMode::Dark
                    } else {
                        gpui_kit::component::ThemeMode::Light
                    };
                    gpui_kit::component::Theme::change(mode, Some(window), cx);
                    cx.notify();
                })),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Tercih uygulama yeniden açıldığında korunur."),
        )
}

const WORK_OPTIONS: [u32; 3] = [15, 25, 50];
const BREAK_OPTIONS: [u32; 3] = [5, 10, 15];

fn pomodoro_card(cx: &mut Context<DashboardView>) -> Div {
    let work = cx.global::<DashboardData>().work_min;
    let break_min = cx.global::<DashboardData>().break_min;
    let work_idx = WORK_OPTIONS.iter().position(|o| *o == work).unwrap_or(1);
    let break_idx = BREAK_OPTIONS
        .iter()
        .position(|o| *o == break_min)
        .unwrap_or(0);

    card(cx)
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_medium()
                .child("Pomodoro varsayılanları"),
        )
        .child(
            h_flex()
                .gap_6()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Odak süresi"),
                        )
                        .child(
                            RadioGroup::new("settings-work")
                                .children(["15 dk", "25 dk", "50 dk"])
                                .selected_index(Some(work_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let work = WORK_OPTIONS[*index];
                                    let data = cx.global::<DashboardData>().clone();
                                    cx.global_mut::<DashboardData>().set_settings(
                                        data.name.clone(),
                                        work,
                                        data.break_min,
                                    );
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Mola süresi"),
                        )
                        .child(
                            RadioGroup::new("settings-break")
                                .children(["5 dk", "10 dk", "15 dk"])
                                .selected_index(Some(break_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let break_min = BREAK_OPTIONS[*index];
                                    let data = cx.global::<DashboardData>().clone();
                                    cx.global_mut::<DashboardData>().set_settings(
                                        data.name.clone(),
                                        data.work_min,
                                        break_min,
                                    );
                                    cx.notify();
                                })),
                        ),
                ),
        )
}

const VIZ_BAR_OPTIONS: [u8; 5] = [16, 24, 32, 48, 64];
const VIZ_COLOR_OPTIONS: [&str; 6] = [
    "accent", "primary", "success", "warning", "danger", "rainbow",
];
const VIZ_COLOR_LABELS: [&str; 6] = ["Vurgu", "Ana", "Yeşil", "Sarı", "Kırmızı", "Gökkuşağı"];

/// Görselleştirici kartını özelleştirme (CAVA tarzı ses görselleştirici):
/// bar sayısı, bar rengi, aynalanma ve tepe göstergesi.
fn visualizer_card(cx: &mut Context<DashboardView>) -> Div {
    let (bars, color, mirror, peaks) = {
        let d = cx.global::<DashboardData>();
        (d.viz_bars, d.viz_color.clone(), d.viz_mirror, d.viz_peaks)
    };
    let bar_idx = VIZ_BAR_OPTIONS.iter().position(|o| *o == bars).unwrap_or(1);
    let color_idx = VIZ_COLOR_OPTIONS
        .iter()
        .position(|o| *o == color)
        .unwrap_or(0);

    card(cx)
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_medium()
                .child("Görselleştirici görünümü"),
        )
        .child(
            h_flex()
                .gap_6()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Bar sayısı"),
                        )
                        .child(
                            RadioGroup::new("viz-bars")
                                .children(VIZ_BAR_OPTIONS.iter().map(|b| b.to_string()))
                                .selected_index(Some(bar_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let bars = VIZ_BAR_OPTIONS[*index];
                                    cx.global_mut::<DashboardData>().set_viz_bars(bars);
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Bar rengi"),
                        )
                        .child(
                            RadioGroup::new("viz-color")
                                .children(VIZ_COLOR_LABELS)
                                .selected_index(Some(color_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let color = VIZ_COLOR_OPTIONS[*index];
                                    cx.global_mut::<DashboardData>().set_viz_color(color);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            h_flex()
                .gap_4()
                .child(
                    Checkbox::new("viz-mirror")
                        .label("Aynalanmış (ortada)")
                        .checked(mirror)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_viz_mirror(*checked);
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("viz-peaks")
                        .label("Tepe göstergesi")
                        .checked(peaks)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_viz_peaks(*checked);
                            cx.notify();
                        })),
                ),
        )
}

/// Büyük saat (peaclock) kartını özelleştirme: saat biçimi, saniye,
/// yanıp sönen iki nokta, tarih satırı ve rakam rengi.
fn peaclock_card(cx: &mut Context<DashboardView>) -> Div {
    let (h24, secs, blink, date, color) = {
        let d = cx.global::<DashboardData>();
        (
            d.clock_24h,
            d.clock_seconds,
            d.clock_blink,
            d.clock_date,
            d.clock_color.clone(),
        )
    };
    let color_idx = VIZ_COLOR_OPTIONS
        .iter()
        .position(|o| *o == color)
        .unwrap_or(0);

    card(cx)
        .gap_3()
        .child(div().text_sm().font_medium().child("Büyük saat görünümü"))
        .child(
            h_flex()
                .gap_6()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Saat biçimi"),
                        )
                        .child(
                            RadioGroup::new("clock-24h")
                                .children(["24 saat", "12 saat"])
                                .selected_index(Some(if h24 { 0 } else { 1 }))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    cx.global_mut::<DashboardData>().set_clock_24h(*index == 0);
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Rakam rengi"),
                        )
                        .child(
                            RadioGroup::new("clock-color")
                                .children(VIZ_COLOR_LABELS)
                                .selected_index(Some(color_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let color = VIZ_COLOR_OPTIONS[*index];
                                    cx.global_mut::<DashboardData>().set_clock_color(color);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            h_flex()
                .gap_4()
                .child(
                    Checkbox::new("clock-seconds")
                        .label("Saniyeler")
                        .checked(secs)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_clock_seconds(*checked);
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("clock-blink")
                        .label("Yanıp sönen iki nokta")
                        .checked(blink)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_clock_blink(*checked);
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("clock-date")
                        .label("Tarih satırı")
                        .checked(date)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_clock_date(*checked);
                            cx.notify();
                        })),
                ),
        )
}
