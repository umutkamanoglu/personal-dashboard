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
use crate::data::{self, DashboardData, ScheduleEntry};

const DAY_LABELS: [&str; 7] = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];

pub fn schedule_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(860.))
        .child(section_title("Saatler"))
        .child(add_card(view, cx))
        .child(list_card(cx))
        .into_any_element()
}

fn add_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    card(cx).gap_3().child(
        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(Input::new(&view.sched_name_input).flex_1())
                    .child(Input::new(&view.sched_start_input).w(px(96.)))
                    .child(Input::new(&view.sched_end_input).w(px(96.))),
            )
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(day_checkboxes(view, cx))
                    .child(
                        Button::new("add-schedule")
                            .primary()
                            .icon(IconName::Plus)
                            .label("Ekle")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let name =
                                    this.sched_name_input.read(cx).value().trim().to_string();
                                if name.is_empty() {
                                    return;
                                }
                                let start = ScheduleEntry::parse_time(
                                    &this.sched_start_input.read(cx).value(),
                                );
                                let end = ScheduleEntry::parse_time(
                                    &this.sched_end_input.read(cx).value(),
                                );
                                let (Some(start), Some(end)) = (start, end) else {
                                    return;
                                };
                                let days =
                                    (0..7).filter(|d| this.sched_days[*d as usize]).collect();
                                cx.global_mut::<DashboardData>()
                                    .add_schedule(name, start, end, days);
                                cx.notify();
                            })),
                    ),
            ),
    )
}

fn day_checkboxes(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    h_flex()
        .gap_2()
        .children((0..7).map(|i| {
            Checkbox::new(SharedString::from(format!("sched-day-{}", i)))
                .label(DAY_LABELS[i])
                .checked(view.sched_days[i])
                .on_change(cx.listener(move |this, checked, _, cx| {
                    this.sched_days[i] = *checked;
                    cx.notify();
                }))
        }))
        .into_element()
}

fn list_card(cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>();
    if data.schedule.is_empty() {
        return card(cx).child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz kayıt yok. Yukarıdan saat dilimi ekle."),
        );
    }

    let today_weekday = data::weekday_of_today();
    let mut entries = data.schedule.clone();
    entries.sort_by(|a, b| a.start.cmp(&b.start));

    let mut card = card(cx);
    for entry in &entries {
        let is_today = entry.days.contains(&today_weekday);
        let id2 = entry.id.clone();
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_2()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().font_medium().child(entry.name.clone()))
                                .when(is_today, |this| {
                                    this.child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().primary)
                                            .child("bugün"),
                                    )
                                }),
                        )
                        .child(h_flex().gap_1().children(entry.days.iter().map(|d| {
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(DAY_LABELS[*d as usize])
                        }))),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} – {}", entry.start, entry.end)),
                        )
                        .child(
                            Button::new(SharedString::from(format!("rm-sched-{}", entry.id)))
                                .small()
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().remove_schedule(&id2);
                                    cx.notify();
                                })),
                        ),
                ),
        );
    }
    card
}
