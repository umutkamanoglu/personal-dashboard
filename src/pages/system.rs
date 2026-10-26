use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{button::Button, h_flex, v_flex, ActiveTheme, IconName, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _,
};
use sysinfo::System;

use super::card;
use crate::app::DashboardView;
use crate::hardware::{GpuInfo, HardwareInfo};

pub fn system_page(_view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(960.))
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(div().font_semibold().text_lg().child("Sistem"))
                .child(
                    Button::new("refresh-system")
                        .small()
                        .icon(IconName::RefreshCw)
                        .label("Yenile")
                        .on_click(cx.listener(|_, _, _, cx| {
                            {
                                let info = cx.global_mut::<HardwareInfo>();
                                info.sys.refresh_all();
                            }
                            cx.notify();
                        })),
                ),
        )
        .child(cpu_card(cx))
        .child(gpu_card(cx))
        .child(ram_card(cx))
        .child(disks_card(cx))
        .child(general_card(cx))
        .child(temps_card(cx))
        .child(network_card(cx))
        .into_any_element()
}

fn stat_row(label: &str, value: String, cx: &gpui_kit::App) -> Div {
    h_flex()
        .justify_between()
        .py_1()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_string()),
        )
        .child(div().text_sm().font_medium().child(value))
}

fn usage_bar(percent: f32, cx: &gpui_kit::App) -> Div {
    let fraction = (percent / 100.).clamp(0., 1.);
    div()
        .h(px(6.))
        .w_full()
        .rounded_full()
        .bg(cx.theme().muted)
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .rounded_full()
                .bg(percent_color(percent, cx))
                .w(gpui_kit::relative(fraction)),
        )
}

fn percent_color(percent: f32, cx: &gpui_kit::App) -> gpui_kit::Hsla {
    if percent > 85. {
        cx.theme().danger
    } else if percent > 60. {
        cx.theme().warning
    } else {
        cx.theme().primary
    }
}

fn temp_text(temp: Option<f32>) -> String {
    match temp {
        Some(t) => format!("{:.0}°C", t),
        None => "—".into(),
    }
}

// --- Fanlar & sıcaklıklar ---

fn temps_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let temps = info.all_temps();
    let fan = info.gpu_info().and_then(|g| g.fan_percent);

    let mut card = card(cx).gap_3().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(IconName::Cpu)
            .child(div().font_medium().child("Sıcaklıklar & Fanlar")),
    );

    if let Some(f) = fan {
        card = card.child(stat_row("GPU fanı", format!("{:.0}%", f), cx));
    }
    if temps.is_empty() {
        card = card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Okunabilir sıcaklık sensörü bulunamadı (yönetici izni gerekebilir)."),
        );
    } else {
        for (label, t) in temps {
            card = card.child(stat_row(&label, format!("{:.0}°C", t), cx));
        }
    }
    card
}

// --- CPU ---

fn cpu_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let name = info.cpu_name();
    let usage = info.cpu_usage();
    let temp = info.cpu_temp();
    let freq = info.cpu_freq_mhz();
    let cores = info.sys.cpus().len();
    let per_core = info.cpu_core_usage();

    let mut card = card(cx).gap_3().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(IconName::Cpu)
            .child(div().font_medium().child("İşlemci")),
    );
    card = card.child(stat_row("Model", name, cx));
    card = card.child(stat_row("Çekirdek", cores.to_string(), cx));
    card = card.child(stat_row(
        "Frekans",
        format!("{:.2} GHz", freq as f32 / 1000.),
        cx,
    ));
    card = card.child(stat_row("Toplam kullanım", format!("{:.1}%", usage), cx));
    card = card.child(stat_row("Sıcaklık", temp_text(temp), cx));
    card = card.child(usage_bar(usage, cx));

    // çekirdek başına kullanım (satır halinde minik bar'lar)
    if !per_core.is_empty() {
        let grid = h_flex()
            .gap_2()
            .flex_wrap()
            .children(per_core.iter().enumerate().map(|(ix, u)| {
                v_flex()
                    .gap_1()
                    .w(px(76.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("c{} {:.0}%", ix, u)),
                    )
                    .child(usage_bar(*u, cx))
            }));
        card = card.child(div().mt_2().child(grid));
    }
    card
}

// --- GPU ---

fn gpu_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let gpu: Option<GpuInfo> = info.gpu_info();

    let mut card = card(cx).gap_3().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(IconName::MemoryStick)
            .child(div().font_medium().child("GPU")),
    );

    match gpu {
        None => {
            card = card.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .py_2()
                    .child(match &info.nvml_error {
                        Some(err) => format!("NVML okunamadı: {}", err),
                        None => "Bilgi yok.".into(),
                    }),
            );
        }
        Some(g) => {
            let gb = |mib: f32| format!("{:.1} GiB", mib / 1024.);
            card = card.child(stat_row("Model", g.name.clone(), cx));
            if let Some(u) = g.usage {
                card = card.child(stat_row("Kullanım", format!("{:.0}%", u), cx));
                card = card.child(usage_bar(u, cx));
            }
            card = card.child(stat_row("Sıcaklık", temp_text(g.temp), cx));
            if let (Some(used), Some(total)) = (g.memory_used_mib, g.memory_total_mib) {
                card = card.child(stat_row(
                    "VRAM",
                    format!("{} / {}", gb(used), gb(total)),
                    cx,
                ));
            }
            if let Some(p) = g.power_watts {
                card = card.child(stat_row("Güç", format!("{:.1} W", p), cx));
            }
            card = card.child(stat_row(
                "Fan",
                match g.fan_percent {
                    Some(f) => format!("{:.0}%", f),
                    None => "—".into(),
                },
                cx,
            ));
        }
    }
    card
}

// --- RAM ---

fn ram_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let total = info.ram_total_gib();
    let used = info.ram_used_gib();
    let percent = info.ram_percent();

    card(cx)
        .gap_3()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::MemoryStick)
                .child(div().font_medium().child("RAM")),
        )
        .child(stat_row("Toplam", format!("{:.1} GiB", total), cx))
        .child(stat_row("Kullanılan", format!("{:.1} GiB", used), cx))
        .child(stat_row("Boş", format!("{:.1} GiB", total - used), cx))
        .child(stat_row("Yüzde", format!("{:.1}%", percent), cx))
        .child(usage_bar(percent, cx))
}

// --- Diskler ---

fn disks_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let mut card = card(cx).gap_3().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(IconName::HardDrive)
            .child(div().font_medium().child("Diskler")),
    );

    if info.disks.list().is_empty() {
        card = card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_2()
                .child("Disk bilgisi okunamadı."),
        );
        return card;
    }

    for d in info.disks.list() {
        let total = d.total_space() as f32 / 1024. / 1024. / 1024.;
        let free = d.available_space() as f32 / 1024. / 1024. / 1024.;
        let used = total - free;
        let percent = if total > 0. { used / total * 100. } else { 0. };
        card = card.child(
            v_flex()
                .gap_1()
                .py_1()
                .child(
                    h_flex()
                        .justify_between()
                        .child(div().text_sm().font_medium().child(format!(
                            "{} ({})",
                            d.name().to_string_lossy(),
                            d.mount_point().display()
                        )))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{:.0} / {:.0} GiB · %{:.0}", used, total, percent)),
                        ),
                )
                .child(usage_bar(percent, cx)),
        );
    }
    card
}

// --- Sistem genel ---

fn general_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let name: String = System::name().unwrap_or_else(|| "—".into());
    card(cx)
        .gap_3()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Info)
                .child(div().font_medium().child("Bilgisayar")),
        )
        .child(stat_row("İşletim sistemi", info.os_version(), cx))
        .child(stat_row("Sürüm", name.clone(), cx))
        .child(stat_row("Çekirdek", info.kernel_version(), cx))
        .child(stat_row("Makine adı", info.host_name(), cx))
        .child(stat_row("Açık kalma", info.uptime_str(), cx))
}

// --- Ağ ---

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

fn network_card(cx: &mut Context<DashboardView>) -> Div {
    let info = cx.global::<HardwareInfo>();
    let rx = info.net_download_speed_bps();
    let tx = info.net_upload_speed_bps();
    let (total_rx, total_tx) = info.net_total();
    let interfaces: Vec<(String, f32, f32)> = info
        .net_rx_speed
        .iter()
        .zip(info.net_tx_speed.iter())
        .map(|((n, r), (_, t))| (n.clone(), *r, *t))
        .collect();

    card(cx)
        .gap_3()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Network)
                .child(div().font_medium().child("Ağ")),
        )
        .child(stat_row("İndirme (şimdi)", net_speed_str(rx), cx))
        .child(stat_row("Yükleme (şimdi)", net_speed_str(tx), cx))
        .child(stat_row(
            "Oturum toplamı",
            format!(
                "↓ {} · ↑ {}",
                net_total_str(total_rx),
                net_total_str(total_tx)
            ),
            cx,
        ))
        .when(!interfaces.is_empty(), |c| {
            c.child(
                div()
                    .mt_1()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Arayüzler"),
            )
            .children(interfaces.into_iter().map(|(name, r, t)| {
                v_flex()
                    .py_1()
                    .child(div().text_sm().font_medium().child(name))
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("↓ in / ↑ out"),
                            )
                            .child(div().text_sm().font_medium().child(format!(
                                "{} · {}",
                                net_speed_str(r),
                                net_speed_str(t)
                            ))),
                    )
            }))
        })
}
