use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    input::InputState,
    radio::RadioGroup,
    v_flex, ActiveTheme, IconName,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, AnyElement, AppContext as _, Context, Div, IntoElement as _, ParentElement as _,
    Styled as _,
};

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{DashboardData, Frequency};
use chrono::{Datelike as _, Duration, NaiveDate};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::px;

pub fn habits_page(
    view: &DashboardView,
    _window: &mut gpui_kit::Window,
    cx: &mut Context<DashboardView>,
) -> AnyElement {
    let data: DashboardData = cx.global::<DashboardData>().clone();

    v_flex()
        .gap_4()
        .w_full()
        .max_w(gpui_kit::px(820.))
        .child(section_title("Alışkanlıklar"))
        .child(add_card(view, cx))
        .child(habit_group("Günlük", &data, Frequency::Daily, view, cx))
        .child(habit_group("Haftalık", &data, Frequency::Weekly, view, cx))
        .child(habit_group("Aylık", &data, Frequency::Monthly, view, cx))
        .when_some(
            data.habits
                .iter()
                .find(|h| h.id == view.heat_habit.as_deref().unwrap_or_default())
                .cloned(),
            |p, habit| p.child(heatmap_card(&data, &habit, view, cx)),
        )
        .into_any_element()
}

fn add_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    card(cx).gap_3().child(
        h_flex()
            .items_center()
            .gap_2()
            .child(Input::new(&view.habit_input).flex_1().cleanable(true))
            .child(
                RadioGroup::new("new-habit-freq")
                    .children(["Günlük", "Haftalık", "Aylık"])
                    .selected_index(Some(view.new_habit_freq))
                    .on_change(cx.listener(|this, index, _, cx| {
                        this.new_habit_freq = *index;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("add-habit")
                    .primary()
                    .icon(IconName::Plus)
                    .label("Ekle")
                    .on_click(cx.listener(|this, _, window, cx| {
                        let name = this.habit_input.read(cx).value().trim().to_string();
                        if name.is_empty() {
                            return;
                        }
                        let frequency = Frequency::from_index(this.new_habit_freq);
                        cx.global_mut::<DashboardData>().add_habit(name, frequency);
                        this.habit_input = cx.new(|cx| {
                            InputState::new(window, cx).placeholder("Yeni alışkanlık ekle...")
                        });
                        cx.notify();
                    })),
            ),
    )
}

fn habit_group(
    title: &'static str,
    data: &crate::data::DashboardData,
    frequency: Frequency,
    view: &DashboardView,
    cx: &mut Context<DashboardView>,
) -> Div {
    let mut card = card(cx).child(
        div()
            .text_xs()
            .font_medium()
            .text_color(cx.theme().muted_foreground)
            .child(title),
    );

    for habit in &data.habits {
        if habit.frequency != frequency {
            continue;
        }
        let streak = data.habit_streak(habit);
        let id = habit.id.clone();
        let id2 = id.clone();
        let id3 = id.clone();
        let done = data.habit_done(habit, crate::data::today());
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_2()
                .child(habit.name.clone())
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(if streak > 0 {
                                    format!("{} {} seri", streak, frequency.unit())
                                } else {
                                    String::new()
                                }),
                        )
                        .child(
                            Button::new(format!("toggle-habit-{}", habit.id))
                                .small()
                                .when(done, |b| b.primary())
                                .label(if done { "Yapıldı" } else { "Yap" })
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().toggle_habit(&id);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new(format!("heat-habit-{}", habit.id))
                                .small()
                                .ghost()
                                .when(Some(habit.id.clone()) == view.heat_habit, |b| {
                                    b.text_color(cx.theme().accent)
                                })
                                .icon(gpui_kit_assets::IconName::Flame)
                                .tooltip("Isı haritası")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    // aynı buton: aç/kapat
                                    this.heat_habit =
                                        if this.heat_habit.as_deref() == Some(id3.as_str()) {
                                            None
                                        } else {
                                            Some(id3.clone())
                                        };
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new(format!("remove-habit-{}", habit.id))
                                .small()
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().remove_habit(&id2);
                                    cx.notify();
                                })),
                        ),
                ),
        );
    }
    card
}

/// GitHub tarzı ısı haritası: son 26 hafta, Pzt–Paz sütunlar; tamamlanan
/// tarih hücresi renklenir (haftalık/aylık alışkanlıkta tüm dönem dolgu).
fn heatmap_card(
    data: &DashboardData,
    habit: &crate::data::Habit,
    _view: &DashboardView,
    cx: &mut Context<DashboardView>,
) -> Div {
    const WEEKS: i64 = 26;
    let today = crate::data::today();
    // bugünü içeren haftanın pazartesisine hizala
    let today_monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let start = today_monday - Duration::weeks(WEEKS - 1);

    let theme = cx.theme();
    let cell = |date: NaiveDate| -> gpui_kit::Div {
        let done = data.habit_done(habit, date);
        div()
            .size(px(9.))
            .rounded_xs()
            .when(done, |c| c.bg(theme.success))
            .when(!done, |c| c.bg(theme.accent.opacity(0.25)))
    };

    // sütunlar: her sütun bir hafta (Pzt..Paz)
    let mut grid = h_flex().gap(px(2.)).items_start().overflow_x_scrollbar();
    for w in 0..WEEKS {
        let mut col = v_flex().gap(px(2.));
        for d in 0..7 {
            col = col.child(cell(start + Duration::weeks(w) + Duration::days(d)));
        }
        grid = grid.child(col);
    }

    let total = data
        .habit_completions
        .get(&habit.id)
        .map(|s| s.len())
        .unwrap_or(0);
    let streak = data.habit_streak(habit);

    card(cx)
        .gap_3()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .font_semibold()
                        .text_sm()
                        .child(gpui_kit_assets::IconName::Flame)
                        .child(format!(
                            "{} — ısı haritası",
                            if habit.name.chars().count() > 24 {
                                format!("{}…", habit.name.chars().take(24).collect::<String>())
                            } else {
                                habit.name.clone()
                            }
                        )),
                )
                .child(
                    Button::new("heat-close")
                        .small()
                        .ghost()
                        .icon(IconName::Close)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.heat_habit = None;
                            cx.notify();
                        })),
                ),
        )
        .child(grid)
        .child(
            h_flex()
                .gap_3()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("son {} hafta", WEEKS))
                .child(format!("{} tamamlama", total))
                .child(format!("{} {} seri", streak, habit.frequency.unit())),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "Renkli hücre = o gün tamamlanmış. Haftalık/aylık alışkanlıkta \
                     dönem dolgusu tüm haftayı/ayı boyar.",
                ),
        )
}
