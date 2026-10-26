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
use crate::data::{self, DashboardData, PomodoroPhase};
use chrono::{Local, Timelike};

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

fn notes_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let reminders = data.reminders_today();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Notlar & hatırlatıcılar", Page::Notes, cx));

    let mut body = v_flex().gap_1();
    if !reminders.is_empty() {
        body = body.children(reminders.iter().map(|r| {
            h_flex()
                .gap_2()
                .items_center()
                .py_1()
                .child(div().text_color(cx.theme().primary).child(IconName::Bell))
                .child(r.text.clone())
        }));
    }
    let note_text = if data.notes.is_empty() {
        "Not yok.".to_string()
    } else {
        format!("{} not", data.notes.len())
    };
    if reminders.is_empty() {
        body = body.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Bugün hatırlatıcı yok."),
        );
    }
    body = body.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(note_text),
    );
    card.child(body)
}

pub fn home_page(cx: &mut Context<DashboardView>) -> AnyElement {
    let data: DashboardData = cx.global::<DashboardData>().clone();
    let now = chrono::Local::now();

    v_flex()
        .gap_5()
        .w_full()
        .max_w(px(900.))
        // başlık: selamlama solda, büyük saat sağda
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().font_semibold().text_xl().child(format!(
                            "{}, {}",
                            data::greeting(),
                            data.name
                        )))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(data::today_tr()),
                        ),
                )
                .child(
                    v_flex().gap_0().items_end().child(
                        div()
                            .font_semibold()
                            .text_2xl()
                            .child(now.format("%H:%M").to_string()),
                    ),
                ),
        )
        // 2 sütunlu kart grid'i: tüm kartlar eşit genişlikte
        .child(
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_3()
                        .items_stretch()
                        .child(schedule_card(&data, cx).flex_1().min_h(px(120.)))
                        .child(habits_card(&data, cx).flex_1().min_h(px(120.))),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .items_stretch()
                        .child(routines_card(&data, cx).flex_1().min_h(px(120.)))
                        .child(notes_card(&data, cx).flex_1().min_h(px(120.))),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .items_stretch()
                        .child(pomodoro_home_card(&data, cx).flex_1().min_h(px(120.)))
                        .child(clock_card_extra(cx).flex_1().min_h(px(120.))),
                ),
        )
        .into_any_element()
}

/// Saat kartı yerine grid'i dolduran "bugün" özet kartı.
fn clock_card_extra(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let data = cx.global::<DashboardData>();
    let data = data.clone();
    let entries = data.schedule_today();
    let now = Local::now();
    let now_min: i32 = (now.hour() * 60 + now.minute()) as i32;
    let next = entries.iter().find(|e| mins_of(&e.end) > now_min);

    card(cx).gap_2().child(
        v_flex()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_medium()
                    .text_color(cx.theme().muted_foreground)
                    .child("Şimdi / sırada"),
            )
            .child(match next {
                Some(e) => v_flex()
                    .gap_0()
                    .child(div().font_semibold().text_sm().child(e.name.clone()))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} – {}", e.start, e.end)),
                    ),
                None => div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Sırada program yok."),
            }),
    )
}

/// "HH:MM" -> dakika.
fn mins_of(t: &str) -> i32 {
    let (h, m) = t
        .split_once(':')
        .map(|(h, m)| {
            (
                h.trim().parse::<i32>().unwrap_or(0),
                m.trim().parse::<i32>().unwrap_or(0),
            )
        })
        .unwrap_or((0, 0));
    h * 60 + m
}

/// Aktif pomodoro seansını anasayfada gösterir.
fn pomodoro_home_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Pomodoro", Page::Pomodoro, cx));
    match data.pomodoro_remaining() {
        None => {
            card = card.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .py_2()
                    .child("Aktif seans yok."),
            );
        }
        Some((phase, remaining, label, rounds)) => {
            let is_work = phase == PomodoroPhase::Work;
            card = card.child(
                div()
                    .text_2xl()
                    .font_semibold()
                    .text_color(if is_work {
                        cx.theme().primary
                    } else {
                        cx.theme().success
                    })
                    .child(super::pomodoro::fmt_remaining(remaining)),
            );
            card = card.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(if is_work {
                        format!("Odak — {}", label)
                    } else {
                        "Mola".to_string()
                    }),
            );
            card = card.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} tur tamamlandı", rounds)),
            );
        }
    }
    card
}
