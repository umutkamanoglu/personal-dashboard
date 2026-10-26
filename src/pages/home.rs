use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex, v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::{
    div, px, AnyElement, Context, IntoElement as _, ParentElement as _, SharedString, Styled as _,
};

use super::{card, progress_bar};
use crate::app::{DashboardView, Page};
use crate::data::{self, DashboardData};
use chrono::Local;

fn header_button(page: Page, cx: &mut Context<DashboardView>) -> Button {
    let weak = cx.entity().downgrade();
    Button::new(SharedString::from(format!("home-open-{:?}", page)))
        .xsmall()
        .ghost()
        .icon(IconName::ChevronRight)
        .on_click(move |_, _, cx| {
            let _ = weak.update(cx, |this, cx| {
                this.page = page;
                cx.notify();
            });
        })
}

fn card_row(title: &str, page: Page, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    h_flex()
        .justify_between()
        .items_center()
        .child(div().font_semibold().text_sm().child(title.to_string()))
        .child(header_button(page, cx))
}

fn clock_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let now = Local::now();
    card(cx)
        .gap_2()
        .child(
            div()
                .font_semibold()
                .text_xl()
                .child(now.format("%H:%M").to_string()),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(data::today_tr()),
        )
}

fn schedule_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let entries = data.schedule_today();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Bugünün programı", Page::Schedule, cx));

    if entries.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Bugün için program yok."),
        );
    }
    for entry in entries.iter().take(4) {
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_1()
                .child(entry.name.clone())
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} – {}", entry.start, entry.end)),
                ),
        );
    }
    let more = entries.len().saturating_sub(4);
    if more > 0 {
        card = card.child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("… ve {} giriş daha", more)),
        );
    }
    card
}

fn habits_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (done, total) = data.today_progress();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Günlük alışkanlıklar", Page::Habits, cx));
    if total == 0 {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz alışkanlık yok."),
        );
    }
    card = card.child(progress_bar(done as f32, total as f32, cx));
    for habit in &data.habits {
        let streak = data.habit_streak(habit);
        if streak > 0 {
            card = card.child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .py_1()
                    .child(habit.name.clone())
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} {} seri", streak, habit.frequency.unit())),
                    ),
            );
        }
    }
    card
}

fn routines_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (done, total) = data.today_progress();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Günlük rutinler", Page::Routines, cx));
    if data.routine_items.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz rutin yok."),
        );
    }
    card = card.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(format!("{} / {} tamamlandı", done, total)),
    );
    for item in data.routine_items.iter().take(4) {
        let checked = data.routine_done(&item.id, data::today());
        let id = item.id.clone();
        card = card.child(
            h_flex().justify_between().items_center().py_1().child(
                Checkbox::new(SharedString::from(format!("home-routine-{}", item.id)))
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

pub fn home_page(cx: &mut Context<DashboardView>) -> AnyElement {
    let data: DashboardData = cx.global::<DashboardData>().clone();

    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(900.))
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .font_semibold()
                        .text_xl()
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
            h_flex()
                .flex_wrap()
                .gap_3()
                .items_start()
                .child(clock_card(cx).w(px(220.)))
                .child(schedule_card(&data, cx).w(px(320.)))
                .child(habits_card(&data, cx).w(px(320.)))
                .child(routines_card(&data, cx).w(px(320.))),
        )
        .into_any_element()
}
