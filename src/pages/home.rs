use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex, v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::img;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, ImageSource, InteractiveElement as _, IntoElement as _,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
};

use super::{card, progress_bar};
use crate::app::{DashboardView, Page};
use crate::data::{self, DashboardData, PomodoroPhase};
use crate::hardware::HardwareInfo;
use crate::media::{self, MediaCmd, MediaState};
use chrono::Local;
use std::sync::Arc;

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
                            let span = data.home_card_span(key);
                            clickable(body, page, span, cx)
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
pub fn card_body(
    key: &str,
    data: &DashboardData,
    cx: &mut Context<DashboardView>,
) -> gpui_kit::Div {
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
        "network" => network_home_card(cx),
        "media" => media_home_card(cx),
        _ => notes_card(data, cx),
    }
}

/// Kartları genişliklerine göre satırlara paketler (3 sütun genişlik).
/// span=3 → tam satır; diğeri sıkıştırıldığında satırda yeri kalıyor.
fn pack_rows(cards: Vec<(&str, u8)>) -> Vec<Vec<&str>> {
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut width = 0u8;
    for (key, span) in cards {
        if width + span > 3 {
            rows.push(std::mem::take(&mut cur));
            width = 0;
        }
        cur.push(key);
        width += span;
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    rows
}

/// Tam kartı tıklanabilir yapar. `span`: 3 = tam satır, 2 = iki sütun,
/// 1 = tek sütun. İç kart `h_full` ile satırdaki en uzun karta eşit
/// yükseklikte uzar.
fn clickable(
    inner: gpui_kit::Div,
    page: Page,
    span: u8,
    cx: &mut Context<DashboardView>,
) -> gpui_kit::AnyElement {
    let weak = cx.entity().downgrade();
    div()
        .id(SharedString::from(format!("go-{:?}", page)))
        .cursor_pointer()
        .flex()
        .items_stretch()
        .when(span == 3, |w| w.w_full())
        .when(span != 3, |w| {
            w.flex_grow(span as f32).flex_shrink_1().min_w_0()
        })
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

/// Bayt/sn -> okunabilir hız (KB/s, MB/s, GB/s).
fn net_speed_str(bps: f32) -> String {
    if bps >= 1024. * 1024. * 1024. {
        format!("{:.2} GB/s", bps / 1024. / 1024. / 1024.)
    } else if bps >= 1024. * 1024. {
        format!("{:.1} MB/s", bps / 1024. / 1024.)
    } else if bps >= 1024. {
        format!("{:.0} KB/s", bps / 1024.)
    } else {
        format!("{:.0} B/s", bps)
    }
}

/// Bayt -> okunabilir toplam (KB, MB, GB).
fn net_total_str(b: u64) -> String {
    let b = b as f32;
    if b >= 1024. * 1024. * 1024. {
        format!("{:.2} GB", b / 1024. / 1024. / 1024.)
    } else if b >= 1024. * 1024. {
        format!("{:.1} MB", b / 1024. / 1024.)
    } else {
        format!("{:.0} KB", b / 1024.)
    }
}

/// Ağ özeti: indirme/yükleme hızları + açılımdan bu yana toplam.
fn network_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let (rx, tx, (total_rx, total_tx)) = {
        let info = cx.global::<HardwareInfo>();
        (
            info.net_download_speed_bps(),
            info.net_upload_speed_bps(),
            info.net_total(),
        )
    };
    card(cx)
        .gap_2()
        .child(card_row("Ağ", Page::System, cx))
        .child(
            h_flex()
                .gap_4()
                .items_center()
                .child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            div()
                                .text_color(cx.theme().primary)
                                .child(IconName::ArrowDown),
                        )
                        .child(div().text_sm().font_medium().child(net_speed_str(rx))),
                )
                .child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            div()
                                .text_color(cx.theme().success)
                                .child(IconName::ArrowUp),
                        )
                        .child(div().text_sm().font_medium().child(net_speed_str(tx))),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "oturum: ↓ {} · ↑ {}",
                    net_total_str(total_rx),
                    net_total_str(total_tx)
                )),
        )
}

/// Medya kontrol kartı: SMTC üzerinden çalan müzik + oynatma kontrolleri.
fn media_home_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let state = cx.global::<MediaState>();

    if !state.has_session {
        return card(cx)
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Play)
                    .child(div().font_semibold().text_sm().child("Medya")),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Şu an çalan bir medya yok"),
            );
    }

    // kapak görseli: SMTC'den gelen baytlar gpui Image'e dönüştürülür
    let (cover, app, title, artist, album, playing) = {
        let state = cx.global::<MediaState>();
        let cover = state.thumb.as_ref().map(|thumb| {
            let format = if thumb.format == "png" {
                gpui_kit::ImageFormat::Png
            } else {
                gpui_kit::ImageFormat::Jpeg
            };
            let image = Arc::new(gpui_kit::Image {
                format,
                bytes: thumb.bytes.clone(),
                id: thumb.id,
            });
            img(ImageSource::Image(image))
                .size(px(72.))
                .rounded_sm()
                .flex_shrink_0()
        });
        (
            cover,
            state.app.clone(),
            state.title.clone(),
            state.artist.clone(),
            state.album.clone(),
            state.playing,
        )
    };

    // düğmeler: komutu arka planda yollar (WinRT bloklu)
    let cmd_btn = |id: &'static str, icon: IconName, cmd: MediaCmd| {
        div()
            .id(id)
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size_7()
            .rounded_sm()
            .text_color(cx.theme().muted_foreground)
            .hover(|s| s.bg(cx.theme().accent.opacity(0.5)))
            .child(icon)
            .on_click(move |_, _, cx| {
                // tıklama dıştaki kart sarmalayıcısına (sayfa geçişi) sızmaz
                cx.stop_propagation();
                cx.background_executor()
                    .spawn(async move { media::send(cmd) })
                    .detach();
            })
    };

    let play_icon = if playing {
        IconName::Pause
    } else {
        IconName::Play
    };

    let mut card = card(cx).gap_2().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(IconName::Play)
            .child(div().font_semibold().text_sm().child("Medya"))
            .child(
                div()
                    .text_xs()
                    .px_1p5()
                    .rounded_full()
                    .bg(cx.theme().accent.opacity(0.4))
                    .text_color(cx.theme().muted_foreground)
                    .child(app.clone()),
            ),
    );

    if let Some(cover) = cover {
        card =
            card.child(
                h_flex().gap_3().items_start().child(cover).child(
                    v_flex()
                        .min_w_0()
                        .overflow_hidden()
                        .gap_1()
                        .child(div().text_sm().font_semibold().truncate().child(
                            if title.is_empty() {
                                "Bilinmeyen parça".to_string()
                            } else {
                                title.clone()
                            },
                        ))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .truncate()
                                .child(if artist.is_empty() {
                                    "Bilinmeyen sanatçı".to_string()
                                } else {
                                    artist.clone()
                                }),
                        )
                        .when(!album.is_empty(), |c| {
                            c.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .truncate()
                                    .child(album.clone()),
                            )
                        }),
                ),
            );
    } else {
        card = card
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .truncate()
                    .child(if title.is_empty() {
                        "Bilinmeyen parça".to_string()
                    } else {
                        title.clone()
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .truncate()
                    .child(if artist.is_empty() {
                        "Bilinmeyen sanatçı".to_string()
                    } else {
                        artist.clone()
                    }),
            );
    }

    card = card.child(
        h_flex()
            .gap_1()
            .justify_center()
            .pt_1()
            .child(cmd_btn("media-prev", IconName::ChevronLeft, MediaCmd::Prev))
            .child(cmd_btn("media-toggle", play_icon, MediaCmd::Toggle))
            .child(cmd_btn(
                "media-next",
                IconName::ChevronRight,
                MediaCmd::Next,
            )),
    );

    card
}
