use gpui_kit::component::{
    ActiveTheme, IconName, button::Button, button::ButtonVariants as _, checkbox::Checkbox,
    h_flex, input::Input, input::InputState, v_flex,
};
use gpui_kit::{
    AppContext as _, AnyElement, Context, Div, IntoElement as _, ParentElement as _,
    SharedString, Styled as _, px, div,
};
use gpui_kit::component::Sizable as _;

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{self, DashboardData};

pub fn routines_page(
    view: &DashboardView,
    _window: &mut gpui_kit::Window,
    cx: &mut Context<DashboardView>,
) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(820.))
        .child(section_title("Günlük rutinler"))
        .child(card(cx).gap_3().child(
            h_flex()
                .items_center()
                .gap_2()
                .child(Input::new(&view.routine_input).flex_1().cleanable(true))
                .child(
                    Button::new("add-routine")
                        .primary()
                        .icon(IconName::Plus)
                        .label("Ekle")
                        .on_click(cx.listener(|this, _, window, cx| {
                            let name = this.routine_input.read(cx).value().trim().to_string();
                            if name.is_empty() {
                                return;
                            }
                            cx.global_mut::<DashboardData>().add_routine(name);
                            this.routine_input = cx.new(|cx| {
                                InputState::new(window, cx).placeholder("Yeni rutin ekle...")
                            });
                            cx.notify();
                        })),
                ),
        ))
        .child(list_card(cx))
        .into_any_element()
}

fn list_card(cx: &mut Context<DashboardView>) -> Div {
    if cx.global::<DashboardData>().routine_items.is_empty() {
        return card(cx).child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz rutin yok. Yukarıdan ekleyebilirsin."),
        );
    }

    let items = cx.global::<DashboardData>().routine_items.clone();
    let mut card = card(cx);
    for item in &items {
        let checked = cx.global::<DashboardData>().routine_done(&item.id, data::today());
        let id = item.id.clone();
        let id2 = id.clone();
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
                )
                .child(
                    Button::new(SharedString::from(format!("remove-routine-{}", item.id)))
                        .small()
                        .ghost()
                        .icon(IconName::Close)
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().remove_routine(&id2);
                            cx.notify();
                        })),
                ),
        );
    }
    card
}
