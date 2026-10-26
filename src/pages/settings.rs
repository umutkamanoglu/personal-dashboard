use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::Button, h_flex, input::Input, radio::RadioGroup, v_flex, ActiveTheme, Sizable as _,
};
use gpui_kit::{
    div, px, AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _,
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
        .into_any_element()
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
