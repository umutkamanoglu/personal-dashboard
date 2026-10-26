use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    radio::RadioGroup,
    v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::{
    div, px, AnyElement, Context, Div, IntoElement as _, ParentElement as _, SharedString,
    Styled as _,
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
        .child(home_layout_card(cx))
        .into_any_element()
}

const CARD_LABELS: [(&str, &str); 12] = [
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
        let wide = data.home_card_span(&key) == 2;
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
                if wide { " (geniş)" } else { "" }
            )))
            .child(
                Button::new(SharedString::from(format!("home-span-{}", key)))
                    .xsmall()
                    .ghost()
                    .label(if wide { "2 sütun" } else { "1 sütun" })
                    .tooltip("Genişliği değiştir")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            let span = cx.global::<DashboardData>().home_card_span(&key);
                            cx.global_mut::<DashboardData>()
                                .set_home_span(&key, if span == 2 { 1 } else { 2 });
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

    // koleksiyondan eklenebilir kartlar
    let available = data.home_cards_available();
    if !available.is_empty() {
        card = card.child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Eklenebilir kartlar:"),
        );
        card = card.child(
            h_flex()
                .gap_2()
                .flex_wrap()
                .children(available.iter().map(|key| {
                    Button::new(SharedString::from(format!("home-add-{}", key)))
                        .xsmall()
                        .ghost()
                        .label(format!("+ {}", card_label(key)))
                        .on_click(cx.listener({
                            let key = key.to_string();
                            move |_, _, _, cx| {
                                cx.global_mut::<DashboardData>().add_home_card(&key);
                                cx.notify();
                            }
                        }))
                })),
        );
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
