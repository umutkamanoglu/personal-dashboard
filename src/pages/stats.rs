use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _,
};

use chrono::Datelike as _;

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{DashboardData, Frequency};

const DAY_LABELS: [&str; 7] = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];

pub fn stats_page(_view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    let data: DashboardData = cx.global::<DashboardData>().clone();

    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(820.))
        .child(section_title("İstatistik"))
        .child(last_days_card(&data, cx))
        .child(routine_week_card(&data, cx))
        .children(habit_cards(&data, cx))
        .into_any_element()
}

/// Son 7 gün: günlük alışkanlık + rutin tamamlama oranı sütun grafiği.
fn last_days_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> Div {
    let history = data.daily_history(7);
    let any_total = history.iter().any(|(_, _, total)| *total > 0);

    let card = card(cx).gap_3().child(
        div()
            .text_xs()
            .font_medium()
            .text_color(cx.theme().muted_foreground)
            .child("Son 7 gün (günlük alışkanlık + rutin)"),
    );

    if !any_total {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Grafik için önce günlük alışkanlık veya rutin ekle."),
        );
    }

    let card = card.child(
        h_flex()
            .gap_3()
            .items_end()
            .children(history.into_iter().map(|(date, done, total)| {
                let fraction = if total == 0 {
                    0.
                } else {
                    done as f32 / total as f32
                };
                let weekday = date.weekday().num_days_from_monday() as usize;
                v_flex()
                    .flex_1()
                    .gap_1()
                    .items_center()
                    .child(
                        // sütun alanı: sabit yükseklik, bar alttan büyür
                        div()
                            .h(px(110.))
                            .w_full()
                            .flex()
                            .items_end()
                            .justify_center()
                            .child(
                                div()
                                    .w(px(28.))
                                    .rounded_t(cx.theme().radius)
                                    .bg(cx.theme().primary)
                                    .when(fraction <= 0., |b| b.bg(cx.theme().muted))
                                    .h(gpui_kit::relative(fraction.max(0.02))),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} / {}", done, total)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(DAY_LABELS[weekday]),
                    )
            })),
    );
    card
}

/// Bu haftanın rutin durumu.
fn routine_week_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> Div {
    let mut card = card(cx).gap_2().child(
        div()
            .text_xs()
            .font_medium()
            .text_color(cx.theme().muted_foreground)
            .child("Rutinler"),
    );

    if data.routine_items.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz rutin yok."),
        );
    }

    for item in &data.routine_items {
        let week = data.routine_week_count(&item.id, 8);
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_1()
                .child(item.name.clone())
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("son 8 haftanın {} günü yapıldı", week)),
                ),
        );
    }
    card
}

/// Her alışkanlık için son dönemlerin nokta görünümü ve tamamlama oranı.
fn habit_cards(data: &DashboardData, cx: &mut Context<DashboardView>) -> Vec<Div> {
    data.habits
        .iter()
        .map(|habit| {
            let periods = match habit.frequency {
                Frequency::Daily => 14,
                Frequency::Weekly => 8,
                Frequency::Monthly => 6,
            };
            let history = data.habit_history(habit, periods);
            let hits = history.iter().filter(|(_, done)| *done).count();
            let rate = if periods == 0 {
                0.
            } else {
                hits as f32 / periods as f32 * 100.
            };

            let mut card = card(cx).gap_3().child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(div().font_medium().text_sm().child(habit.name.clone()))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{rate:.0}%")),
                    ),
            );

            card = card.child(h_flex().gap_1().flex_wrap().children(history.iter().map(
                |(_, done)| {
                    div()
                        .size_3()
                        .rounded_sm()
                        .when(*done, |d| d.bg(cx.theme().primary))
                        .when(!done, |d| d.bg(cx.theme().muted))
                },
            )));

            let unit = habit.frequency.unit();
            card.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("son {periods} {unit} döneminde {hits} tamamlama")),
            )
        })
        .collect()
}
