mod habits;
mod routines;
mod today;

pub use habits::habits_page;
pub use routines::routines_page;
pub use today::today_page;

use gpui_kit::component::{
    ActiveTheme, button::Button, button::ButtonVariants as _, h_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _, div,
};
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Sizable as _;

use crate::app::{DashboardView, Page};

/// Styled card container.
pub fn card(cx: &gpui_kit::App) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
}

pub fn section_title(text: &'static str) -> Div {
    div()
        .font_semibold()
        .text_sm()
        .child(text)
}

/// Progress bar between done and total.
pub fn progress_bar(done: f32, total: f32, cx: &gpui_kit::App) -> AnyElement {
    let fraction = if total <= 0. { 0. } else { (done / total).clamp(0., 1.) };
    h_flex()
        .flex_1()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex_1()
                .h_1p5()
                .rounded_full()
                .overflow_hidden()
                .bg(cx.theme().muted)
                .when(fraction > 0., |this| {
                    this.child(
                        div()
                            .h_full()
                            .rounded_full()
                            .bg(cx.theme().primary)
                            .w(gpui_kit::relative(fraction)),
                    )
                }),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}/{}", done as usize, total as usize)),
        )
        .into_any_element()
}

/// Muted empty state with optional quick-action to jump to a page.
pub fn empty_state(
    text: &str,
    page: Option<Page>,
    cx: &mut Context<DashboardView>,
) -> Div {
    let weak = cx.entity().downgrade();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .py_4()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(text.to_string()),
        )
        .when_some(page, |this, page| {
            let weak = weak.clone();
            this.child(
                Button::new(format!("go-{}", format!("{:?}", page).to_lowercase()))
                    .small()
                    .ghost()
                    .label("Ekle")
                    .on_click(move |_, _, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            this.page = page;
                            cx.notify();
                        });
                    }),
            )
        })
}
