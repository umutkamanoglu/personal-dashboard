use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{
    h_flex,
    input::InputState,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex, ActiveTheme, IconName, TitleBar,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AppContext as _, Context, Entity, InteractiveElement as _, ParentElement as _,
    Styled as _,
};

use crate::data;
use crate::pages;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Schedule,
    Habits,
    Routines,
    Stats,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Home => "Anasayfa",
            Page::Schedule => "Saatler",
            Page::Habits => "Alışkanlıklar",
            Page::Routines => "Rutinler",
            Page::Stats => "İstatistik",
        }
    }

    const ALL: [Page; 5] = [
        Page::Home,
        Page::Schedule,
        Page::Habits,
        Page::Routines,
        Page::Stats,
    ];

    fn icon(self) -> IconName {
        match self {
            Page::Home => IconName::LayoutDashboard,
            Page::Schedule => IconName::Calendar,
            Page::Habits => IconName::CircleCheck,
            Page::Routines => IconName::RefreshCw,
            Page::Stats => IconName::ChartPie,
        }
    }
}

pub struct DashboardView {
    pub(crate) page: Page,
    collapsed: bool,
    // Saatler formu
    pub(crate) sched_name_input: Entity<InputState>,
    pub(crate) sched_start_input: Entity<InputState>,
    pub(crate) sched_end_input: Entity<InputState>,
    pub(crate) sched_days: [bool; 7],
    // Alışkanlık formu
    pub(crate) habit_input: Entity<InputState>,
    pub(crate) new_habit_freq: usize,
    // Rutin formu
    pub(crate) routine_input: Entity<InputState>,
}

impl DashboardView {
    pub fn new(window: &mut gpui_kit::Window, cx: &mut Context<Self>) -> Self {
        let sched_name_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Günde ne var? (ör. spor)"));
        let sched_start_input = cx.new(|cx| InputState::new(window, cx).placeholder("07:00"));
        let sched_end_input = cx.new(|cx| InputState::new(window, cx).placeholder("09:00"));

        let habit_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Yeni alışkanlık ekle..."));
        let routine_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Yeni rutin ekle..."));

        // her 30 saniyede saat/tarih güncellensin
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(30))
                .await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |_, cx| cx.notify());
            } else {
                break;
            }
        })
        .detach();

        Self {
            page: Page::Home,
            collapsed: false,
            sched_name_input,
            sched_start_input,
            sched_end_input,
            sched_days: [true; 7],
            habit_input,
            new_habit_freq: 0,
            routine_input,
        }
    }

    fn sidebar_item(&self, page: Page, cx: &mut Context<Self>) -> SidebarMenuItem {
        let weak = cx.entity().downgrade();
        SidebarMenuItem::new(page.label())
            .icon(page.icon())
            .active(self.page == page)
            .on_click(move |_, _, cx| {
                let _ = weak.update(cx, |this, cx| {
                    this.page = page;
                    cx.notify();
                });
            })
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> Sidebar<SidebarGroup<SidebarMenu>> {
        let menu = SidebarMenu::new().children(
            Page::ALL
                .iter()
                .map(|page| self.sidebar_item(*page, cx))
                .collect::<Vec<_>>(),
        );

        let collapsed = self.collapsed;
        Sidebar::new("app-sidebar")
            .collapsed(collapsed)
            .w(px(232.))
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_8()
                                .flex_shrink_0()
                                .rounded(cx.theme().radius)
                                .bg(cx.theme().primary)
                                .text_color(cx.theme().primary_foreground)
                                .child(IconName::LayoutDashboard),
                        )
                        .when(!collapsed, |this| {
                            this.child(
                                v_flex()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .child(div().text_sm().child("Personal Dashboard"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("alışkanlık takibi"),
                                    ),
                            )
                        }),
                ),
            )
            .child(SidebarGroup::new("main-group").child(menu))
            .footer(
                SidebarFooter::new().child(
                    h_flex()
                        .gap_2()
                        .child(IconName::CircleUser)
                        .when(!collapsed, |this| this.child("Umut")),
                ),
            )
    }
}

impl gpui_kit::Render for DashboardView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        let content = match self.page {
            Page::Home => pages::home_page(cx),
            Page::Schedule => pages::schedule_page(self, cx),
            Page::Habits => pages::habits_page(self, window, cx),
            Page::Routines => pages::routines_page(self, window, cx),
            Page::Stats => pages::stats_page(self, cx),
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                TitleBar::new().child(
                    h_flex()
                        .w_full()
                        .pr_2()
                        .justify_between()
                        .child("Personal Dashboard")
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(data::today_tr()),
                        ),
                ),
            )
            .child(
                h_flex().flex_1().min_h_0().child(self.sidebar(cx)).child(
                    div()
                        .id("content")
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .overflow_y_scrollbar()
                        .child(div().p_6().child(content)),
                ),
            )
    }
}
