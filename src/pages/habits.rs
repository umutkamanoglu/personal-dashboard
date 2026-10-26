use gpui_kit::component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants as _},
    h_flex, input::Input,
    input::InputState,
    radio::RadioGroup,
    v_flex,
};
use gpui_kit::{
    AppContext as _, AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _,
    div,
};
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Sizable as _;

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{DashboardData, Frequency};

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
        .child(habit_group("Günlük", &data, Frequency::Daily, cx))
        .child(habit_group("Haftalık", &data, Frequency::Weekly, cx))
        .child(habit_group("Aylık", &data, Frequency::Monthly, cx))
        .into_any_element()
}

fn add_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    card(cx).gap_3().child(
        h_flex()
            .items_center()
            .gap_2()
            .child(
                Input::new(&view.habit_input)
                    .flex_1()
                    .cleanable(true),
            )
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
                        let name = this
                            .habit_input
                            .read(cx)
                            .value()
                            .trim()
                            .to_string();
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
                            Button::new(format!("remove-habit-{}", habit.id))
                                .small()
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().remove_habit(&id);
                                    cx.notify();
                                })),
                        ),
                ),
        );
    }
    card
}
