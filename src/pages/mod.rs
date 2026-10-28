mod habits;
mod home;
mod notes;
mod pomodoro;
mod routines;
mod schedule;
mod settings;
mod stats;
mod system;

mod downloads;

pub use downloads::downloads_page;
pub use habits::habits_page;
pub use home::{card_body, home_page};
pub use notes::notes_page;
pub use pomodoro::pomodoro_page;
pub use routines::routines_page;
pub use schedule::schedule_page;
pub use settings::settings_page;
pub use stats::stats_page;
pub use system::system_page;

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{h_flex, ActiveTheme};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{div, AnyElement, Div, IntoElement as _, ParentElement as _, Styled as _};

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
    div().font_semibold().text_sm().child(text)
}

/// Progress bar between done and total.
pub fn progress_bar(done: f32, total: f32, cx: &gpui_kit::App) -> AnyElement {
    let fraction = if total <= 0. {
        0.
    } else {
        (done / total).clamp(0., 1.)
    };
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
