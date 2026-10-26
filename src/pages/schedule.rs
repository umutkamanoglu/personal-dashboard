use chrono::{Local, Timelike};
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, rgb, AnyElement, Context, Div, InteractiveElement as _, IntoElement as _,
    ParentElement as _, SharedString, Styled as _,
};

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{self, DashboardData, ScheduleEntry};

const DAY_LABELS: [&str; 7] = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];

/// Timeline varsayılan aralığı: 07:00–23:00
const FIRST_HOUR: i32 = 7;
const LAST_HOUR: i32 = 23;
const HOURS: i32 = LAST_HOUR - FIRST_HOUR;
const ROW_H: f32 = 44.;
const LABEL_W: f32 = 76.;
const BASE_HOUR_W: f32 = 48.;

/// Girişlerin hap renkleri.
const PALETTE: [u32; 5] = [0xF97316, 0x22C55E, 0x8B5CF6, 0x3B82F6, 0xEC4899];

/// "HH:MM" -> dakika (00:00'dan itibaren).
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

/// "HH:MM" -> timeline üzerindeki x konumu (px).
fn x_of(mins: i32, hour_w: f32) -> f32 {
    ((mins - FIRST_HOUR * 60).max(0) as f32 / 60.) * hour_w
}

pub fn schedule_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(980.))
        .child(section_title("Saatler"))
        .child(add_card(view, cx))
        .child(timeline_card(view, cx))
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

// --- Timeline ---

fn hour_labels_row(hour_w: f32, cx: &Context<DashboardView>) -> Div {
    h_flex()
        .child(div().w(px(LABEL_W)).flex_shrink_0())
        .child(h_flex().children((FIRST_HOUR..LAST_HOUR).map(|h| {
            div()
                .w(px(hour_w))
                .flex()
                .justify_center()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{h:02}:00"))
        })))
}

fn timeline_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> gpui_kit::AnyElement {
    let data = cx.global::<DashboardData>();
    let data = data.clone();
    let today_wd = data::weekday_of_today();
    let now = Local::now();
    let now_min = (now.hour() * 60 + now.minute()) as i32;

    let hour_w = BASE_HOUR_W * view.tl_zoom;
    let grid_w = HOURS as f32 * hour_w;

    // üst bar: açıklama + zoom kontrolleri
    let mut card = card(cx).gap_2().child(
        h_flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .text_xs()
                    .font_medium()
                    .text_color(cx.theme().muted_foreground)
                    .child("Fare tekerleği ile yakınlaştır / uzaklaştır"),
            )
            .child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{:.0}%", view.tl_zoom * 100.)),
                    )
                    .child(
                        Button::new("zoom-out")
                            .xsmall()
                            .ghost()
                            .label("–")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.tl_zoom = (this.tl_zoom / 1.25).clamp(0.5, 4.);
                                cx.notify();
                            })),
                    )
                    .child(Button::new("zoom-in").xsmall().ghost().label("+").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.tl_zoom = (this.tl_zoom * 1.25).clamp(0.5, 4.);
                            cx.notify();
                        }),
                    )),
            ),
    );

    card = card.child(hour_labels_row(hour_w, cx));

    for (day, label) in DAY_LABELS.iter().enumerate() {
        let is_today = day as u8 == today_wd;

        // bu günün girişleri saat sırasına dizilir
        let mut day_entries: Vec<&ScheduleEntry> = data
            .schedule
            .iter()
            .filter(|e| e.days.contains(&(day as u8)))
            .collect();
        day_entries.sort_by_key(|e| mins_of(&e.start));

        let mut row = div()
            .relative()
            .w(px(grid_w))
            .h(px(ROW_H))
            .rounded_sm()
            .when(is_today, |r| r.bg(cx.theme().muted));

        // saat çizgileri
        row = row.child(h_flex().children((FIRST_HOUR..LAST_HOUR).map(|_| {
            div()
                .w(px(hour_w))
                .h_full()
                .border_l_1()
                .border_color(cx.theme().border)
        })));

        // giriş hapları
        for (i, entry) in day_entries.into_iter().enumerate() {
            let start = mins_of(&entry.start);
            let end = mins_of(&entry.end).max(start + 30);
            let left = x_of(start, hour_w);
            let width = (x_of(end, hour_w) - left).max(28.);
            let color = PALETTE[i % PALETTE.len()];
            row = row.child(
                div()
                    .absolute()
                    .top(px(5.))
                    .h(px(ROW_H - 10.))
                    .left(px(left))
                    .w(px(width))
                    .rounded_md()
                    .bg(rgb(color))
                    .text_color(rgb(0xffffff))
                    .overflow_hidden()
                    .px_2()
                    .py_1()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_ellipsis()
                                    .child(entry.name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .child(format!("{}–{}", entry.start, entry.end)),
                            ),
                    ),
            );
        }

        // canlı "şimdi" çizgisi (bugünün satırında)
        let on_range = (FIRST_HOUR * 60..=LAST_HOUR * 60).contains(&now_min);
        if is_today && on_range {
            let x = x_of(now_min, hour_w);
            if x <= grid_w {
                row = row.child(
                    div()
                        .absolute()
                        .top(px(2.))
                        .h(px(ROW_H - 4.))
                        .left(px(x))
                        .w(px(2.))
                        .rounded_full()
                        .bg(rgb(0x3B82F6)),
                );
            }
        }

        // sağ kenar çizgisi
        row = row.child(
            div()
                .absolute()
                .top(px(0.))
                .h(px(ROW_H))
                .right(px(0.))
                .w(px(1.))
                .bg(cx.theme().border),
        );

        card = card.child(
            h_flex()
                .items_center()
                .child(
                    h_flex()
                        .w(px(LABEL_W))
                        .flex_shrink_0()
                        .gap_1()
                        .items_center()
                        .children(if is_today {
                            vec![
                                div()
                                    .w(px(6.))
                                    .h(px(6.))
                                    .rounded_full()
                                    .bg(cx.theme().primary),
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(cx.theme().primary)
                                    .child(*label),
                            ]
                        } else {
                            vec![div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(*label)]
                        }),
                )
                .child(row),
        );
    }

    // tüm timeline tek yatay scroll alanında; tekerlek zoom yapar
    div()
        .id("tl-scroll")
        .w_full()
        .overflow_x_scrollbar()
        .on_scroll(cx.listener(|this, ev: &gpui_kit::ScrollEvent, _, cx| {
            let d = match ev.delta {
                gpui_kit::ScrollDelta::Pixels(p) => f32::from(p.y),
                gpui_kit::ScrollDelta::Lines(l) => l.y,
            };
            if d.abs() > f32::EPSILON {
                this.tl_zoom = (this.tl_zoom * (1. - d * 0.002)).clamp(0.5, 4.);
                cx.notify();
            }
        }))
        .child(card)
        .into_any_element()
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
