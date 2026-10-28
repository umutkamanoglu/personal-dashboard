use gpui_kit::base::StyledExt as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::InputState,
    slider::{SliderEvent, SliderState},
    v_flex, ActiveTheme, IconName, Sizable as _, TitleBar,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, AppContext as _, Context, Entity, InteractiveElement as _,
    IntoElement as _, ParentElement as _, SharedString, StatefulInteractiveElement as _,
    Styled as _,
};

use crate::audio;
use crate::data;
use crate::hardware::HardwareInfo;
use crate::media;
use crate::media::MediaState;
use crate::pages;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Schedule,
    Habits,
    Routines,
    Notes,
    Stats,
    Pomodoro,
    Settings,
    System,
    Downloads,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Home => "Anasayfa",
            Page::Schedule => "Saatler",
            Page::Habits => "Alışkanlıklar",
            Page::Routines => "Rutinler",
            Page::Stats => "İstatistik",
            Page::Notes => "Notlar",
            Page::Pomodoro => "Pomodoro",
            Page::Settings => "Ayarlar",
            Page::System => "Sistem",
            Page::Downloads => "İndirmeler",
        }
    }

    fn icon(self) -> gpui_kit_assets::IconName {
        match self {
            Page::Home => gpui_kit_assets::IconName::LayoutDashboard,
            Page::Schedule => gpui_kit_assets::IconName::Calendar,
            Page::Habits => gpui_kit_assets::IconName::CircleCheck,
            Page::Routines => gpui_kit_assets::IconName::RefreshCw,
            Page::Stats => gpui_kit_assets::IconName::ChartPie,
            Page::Notes => gpui_kit_assets::IconName::FileText,
            Page::Pomodoro => gpui_kit_assets::IconName::Play,
            Page::Settings => gpui_kit_assets::IconName::Settings2,
            Page::System => gpui_kit_assets::IconName::Cpu,
            Page::Downloads => gpui_kit_assets::IconName::Download,
        }
    }
}

/// Menü grupları: (etiket, sayfalar)
const NAV_GROUPS: [(&str, &[Page]); 3] = [
    (
        "Genel",
        &[Page::Home, Page::System, Page::Downloads, Page::Settings],
    ),
    ("Takip", &[Page::Habits, Page::Routines, Page::Stats]),
    ("Zaman", &[Page::Schedule, Page::Notes, Page::Pomodoro]),
];

pub struct DashboardView {
    pub(crate) page: Page,
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
    // Not / hatırlatıcı formu
    pub(crate) note_input: Entity<InputState>,
    pub(crate) remind_input: Entity<InputState>,
    pub(crate) remind_days: [bool; 7],
    // Pomodoro formu
    pub(crate) pomo_label_input: Entity<InputState>,
    // Ayarlar formu
    pub(crate) name_input: Entity<InputState>,
    // İndirme sayfası: URL girişi + ses/vidyo tercihi
    pub(crate) download_url_input: Entity<InputState>,
    pub(crate) down_audio: bool,
    /// "Bilgi Al" sürerken true
    pub(crate) preparing: bool,
    /// önizleme hatası (None = sorun yok)
    pub(crate) preview_error: Option<String>,
    /// veri sıfırlama onayı (ilk tık arms, ikinci siler)
    pub(crate) reset_armed: bool,
    // Saatler timeline zoom (0.5..4)
    pub(crate) tl_zoom: f32,
    // Sol menü: grup açık mı (NAV_GROUPS sırasıyla)
    pub(crate) open_groups: [bool; 3],
    // Sol menü: simge moduna sıkıştırılmış mı
    pub(crate) sidebar_collapsed: bool,
}

impl DashboardView {
    pub fn new(window: &mut gpui_kit::Window, cx: &mut Context<Self>) -> Self {
        let sched_name_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Günde ne var? (ör. spor)"));
        let sched_start_input = cx.new(|cx| InputState::new(window, cx).placeholder("07:00"));
        let sched_end_input = cx.new(|cx| InputState::new(window, cx).placeholder("09:00"));

        let habit_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Yeni alışkanlık ekle..."));
        let note_input = cx.new(|cx| InputState::new(window, cx).placeholder("Yeni not..."));
        let remind_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Hatırlatıcı (ör. çöp çıkar)"));
        let pomo_label_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Ne üzerine odaklanıyorsun?"));
        let name_input = cx.new(|cx| {
            let mut st = InputState::new(window, cx);
            st.set_value(cx.global::<data::DashboardData>().name.clone(), window, cx);
            st
        });
        let routine_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Yeni rutin ekle..."));

        let volume_slider = cx.new(|cx| {
            let start = audio::master_volume().map(|v| v.0).unwrap_or(0.5);
            let mut st = SliderState::new().min(0.).max(1.).step(0.05);
            st.set_value(start, window, cx);
            st
        });
        cx.global_mut::<MediaState>().volume_slider = Some(volume_slider.clone());
        // slider değeri değiştikçe ana sesi ayarla
        let slider_entity = volume_slider.clone();
        cx.subscribe(&slider_entity, move |_view, _slider, event, cx| {
            let v = match *event {
                SliderEvent::Change(v) | SliderEvent::Release(v) => v.end(),
            };
            let volume = v.clamp(0., 1.);
            cx.global_mut::<MediaState>().volume = volume;
            cx.background_executor()
                .spawn(async move { audio::set_master_volume(volume) })
                .detach();
        })
        .detach();
        let download_url_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("YouTube / Instagram / Vimeo…"));

        // saniyelik tick: saat + pomodoro + donanım güncellensin
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(1))
                .await;
            if let Some(this) = this.upgrade() {
                // medya yoklaması WinRT bloklu çağrılar içerir; arka planda çalışır
                let media_state = cx
                    .background_executor()
                    .spawn(async { media::poll() })
                    .await;
                this.update(cx, |_, cx| {
                    // pomodoro fazı dolduysa otomatik geçir
                    cx.global_mut::<data::DashboardData>().advance_pomodoro();
                    // donanım: CPU/RAM her saniye, sıcaklıklar seyrek (hardware içinden)
                    cx.global_mut::<HardwareInfo>().refresh_light();
                    // slider state'i global'de kalıcı; yoklama sonucu onu sıfırlamasın
                    let keep_slider = cx.global::<media::MediaState>().volume_slider.clone();
                    *cx.global_mut::<media::MediaState>() = media_state;
                    cx.global_mut::<media::MediaState>().volume_slider = keep_slider;
                    cx.notify()
                });
            } else {
                break;
            }
        })
        .detach();

        // görselleştirici ~60 fps tick: barların akıcı görünmesi için
        // anasayfa görünürken yüksek hızda bildirir; peaclock da yanıp
        // sönen iki nokta için seyreltilmiş bildirim kullanır
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(16))
                .await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |view, cx| {
                    if view.page != Page::Home {
                        return;
                    }
                    let has_viz = cx
                        .global::<data::DashboardData>()
                        .home_order
                        .iter()
                        .any(|k| k == "visualizer");
                    let has_clock = cx
                        .global::<data::DashboardData>()
                        .home_order
                        .iter()
                        .any(|k| k == "peaclock");
                    if !has_viz && !has_clock {
                        return;
                    }
                    let viz = cx.global::<audio::VisualizerState>();
                    let bars_active =
                        has_viz && viz.snapshot(1).first().copied().unwrap_or(0.) > 0.001;
                    if bars_active || !viz.slow_tick() {
                        cx.notify();
                    }
                });
            } else {
                break;
            }
        })
        .detach();

        Self {
            page: Page::Home,
            sched_name_input,
            sched_start_input,
            sched_end_input,
            sched_days: [true; 7],
            habit_input,
            new_habit_freq: 0,
            routine_input,
            note_input,
            remind_input,
            remind_days: [false; 7],
            pomo_label_input,
            name_input,
            download_url_input,
            down_audio: false,
            preparing: false,
            preview_error: None,
            reset_armed: false,
            tl_zoom: 1.0,
            open_groups: [true; 3],
            sidebar_collapsed: false,
        }
    }

    fn sidebar_item(&self, page: Page, compact: bool, cx: &mut Context<Self>) -> AnyElement {
        let weak = cx.entity().downgrade();
        let active = self.page == page;
        let mut row = div()
            .id(SharedString::from(format!("nav-{:?}", page)))
            .cursor_pointer()
            .w_full()
            .px_3()
            .py_2()
            .rounded(cx.theme().radius)
            .flex()
            .items_center()
            .gap_2()
            .text_sm()
            .when(active, |r| {
                r.bg(cx.theme().accent)
                    .text_color(cx.theme().accent_foreground)
                    .font_medium()
            })
            .when(!active, |r| {
                r.text_color(cx.theme().muted_foreground).hover(|s| {
                    s.bg(cx.theme().accent.opacity(0.5))
                        .text_color(cx.theme().accent_foreground)
                })
            })
            .child(div().child(page.icon()))
            .on_click(move |_, _, cx| {
                let _ = weak.update(cx, |this, cx| {
                    this.page = page;
                    cx.notify();
                });
            });
        if compact {
            row = row.justify_center().px_0();
        } else {
            row = row.child(page.label().to_string());
        }
        row.into_any_element()
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        // gruplu, katlanabilir menü; her grup üst bilgisine tıklanınca açılır/kapanır.
        // sıkıştırılmış modda sadece simgeler gösterilir.
        let collapsed = self.sidebar_collapsed;
        let mut items: Vec<AnyElement> = Vec::new();
        if collapsed {
            for (_, pages) in NAV_GROUPS.iter() {
                for page in pages.iter() {
                    items.push(self.sidebar_item(*page, true, cx));
                }
            }
        } else {
            for (ix, (label, pages)) in NAV_GROUPS.iter().enumerate() {
                let open = self.open_groups[ix];
                let any_active = pages.contains(&self.page);
                let weak = cx.entity().downgrade();

                items.push(
                    div()
                        .id(SharedString::from(format!("nav-group-toggle-{}", ix)))
                        .cursor_pointer()
                        .w_full()
                        .px_3()
                        .py_2()
                        .mt(if ix == 0 { px(0.) } else { px(6.) })
                        .rounded_sm()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_medium()
                                .text_color(if any_active {
                                    cx.theme().foreground
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .flex_1()
                                .child(*label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(if open {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronRight
                                }),
                        )
                        .on_click(move |_, _, cx| {
                            let _ = weak.update(cx, |this, cx| {
                                this.open_groups[ix] = !this.open_groups[ix];
                                cx.notify();
                            });
                        })
                        .into_any_element(),
                );

                if open {
                    for page in pages.iter() {
                        items.push(self.sidebar_item(*page, false, cx));
                    }
                }
            }
        }

        // sıkıştırma düğmesi: menü başında; Stateful<Div> klonlanamadığından
        // collapsed/expanded dalları için ayrı örnek üreten yardımcı yapı
        macro_rules! toggle_btn {
            ($icon:expr) => {{
                let weak = cx.entity().downgrade();
                let id = if matches!($icon, IconName::PanelLeftOpen) {
                    "sb-collapse"
                } else {
                    "sb-expand"
                };
                div()
                    .id(SharedString::from(id))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_7()
                    .rounded_sm()
                    .text_color(cx.theme().muted_foreground)
                    .hover(|s| s.bg(cx.theme().accent.opacity(0.5)))
                    .child($icon)
                    .on_click(move |_, _, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            this.sidebar_collapsed = !this.sidebar_collapsed;
                            cx.notify();
                        });
                    })
            }};
        }

        v_flex()
            .w(if collapsed { px(64.) } else { px(232.) })
            .flex_shrink_0()
            .h_full()
            .p_3()
            .gap_1()
            .border_r_1()
            .border_color(cx.theme().border)
            .when(collapsed, |col| {
                // dar modda üstte sadece ortalanmış logo + düğme
                col.child(
                    v_flex()
                        .w_full()
                        .items_center()
                        .gap_2()
                        .pb_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_8()
                                .rounded(cx.theme().radius)
                                .bg(cx.theme().primary)
                                .text_color(cx.theme().primary_foreground)
                                .child(IconName::LayoutDashboard),
                        )
                        .child(toggle_btn!(IconName::PanelLeftOpen)),
                )
            })
            .when(!collapsed, |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .items_center()
                        .px_1()
                        .pb_3()
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
                        .child(
                            v_flex()
                                .min_w_0()
                                .overflow_hidden()
                                .flex_1()
                                .child(div().text_sm().child("Personal Dashboard"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("alışkanlık takibi"),
                                ),
                        )
                        .child(toggle_btn!(IconName::PanelLeftClose)),
                )
            })
            .children(items)
            .child(
                h_flex()
                    .mt_auto()
                    .gap_2()
                    .items_center()
                    .px_1()
                    .pt_3()
                    .child(IconName::CircleUser)
                    .when(!collapsed, |row| {
                        row.child(cx.global::<data::DashboardData>().name.clone())
                    }),
            )
            .into_any_element()
    }
}

/// Sağ üstteki ay/güneş tema düğmesi.
fn theme_button(cx: &mut Context<DashboardView>) -> Button {
    let dark = cx.theme().mode.is_dark();
    Button::new("theme-toggle")
        .small()
        .ghost()
        .icon(if dark { IconName::Sun } else { IconName::Moon })
        .tooltip(if dark { "Açık tema" } else { "Koyu tema" })
        .on_click(|_, window, cx| {
            let dark = cx.theme().mode.is_dark();
            let mode = if dark {
                gpui_kit::component::ThemeMode::Light
            } else {
                gpui_kit::component::ThemeMode::Dark
            };
            cx.global_mut::<crate::data::DashboardData>()
                .set_dark_mode(!dark);
            gpui_kit::component::Theme::change(mode, Some(window), cx);
            cx.refresh_windows();
        })
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
            Page::Notes => pages::notes_page(self, cx),
            Page::Pomodoro => pages::pomodoro_page(self, cx),
            Page::Settings => pages::settings_page(self, cx),
            Page::System => pages::system_page(self, cx),
            Page::Downloads => pages::downloads_page(self, cx),
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
                            h_flex()
                                .gap_2()
                                .items_center()
                                .occlude()
                                .child(theme_button(cx))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(data::today_tr()),
                                ),
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
                        .child(div().p_6().flex().justify_center().child(content)),
                ),
            )
    }
}
