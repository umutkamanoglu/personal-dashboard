use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    radio::RadioGroup,
    v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::{
    div, px, AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _,
};

use super::{card, section_title};
use crate::app::DashboardView;
use crate::data::{DashboardData, PomodoroPhase};

const WORK_OPTIONS: [u32; 3] = [15, 25, 50];
const BREAK_MIN: u32 = 5;

pub fn pomodoro_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(680.))
        .child(section_title("Pomodoro"))
        .child(pomodoro_card(view, cx))
        .into_any_element()
}

/// Kalan süreyi mm:ss gösterir.
pub fn fmt_remaining(secs: i64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m:02}:{s:02}")
}

fn pomodoro_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>();
    let data = data.clone();

    match data.pomodoro_remaining() {
        None => start_card(view, cx),
        Some((phase, remaining, label, rounds)) => {
            let is_work = phase == PomodoroPhase::Work;
            card(cx).gap_4().child(
                v_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .font_medium()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} — {} tur tamamlandı", label, rounds)),
                    )
                    .child(div().text_xl().font_semibold().child(if is_work {
                        "Odak"
                    } else {
                        "Mola"
                    }))
                    .child(
                        div()
                            .text_3xl()
                            .font_semibold()
                            .text_color(if is_work {
                                cx.theme().primary
                            } else {
                                cx.theme().green
                            })
                            .child(fmt_remaining(remaining)),
                    )
                    .child(
                        h_flex().gap_2().child(
                            Button::new("skip-phase")
                                .small()
                                .icon(IconName::RotateCw)
                                .label(if is_work {
                                    "Molaya geç"
                                } else {
                                    "Odağa geç"
                                })
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().skip_pomodoro_phase();
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(
                        Button::new("stop-pomodoro")
                            .small()
                            .ghost()
                            .label("Seansı bitir")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.global_mut::<DashboardData>().stop_pomodoro();
                                cx.notify();
                            })),
                    ),
            )
        }
    }
}

fn start_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    card(cx).gap_3().child(
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .font_medium()
                            .text_color(cx.theme().muted_foreground)
                            .child("Odak süresi"),
                    )
                    .child(
                        RadioGroup::new("pomo-work-min")
                            .children(["15 dk", "25 dk", "50 dk"])
                            .selected_index(Some(view.pomo_work_idx))
                            .on_change(cx.listener(|this, index, _, cx| {
                                this.pomo_work_idx = *index;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Input::new(&view.pomo_label_input).flex_1())
                    .child(
                        Button::new("start-pomodoro")
                            .primary()
                            .icon(IconName::Play)
                            .label("Odak başlat")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let label =
                                    this.pomo_label_input.read(cx).value().trim().to_string();
                                let work = WORK_OPTIONS[this.pomo_work_idx];
                                cx.global_mut::<DashboardData>()
                                    .start_pomodoro(label, work, BREAK_MIN);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Her odak fazından sonra 5 dk mola otomatik başlar."),
            ),
    )
}
