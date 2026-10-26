use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, Div, IntoElement as _, ParentElement as _, SharedString,
    Styled as _,
};

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{self, DashboardData};

const DAY_LABELS: [&str; 7] = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];

pub fn notes_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(820.))
        .child(section_title("Notlar"))
        .child(notes_card(view, cx))
        .child(reminders_card(view, cx))
        .into_any_element()
}

fn notes_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>();
    let data = data.clone();

    let mut card = card(cx).gap_3().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(Input::new(&view.note_input).flex_1().cleanable(true))
            .child(
                Button::new("add-note")
                    .primary()
                    .icon(IconName::Plus)
                    .label("Not ekle")
                    .on_click(cx.listener(|this, _, _, cx| {
                        let text = this.note_input.read(cx).value().trim().to_string();
                        if text.is_empty() {
                            return;
                        }
                        cx.global_mut::<DashboardData>().add_note(text);
                        cx.notify();
                    })),
            ),
    );

    if data.notes.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Not yok. Yukarıya yazıp ekle."),
        );
    }

    for note in &data.notes {
        let id = note.id.clone();
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_sm()
                        .min_w_0()
                        .overflow_hidden()
                        .child(note.text.clone()),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(note.created_at.format("%d.%m").to_string()),
                        )
                        .child(
                            Button::new(SharedString::from(format!("rm-note-{}", note.id)))
                                .small()
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().remove_note(&id);
                                    cx.notify();
                                })),
                        ),
                ),
        );
    }
    card
}

fn reminders_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>();
    let data = data.clone();
    let today_wd = data::weekday_of_today();

    let mut card = card(cx).gap_3().child(
        div()
            .text_xs()
            .font_medium()
            .text_color(cx.theme().muted_foreground)
            .child("Hatırlatıcılar"),
    );

    card = card.child(
        h_flex()
            .gap_2()
            .items_center()
            .child(Input::new(&view.remind_input).flex_1())
            .child(day_checkboxes(view, cx))
            .child(
                Button::new("add-reminder")
                    .primary()
                    .icon(IconName::Bell)
                    .label("Ekle")
                    .on_click(cx.listener(|this, _, _, cx| {
                        let text = this.remind_input.read(cx).value().trim().to_string();
                        if text.is_empty() {
                            return;
                        }
                        let days: Vec<u8> = (0..7)
                            .filter(|d| this.remind_days[*d])
                            .map(|d| d as u8)
                            .collect();
                        if days.is_empty() {
                            return;
                        }
                        cx.global_mut::<DashboardData>().add_reminder(text, days);
                        cx.notify();
                    })),
            ),
    );

    if data.reminders.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Hatırlatıcı yok. Bir gün seç ve ekle."),
        );
    }

    for r in &data.reminders {
        let id = r.id.clone();
        let is_today = r.days.contains(&today_wd);
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Bell)
                        .child(div().text_sm().child(r.text.clone()))
                        .when(is_today, |this| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().primary)
                                    .child("bugün"),
                            )
                        }),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(h_flex().gap_1().children(r.days.iter().map(|d| {
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(DAY_LABELS[*d as usize])
                        })))
                        .child(
                            Button::new(SharedString::from(format!("rm-remind-{}", r.id)))
                                .small()
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().remove_reminder(&id);
                                    cx.notify();
                                })),
                        ),
                ),
        );
    }
    card
}

fn day_checkboxes(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    h_flex()
        .gap_2()
        .children((0..7).map(|i| {
            Checkbox::new(SharedString::from(format!("remind-day-{}", i)))
                .label(DAY_LABELS[i])
                .checked(view.remind_days[i])
                .on_change(cx.listener(move |this, checked, _, cx| {
                    this.remind_days[i] = *checked;
                    cx.notify();
                }))
        }))
        .into_element()
}
