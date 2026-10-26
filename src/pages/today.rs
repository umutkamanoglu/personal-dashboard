use gpui_kit::component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Div, IntoElement as _, ParentElement as _,
    SharedString, Styled as _, px, div,
};
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Sizable as _;

use super::{card, empty_state, progress_bar, section_title};
use crate::app::{DashboardView, Page};
use crate::data::{self, DashboardData, Frequency, Habit};

pub fn today_page(cx: &mut Context<DashboardView>) -> AnyElement {
    let data: DashboardData = cx.global::<DashboardData>().clone();
    let (done, total) = data.today_progress();
    let (week_done, week_total) = data.period_progress(Frequency::Weekly);
    let (month_done, month_total) = data.period_progress(Frequency::Monthly);

    let daily_habits: Vec<Habit> = data
        .habits
        .iter()
        .filter(|h| h.frequency == Frequency::Daily)
        .cloned()
        .collect();
    let weekly_habits: Vec<Habit> = data
        .habits
        .iter()
        .filter(|h| h.frequency == Frequency::Weekly)
        .cloned()
        .collect();
    let monthly_habits: Vec<Habit> = data
        .habits
        .iter()
        .filter(|h| h.frequency == Frequency::Monthly)
        .cloned()
        .collect();

    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(820.))
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xl()
                        .font_semibold()
                        .child(format!("{}, Umut", data::greeting())),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(data::today_tr()),
                ),
        )
        .child(
            card(cx)
                .gap_3()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(section_title("Günlük ilerleme"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().success)
                                .child(if total > 0 && done == total {
                                    "Tamamlandı ✓"
                                } else {
                                    "Devam ediyor"
                                }),
                        ),
                )
                .child(progress_bar(done as f32, total as f32, cx))
                .when(week_total > 0, |this| {
                    this.child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_xs().w_20().child("Bu hafta"))
                            .child(progress_bar(week_done as f32, week_total as f32, cx)),
                    )
                })
                .when(month_total > 0, |this| {
                    this.child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_xs().w_20().child("Bu ay"))
                            .child(progress_bar(month_done as f32, month_total as f32, cx)),
                    )
                }),
        )
        .child(
            v_flex()
                .gap_2()
                .child(section_title("Günlük rutinler"))
                .child(routines_card(&data, cx)),
        )
        .child(
            v_flex()
                .gap_2()
                .child(section_title("Bugünün alışkanlıkları"))
                .child(habits_card(&daily_habits, cx)),
        )
        .when(!weekly_habits.is_empty(), |this| {
            this.child(
                v_flex()
                    .gap_2()
                    .child(section_title("Bu hafta"))
                    .child(habits_card(&weekly_habits, cx)),
            )
        })
        .when(!monthly_habits.is_empty(), |this| {
            this.child(
                v_flex()
                    .gap_2()
                    .child(section_title("Bu ay"))
                    .child(habits_card(&monthly_habits, cx)),
            )
        })
        .into_any_element()
}

fn routines_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> Div {
    let _ = data;
    if cx.global::<DashboardData>().routine_items.is_empty() {
        return empty_state(
            "Henüz rutin yok. Rutinler sayfasından günlük akışını oluştur.",
            Some(Page::Routines),
            cx,
        );
    }

    let mut card = card(cx);
    let items: Vec<crate::data::RoutineItem> = cx.global::<DashboardData>().routine_items.clone();
    for item in &items {
        let checked = cx.global::<DashboardData>().routine_done(&item.id, data::today());
        let id = item.id.clone();
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_2()
                .child(
                    Checkbox::new(SharedString::from(format!("routine-check-{}", item.id)))
                        .label(item.name.clone())
                        .checked(checked)
                        .on_change(cx.listener(move |_, _checked, _, cx| {
                            cx.global_mut::<DashboardData>().toggle_routine(&id);
                            cx.notify();
                        })),
                ),
        );
    }
    card
}

fn habits_card(habits: &[Habit], cx: &mut Context<DashboardView>) -> Div {
    if habits.is_empty() {
        return empty_state(
            "Bu bölümde alışkanlık yok. Alışkanlıklar sayfasından ekleyebilirsin.",
            Some(Page::Habits),
            cx,
        );
    }

    let mut card = card(cx);
    for habit in habits {
        let (toggled, streak) = {
            let data = cx.global::<DashboardData>();
            (data.habit_done(habit, data::today()), data.habit_streak(habit))
        };
        let id = habit.id.clone();
        let button_id = format!("habit-done-{}", habit.id);
        let button = if toggled {
            Button::new(button_id)
                .small()
                .ghost()
                .icon(IconName::CircleCheck)
                .label("Tamamlandı")
        } else {
            Button::new(button_id).small().primary().label("Yapıldı")
        };
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_2()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().child(habit.name.clone()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(if streak > 0 {
                                    format!(
                                        "{} · {} {} seri",
                                        habit.frequency.label(),
                                        streak,
                                        habit.frequency.unit()
                                    )
                                } else {
                                    habit.frequency.label().to_string()
                                }),
                        ),
                )
                .child(button.on_click(cx.listener(move |_, _, _, cx| {
                    cx.global_mut::<DashboardData>().toggle_habit(&id);
                    cx.notify();
                }))),
        );
    }
    card
}
