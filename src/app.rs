use gpui_kit::component::{
    ActiveTheme, IconName, TitleBar, h_flex, input::InputState, v_flex,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::{
    AppContext as _, Context, Entity, InteractiveElement as _, ParentElement as _, Styled as _,
    px, div,
};

use crate::data;
use crate::pages;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Today,
    Habits,
    Routines,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Today => "Bugün",
            Page::Habits => "Alışkanlıklar",
            Page::Routines => "Rutinler",
        }
    }

    const ALL: [Page; 3] = [Page::Today, Page::Habits, Page::Routines];

    fn icon(self) -> IconName {
        match self {
            Page::Today => IconName::Calendar,
            Page::Habits => IconName::LayoutDashboard,
            Page::Routines => IconName::RefreshCw,
        }
    }
}

pub struct DashboardView {
    pub(crate) page: Page,
    collapsed: bool,
    pub(crate) habit_input: Entity<InputState>,
    pub(crate) new_habit_freq: usize,
    pub(crate) routine_input: Entity<InputState>,
}

impl DashboardView {
    pub fn new(window: &mut gpui_kit::Window, cx: &mut Context<Self>) -> Self {
        let habit_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Yeni alışkanlık ekle..."));
        let routine_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Yeni rutin ekle..."));
        Self {
            page: Page::Today,
            collapsed: false,
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
                                    .child(
                                        div()
                                            .text_sm()
                                            .child("Personal Dashboard"),
                                    )
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
    fn render(&mut self, window: &mut gpui_kit::Window, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        let content = match self.page {
            Page::Today => pages::today_page(cx),
            Page::Habits => pages::habits_page(self, window, cx),
            Page::Routines => pages::routines_page(self, window, cx),
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
                h_flex().flex_1().min_h_0()
                    .child(self.sidebar(cx))
                    .child(
                        div()
                            .id("content")
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .overflow_y_scrollbar()
                            .child(
                                div()
                                    .p_6()
                                    .child(content),
                            ),
                    ),
            )
    }
}
