use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex, v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, InteractiveElement as _, IntoElement as _, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _,
};

use super::{card, progress_bar};
use crate::app::{DashboardView, Page};
use crate::data::{self, DashboardData, PomodoroPhase};
use crate::hardware::HardwareInfo;
use chrono::Local;

fn header_button(page: Page, cx: &mut Context<DashboardView>) -> Button {
    let weak = cx.entity().downgrade();
    Button::new(SharedString::from(format!("home-open-{:?}", page)))
        .xsmall()
        .ghost()
        .icon(IconName::ChevronRight)
        .on_click(move |_, _, cx| {
            let _ = weak.update(cx, |this, cx| {
                this.page = page;
                cx.notify();
            });
        })
}

fn card_row(title: &str, page: Page, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    h_flex()
        .justify_between()
        .items_center()
        .child(div().font_semibold().text_sm().child(title.to_string()))
        .child(header_button(page, cx))
}

fn schedule_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let entries = data.schedule_today();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Bugünün programı", Page::Schedule, cx));

    if entries.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Bugün için program yok."),
        );
    }
    for entry in entries.iter().take(4) {
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .py_1()
                .child(entry.name.clone())
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} – {}", entry.start, entry.end)),
                ),
        );
    }
    let more = entries.len().saturating_sub(4);
    if more > 0 {
        card = card.child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("… ve {} giriş daha", more)),
        );
    }
    card
}

fn habits_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (done, total) = data.today_progress();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Günlük alışkanlıklar", Page::Habits, cx));
    if total == 0 {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz alışkanlık yok."),
        );
    }
    card = card.child(progress_bar(done as f32, total as f32, cx));
    for habit in &data.habits {
        let streak = data.habit_streak(habit);
        if streak > 0 {
            card = card.child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .py_1()
                    .child(habit.name.clone())
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} {} seri", streak, habit.frequency.unit())),
                    ),
            );
        }
    }
    card
}

fn routines_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (done, total) = data.today_progress();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Günlük rutinler", Page::Routines, cx));
    if data.routine_items.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Henüz rutin yok."),
        );
    }
    card = card.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(format!("{} / {} tamamlandı", done, total)),
    );
    for item in data.routine_items.iter().take(4) {
        let checked = data.routine_done(&item.id, data::today());
        let id = item.id.clone();
        card = card.child(
            h_flex().justify_between().items_center().py_1().child(
                Checkbox::new(SharedString::from(format!("home-routine-{}", item.id)))
                    .label(item.name.clone())
                    .checked(checked)
                    .on_change(cx.listener(move |_, _checked, _, cx| {
                        cx.global_mut::<DashboardData>().toggle_routine(&id);
                        cx.notify();
                    })),
            ),
        );
    }
    card
}

fn notes_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let reminders = data.reminders_today();
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Notlar & hatırlatıcılar", Page::Notes, cx));

    let mut body = v_flex().gap_1();
    if !reminders.is_empty() {
        body = body.children(reminders.iter().map(|r| {
            h_flex()
                .gap_2()
                .items_center()
                .py_1()
                .child(div().text_color(cx.theme().primary).child(IconName::Bell))
                .child(r.text.clone())
        }));
    }
    let note_text = if data.notes.is_empty() {
        "Not yok.".to_string()
    } else {
        format!("{} not", data.notes.len())
    };
    if reminders.is_empty() {
        body = body.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Bugün hatırlatıcı yok."),
        );
    }
    body = body.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(note_text),
    );
    card.child(body)
}

pub fn home_page(cx: &mut Context<DashboardView>) -> AnyElement {
    let data: DashboardData = cx.global::<DashboardData>().clone();
    let now = Local::now();

    v_flex()
        .gap_5()
        .w_full()
        .max_w(px(900.))
        // başlık: selamlama solda, büyük saat sağda
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().font_semibold().text_xl().child(format!(
                            "{}, {}",
                            data::greeting(),
                            data.name
                        )))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(data::today_tr()),
                        ),
                )
                .child(
                    v_flex().gap_0().items_end().child(
                        div()
                            .font_semibold()
                            .text_2xl()
                            .child(now.format("%H:%M").to_string()),
                    ),
                ),
        )
        // kart grid'i: kullanıcının Ayarlar'dan seçtiği sıra + genişlik
        // (span 1 = yarım sütun, span 2 = tam sütun); satır içi kartlar
        // eşit yükseklikte (h_full) uzar
        .child(
            v_flex().gap_3().children(
                pack_rows(
                    data.home_cards_visible()
                        .into_iter()
                        .map(|k| (k, data.home_card_span(k)))
                        .collect(),
                )
                .into_iter()
                .map(|row| {
                    h_flex()
                        .gap_3()
                        .items_stretch()
                        .children(row.iter().map(|key| {
                            let page = home_key_page(key);
                            let body = card_body(key, &data, cx);
                            let wide = data.home_card_span(key) == 2;
                            clickable(body, page, wide, cx)
                        }))
                }),
            ),
        )
        .into_any_element()
}

/// Kart anahtarı -> sayfa.
fn home_key_page(key: &str) -> Page {
    match key {
        "schedule" => Page::Schedule,
        "habits" => Page::Habits,
        "routines" => Page::Routines,
        "notes" => Page::Notes,
        "pomodoro" => Page::Pomodoro,
        _ => Page::System,
    }
}

/// Kart koleksiyonu: anahtar -> içeriği üreten fonksiyon.
fn card_body(key: &str, data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    match key {
        "schedule" => schedule_card(data, cx),
        "habits" => habits_card(data, cx),
        "routines" => routines_card(data, cx),
        "notes" => notes_card(data, cx),
        "pomodoro" => pomodoro_home_card(data, cx),
        "system" => system_home_card(data, cx),
        "cpu" => cpu_home_card(cx),
        "ram" => ram_home_card(cx),
        "gpu" => gpu_home_card(cx),
        "disks" => disks_home_card(cx),
        "temps" => temps_home_card(cx),
        "general" => general_home_card(cx),
        _ => notes_card(data, cx),
    }
}

/// Kartları genişliklerine göre satırlara paketler (2 sütun genişlik).
/// span=2 → tam satır; span=1 → 2'ser yan yana (tek kalırsa tek sütun).
fn pack_rows(cards: Vec<(&str, u8)>) -> Vec<Vec<&str>> {
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut width = 0u8;
    for (key, span) in cards {
        if span == 2 {
            if width > 0 {
                rows.push(std::mem::take(&mut cur));
                width = 0;
            }
            rows.push(vec![key]);
        } else {
            if width + 1 > 2 {
                rows.push(std::mem::take(&mut cur));
                width = 0;
            }
            cur.push(key);
            width += 1;
        }
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    rows
}

/// Tam kartı tıklanabilir yapar. `wide` = 2 sütun genişlik.
/// İç kart `h_full` ile satırdaki en uzun karta eşit yükseklikte uzar.
fn clickable(
    inner: gpui_kit::Div,
    page: Page,
    wide: bool,
    cx: &mut Context<DashboardView>,
) -> gpui_kit::AnyElement {
    let weak = cx.entity().downgrade();
    div()
        .id(SharedString::from(format!("go-{:?}", page)))
        .cursor_pointer()
        .flex()
        .items_stretch()
        .when(wide, |w| w.w_full())
        .when(!wide, |w| w.flex_1())
        .min_h(px(150.))
        .child(inner.h_full())
        .on_click(move |_, _, cx| {
            let _ = weak.update(cx, |this, cx| {
                this.page = page;
                cx.notify();
            });
        })
        .into_any_element()
}

/// Donanım özeti kartı (CPU/RAM/GPU).
fn system_home_card(_data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let info = cx.global::<HardwareInfo>();
    let cpu = info.cpu_usage();
    let ram = info.ram_percent();
    let gpu = info.gpu_info();

    let mut card = card(cx).gap_2();
    card = card.child(card_row("Sistem", Page::System, cx));

    let mut rows = v_flex().gap_1();
    rows = rows.child(
        h_flex()
            .justify_between()
            .py_1()
            .child(div().text_sm().child("CPU"))
            .child(div().text_sm().font_medium().child(format!("{:.0}%", cpu))),
    );
    rows = rows.child(
        h_flex()
            .justify_between()
            .py_1()
            .child(div().text_sm().child("RAM"))
            .child(div().text_sm().font_medium().child(format!("{:.0}%", ram))),
    );
    if let Some(g) = gpu {
        rows = rows.child(
            h_flex()
                .justify_between()
                .py_1()
                .child(div().text_sm().child("GPU"))
                .child(div().text_sm().font_medium().child(match g.usage {
                    Some(u) => format!("{:.0}%", u),
                    None => "—".into(),
                })),
        );
    }
    card.child(rows)
}

/// Aktif pomodoro seansını anasayfada gösterir.
fn pomodoro_home_card(data: &DashboardData, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Pomodoro", Page::Pomodoro, cx));
    match data.pomodoro_remaining() {
        None => {
            card = card.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .py_2()
                    .child("Aktif seans yok."),
            );
        }
        Some((phase, remaining, label, rounds)) => {
            let is_work = phase == PomodoroPhase::Work;
            card = card.child(
                div()
                    .text_2xl()
                    .font_semibold()
                    .text_color(if is_work {
                        cx.theme().primary
                    } else {
                        cx.theme().success
                    })
                    .child(super::pomodoro::fmt_remaining(remaining)),
            );
            card = card.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(if is_work {
                        format!("Odak — {}", label)
                    } else {
                        "Mola".to_string()
                    }),
            );
            card = card.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} tur tamamlandı", rounds)),
            );
        }
    }
    card
}

// --- Granüler donanım / program kartları (kart koleksiyonu) ---

fn cpu_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (usage, temp, name) = {
        let info = cx.global::<HardwareInfo>();
        (info.cpu_usage(), info.cpu_temp(), info.cpu_name())
    };
    card(cx)
        .gap_2()
        .child(card_row("CPU", Page::System, cx))
        .child(
            div()
                .text_2xl()
                .font_semibold()
                .child(format!("{:.0}%", usage)),
        )
        .child(progress_bar(usage, 100., cx))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(match temp {
                    Some(t) => format!("{:.0}°C · {}", t, name),
                    None => name,
                }),
        )
}

fn ram_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (used, total, percent) = {
        let info = cx.global::<HardwareInfo>();
        (
            info.ram_used_gib(),
            info.ram_total_gib(),
            info.ram_percent(),
        )
    };
    card(cx)
        .gap_2()
        .child(card_row("RAM", Page::System, cx))
        .child(
            div()
                .text_2xl()
                .font_semibold()
                .child(format!("{:.1} / {:.1} GiB", used, total)),
        )
        .child(progress_bar(percent, 100., cx))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("%{:.0} kullanımda", percent)),
        )
}

fn gpu_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let gpu = {
        let info = cx.global::<HardwareInfo>();
        info.gpu_info()
    };
    card(cx)
        .gap_2()
        .child(card_row("GPU", Page::System, cx))
        .child(match gpu {
            Some(g) => v_flex()
                .gap_1()
                .child(div().text_2xl().font_semibold().child(match g.usage {
                    Some(u) => format!("{:.0}%", u),
                    None => "—".into(),
                }))
                .when_some(g.temp, |c, t| {
                    c.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{:.0}°C · {}", t, g.name)),
                    )
                }),
            None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("GPU verisi okunamadı."),
        })
}

fn disks_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    // (ad, doluluk yüzdesi) — ödünçleşmeyi bırakmak için kopyala
    let disks: Vec<(String, f32)> = {
        let info = cx.global::<HardwareInfo>();
        info.disks
            .list()
            .iter()
            .take(3)
            .map(|d| {
                let total = d.total_space() as f32 / 1024. / 1024. / 1024.;
                let free = d.available_space() as f32 / 1024. / 1024. / 1024.;
                let percent = if total > 0. {
                    (total - free) / total * 100.
                } else {
                    0.
                };
                (d.name().to_string_lossy().to_string(), percent)
            })
            .collect()
    };
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Diskler", Page::System, cx));
    if disks.is_empty() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Disk bilgisi yok."),
        );
    }
    for (name, percent) in disks {
        card = card.child(
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .justify_between()
                        .child(div().text_sm().child(name))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("%{:.0}", percent)),
                        ),
                )
                .child(progress_bar(percent, 100., cx)),
        );
    }
    card
}

fn temps_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let temps = {
        let info = cx.global::<HardwareInfo>();
        info.all_temps()
    };
    let fan = {
        let info = cx.global::<HardwareInfo>();
        info.gpu_info().and_then(|g| g.fan_percent)
    };
    let mut card = card(cx).gap_2();
    card = card.child(card_row("Sıcaklıklar", Page::System, cx));
    if temps.is_empty() && fan.is_none() {
        return card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Sensör verisi yok."),
        );
    }
    for (label, t) in temps.iter().take(4) {
        card = card.child(
            h_flex()
                .justify_between()
                .py_1()
                .child(div().text_sm().child(label.clone()))
                .child(div().text_sm().font_medium().child(format!("{:.0}°C", t))),
        );
    }
    if let Some(f) = fan {
        card = card.child(
            h_flex()
                .justify_between()
                .py_1()
                .child(div().text_sm().child("Fan"))
                .child(div().text_sm().font_medium().child(format!("%{:.0}", f))),
        );
    }
    card
}

fn general_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (os, host, uptime) = {
        let info = cx.global::<HardwareInfo>();
        (info.os_version(), info.host_name(), info.uptime_str())
    };
    card(cx)
        .gap_2()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Info)
                .child(div().font_semibold().text_sm().child("Bilgisayar")),
        )
        .child(div().text_sm().child(os))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{} · {} açık kalma", host, uptime)),
        )
}
