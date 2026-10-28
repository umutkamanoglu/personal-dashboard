use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    radio::RadioGroup,
    v_flex, ActiveTheme, IconName, Sizable as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, Div, InteractiveElement as _, IntoElement as _,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
};

use super::card;
use crate::app::DashboardView;
use crate::data::DashboardData;
use crate::weather::PROVIDERS;

pub fn settings_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    // grid düzeni: her satır iki kart, gap_4 ile nefes alır;
    // geniş kartlar (koleksiyon) tüm satırı kaplar
    let row = |a: Div, b: Div| {
        h_flex()
            .gap_4()
            .items_stretch()
            .child(a.flex_1().min_w_0())
            .child(b.flex_1().min_w_0())
    };
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(860.))
        .child(card_title(gpui_kit_assets::IconName::Settings2, "Ayarlar"))
        .child(row(profile_card(view, cx), theme_card(cx)))
        .child(row(pomodoro_card(cx), peaclock_card(cx)))
        .child(row(weather_card(view, cx), storage_card(cx)))
        .child(reset_card(view, cx))
        .child(visualizer_card(cx))
        .child(backup_card(cx))
        .child(home_layout_card(cx))
        .into_any_element()
}

/// Kart başlığı: aranan ayarı bulmayı kolaylaştıran küçük bir ikonla.
fn card_title(icon: gpui_kit_assets::IconName, label: &'static str) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .font_semibold()
        .text_lg()
        .child(icon)
        .child(label)
}

pub const CARD_LABELS: [(&str, &str); 18] = [
    ("schedule", "Bugünün programı"),
    ("habits", "Alışkanlıklar"),
    ("routines", "Rutinler"),
    ("notes", "Notlar & hatırlatıcılar"),
    ("pomodoro", "Pomodoro"),
    ("system", "Sistem özeti"),
    ("cpu", "CPU"),
    ("ram", "RAM"),
    ("gpu", "GPU"),
    ("disks", "Diskler"),
    ("temps", "Sıcaklıklar"),
    ("general", "Bilgisayar"),
    ("network", "Ağ"),
    ("media", "Medya"),
    ("downloads", "İndirmeler"),
    ("visualizer", "Görselleştirici"),
    ("peaclock", "Büyük saat"),
    ("weather", "Hava durumu"),
];

fn card_label(key: &str) -> &'static str {
    for (k, l) in CARD_LABELS {
        if k == key {
            return l;
        }
    }
    "Sistem"
}

/// Kullanıcının anasayfa ızgarasını özelleştirdiği kart koleksiyonu:
/// kart ekle/çıkar, sıralama ve genişlik (1 = yarım sütun, 2 = tam sütun).
fn home_layout_card(cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>().clone();
    let mut card = card(cx).gap_3().child(card_title(
        gpui_kit_assets::IconName::LayoutGrid,
        "Anasayfa kartları",
    ));

    // 1) Görünür kartlar: ad + genişlik 1/2/3 segmanı + sıralama + kaldırma.
    // Her şey tek satırda: "ne kadar geniş, kimin üstünde" tek bakışta.
    card = card.children(data.home_order.iter().map(|key| {
        let key = key.clone();
        let span = data.home_card_span(&key).clamp(1, 3);
        let mut row = h_flex()
            .gap_1()
            .items_center()
            .py_1()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(card_icon(&key)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .truncate()
                            .child(card_label(&key).to_string()),
                    ),
            );
        // genişlik segmanı: 1 / 2 / 3
        for w in [1u8, 2, 3] {
            row = row.child(
                Button::new(SharedString::from(format!("home-span-{}-{}", key, w)))
                    .xsmall()
                    .when(span == w, |b| {
                        b.primary().label(match w {
                            1 => "1",
                            2 => "2",
                            _ => "3",
                        })
                    })
                    .when(span != w, |b| {
                        b.ghost().label(match w {
                            1 => "1",
                            2 => "2",
                            _ => "3",
                        })
                    })
                    .tooltip(match w {
                        1 => "Yarım sütun",
                        2 => "Tam sütun",
                        _ => "Satırın tamamı",
                    })
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().set_home_span(&key, w);
                            cx.notify();
                        }
                    })),
            );
        }
        row = row
            .child(
                Button::new(SharedString::from(format!("home-up-{}", key)))
                    .xsmall()
                    .ghost()
                    .icon(IconName::ArrowUp)
                    .tooltip("Yukarı taşı")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().move_home_card(&key, true);
                            cx.notify();
                        }
                    })),
            )
            .child(
                Button::new(SharedString::from(format!("home-down-{}", key)))
                    .xsmall()
                    .ghost()
                    .icon(IconName::ArrowDown)
                    .tooltip("Aşağı taşı")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().move_home_card(&key, false);
                            cx.notify();
                        }
                    })),
            )
            .child(
                Button::new(SharedString::from(format!("home-vis-{}", key)))
                    .xsmall()
                    .ghost()
                    .text_color(cx.theme().danger)
                    .icon(IconName::Close)
                    .tooltip("Anasayfadan kaldır")
                    .on_click(cx.listener({
                        let key = key.clone();
                        move |_, _, _, cx| {
                            cx.global_mut::<DashboardData>().remove_home_card(&key);
                            cx.notify();
                        }
                    })),
            );
        row
    }));

    // 2) Eklenebilir kartlar: kompakt chip listesi (+ ile) — önizlemeden daha hızlı
    let available = data.home_cards_available();
    if !available.is_empty() {
        card = card
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Eklenebilir kartlar — çıkarırsan buraya döner:"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .items_start()
                    .children(available.into_iter().map(|key| {
                        let key_str = key.to_string();
                        div()
                            .id(SharedString::from(format!("home-add-{}", key)))
                            .cursor_pointer()
                            .px_2()
                            .py_1()
                            .gap_2()
                            .rounded_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .flex()
                            .items_center()
                            .text_sm()
                            .hover(|s| {
                                s.bg(cx.theme().accent.opacity(0.35))
                                    .text_color(cx.theme().accent_foreground)
                            })
                            .child(
                                div()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(gpui_kit_assets::IconName::Plus),
                            )
                            .child(card_icon(key))
                            .child(card_label(key).to_string())
                            .on_click(cx.listener({
                                let key_str = key_str.clone();
                                move |_, _, _, cx| {
                                    cx.global_mut::<DashboardData>().add_home_card(&key_str);
                                    cx.notify();
                                }
                            }))
                    })),
            );
    }

    card.child(
        div().text_xs().text_color(cx.theme().muted_foreground).child(
            "Genişlik: 1 = yarım, 2 = tam sütun, 3 = satırın tamamı. Sıradaki kart yukarıdan aşağı dizilir.",
        ),
    )
}

/// Kart koleksiyonu anahtarına göre küçük simge (ayar listesi + chip'ler için).
fn card_icon(key: &str) -> gpui_kit_assets::IconName {
    use gpui_kit_assets::IconName as I;
    match key {
        "schedule" => I::Calendar,
        "habits" => I::CircleCheck,
        "routines" => I::RefreshCw,
        "notes" => I::FileText,
        "pomodoro" => I::Play,
        "cpu" | "system" => I::Cpu,
        "ram" => I::MemoryStick,
        "gpu" => I::MonitorCog,
        "disks" => I::HardDrive,
        "temps" => I::Thermometer,
        "general" => I::Monitor,
        "network" => I::Wifi,
        "media" => I::Music,
        "downloads" => I::Download,
        "visualizer" => I::AudioLines,
        "peaclock" => I::Clock,
        "weather" => I::CloudSun,
        _ => I::LayoutDashboard,
    }
}

fn profile_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>().clone();
    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::SquareUserRound,
            "Profil",
        ))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Input::new(&view.name_input).flex_1())
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("şu an: {}", data.name)),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Selamlamada kullanılır. Değiştirmek için yaz ve uygula."),
        )
        .child(
            Button::new("apply-name")
                .small()
                .label("Uygula")
                .on_click(cx.listener(|this, _, _, cx| {
                    let name = this.name_input.read(cx).value().trim().to_string();
                    let data = cx.global::<DashboardData>().clone();
                    cx.global_mut::<DashboardData>().set_settings(
                        name,
                        data.work_min,
                        data.break_min,
                    );
                    cx.notify();
                })),
        )
}

fn theme_card(cx: &mut Context<DashboardView>) -> Div {
    let data = cx.global::<DashboardData>();
    let dark = data.dark_mode;
    let system = data.theme_system;
    card(cx)
        .gap_3()
        .child(card_title(gpui_kit_assets::IconName::SunMoon, "Tema"))
        .child(
            RadioGroup::new("theme-mode")
                .children(["Açık", "Koyu", "Sistem"])
                .selected_index(Some(if system { 2 } else if dark { 1 } else { 0 }))
                .on_change(cx.listener(|_, index, window, cx| {
                    match *index {
                        // Windows'un açık/koyu tercihi izlenir
                        2 => {
                            cx.global_mut::<DashboardData>().set_theme_system();
                            let dark = cx.global::<DashboardData>().dark_mode;
                            let mode = if dark {
                                gpui_kit::component::ThemeMode::Dark
                            } else {
                                gpui_kit::component::ThemeMode::Light
                            };
                            gpui_kit::component::Theme::change(mode, Some(window), cx);
                        }
                        _ => {
                            let dark = *index == 1;
                            cx.global_mut::<DashboardData>().set_dark_mode(dark);
                            let mode = if dark {
                                gpui_kit::component::ThemeMode::Dark
                            } else {
                                gpui_kit::component::ThemeMode::Light
                            };
                            gpui_kit::component::Theme::change(mode, Some(window), cx);
                        }
                    }
                    cx.notify();
                })),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Tercih uygulama yeniden açıldığında korunur. “Sistem” seçeneği Windows'un açık/koyu ayarını izler."),
        )
}

/// Tüm verinin JSON yedeğini dışa aktar / içe aktar.
/// Dosya diyalogları bloklayıcı olduğundan arka filanda çalışır;
/// veri kopyası diyalog açılmadan önce alınır.
fn backup_card(cx: &mut Context<DashboardView>) -> Div {
    card(cx)
        .gap_3()
        .child(card_title(gpui_kit_assets::IconName::Database, "Yedekleme"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "Tüm veri (profil, alışkanlıklar, rutinler, notlar, ayarlar) tek JSON dosyasına yazılır; içe aktarım mevcut verinin yerine geçer.",
                ),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("backup-export")
                        .small()
                        .icon(gpui_kit_assets::IconName::Download)
                        .label("Dışa aktar…")
                        .on_click(cx.listener(|_, _, _, cx| {
                            let snapshot = cx.global::<DashboardData>().clone();
                            cx.spawn(async move |_, _cx| {
                                let saved = rfd::AsyncFileDialog::new()
                                    .set_title("Yedeği dışa aktar")
                                    .set_file_name("personal-dashboard-yedek.json")
                                    .add_filter("JSON", &["json"])
                                    .save_file()
                                    .await;
                                // dosya seçildi: içeriği yaz (handle.path() senkron)
                                let _ = saved.map(|h| {
                                    snapshot.export_to(&std::path::PathBuf::from(h.path()))
                                });
                            })
                            .detach();
                        })),
                )
                .child(
                    Button::new("backup-import")
                        .small()
                        .icon(gpui_kit_assets::IconName::Upload)
                        .label("İçe aktar…")
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.spawn(async move |this, cx| {
                                let picked = rfd::AsyncFileDialog::new()
                                    .set_title("Yedeği içe aktar")
                                    .add_filter("JSON", &["json"])
                                    .pick_file()
                                    .await;
                                let Some(file) = picked else {
                                    return;
                                };
                                let path = std::path::PathBuf::from(file.path());
                                let imported = cx
                                    .background_executor()
                                    .spawn(async move {
                                        DashboardData::import_from(&path)
                                    })
                                    .await;
                                this.update(cx, |_, cx| {
                                    if let Ok(data) = imported {
                                        let dark = data.dark_mode;
                                        cx.set_global(data);
                                        let mode = if dark {
                                            gpui_kit::component::ThemeMode::Dark
                                        } else {
                                            gpui_kit::component::ThemeMode::Light
                                        };
                                        gpui_kit::component::Theme::change(mode, None, cx);
                                        // hava durumu önbelleğini de tazele
                                        if let Ok(mut u) =
                                            cx.global::<crate::weather::WeatherState>().updated.lock()
                                        {
                                            *u = None;
                                        }
                                    }
                                    cx.notify();
                                })
                                .ok();
                            })
                            .detach();
                        })),
                ),
        )
}

/// Hava durumu ayarları: konum gir, sağlayıcı seç, (gerekirse) API anahtarı.
/// Değişiklik anında veritabanına kaydedilir ve önbellek tazelenir.
fn weather_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    let (provider, location) = {
        let data = cx.global::<DashboardData>();
        (data.weather_provider.clone(), data.weather_location.clone())
    };

    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::CloudSun,
            "Hava durumu",
        ))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Input::new(&view.weather_loc_input).flex_1())
                .child(
                    Button::new("weather-loc-apply")
                        .small()
                        .icon(gpui_kit_assets::IconName::Check)
                        .label("Uygula")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let loc = this.weather_loc_input.read(cx).value().trim().to_string();
                            if loc.is_empty() {
                                return;
                            }
                            cx.global_mut::<DashboardData>().set_weather_location(loc);
                            // yeni konum = önbelleği tazele
                            if let Ok(mut u) =
                                cx.global::<crate::weather::WeatherState>().updated.lock()
                            {
                                *u = None;
                            }
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(if location.is_empty() {
                    "Şehir gir (ör. İzmir) — tahmin bunun için çekilir.".to_string()
                } else {
                    format!("Konum: {location}")
                }),
        )
        .child(
            RadioGroup::new("weather-provider")
                .children([
                    "Open-Meteo (anahtar gerekmez)",
                    "OpenWeatherMap (anahtar gerekli)",
                ])
                .selected_index(Some(
                    PROVIDERS.iter().position(|p| p.0 == provider).unwrap_or(0),
                ))
                .on_change(cx.listener(move |_, index: &usize, _, cx| {
                    let id = PROVIDERS[*index].0.to_string();
                    cx.global_mut::<DashboardData>().set_weather_provider(id);
                    if let Ok(mut u) = cx.global::<crate::weather::WeatherState>().updated.lock() {
                        *u = None;
                    }
                    cx.notify();
                })),
        )
        .when(provider == "openweathermap", |c| {
            c.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Input::new(&view.weather_key_input).flex_1())
                    .child(
                        Button::new("weather-key-apply")
                            .small()
                            .icon(gpui_kit_assets::IconName::Check)
                            .label("Kaydet")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let key =
                                    this.weather_key_input.read(cx).value().trim().to_string();
                                cx.global_mut::<DashboardData>()
                                    .set_weather_api_key("openweathermap", key);
                                if let Ok(mut u) =
                                    cx.global::<crate::weather::WeatherState>().updated.lock()
                                {
                                    *u = None;
                                }
                                cx.notify();
                            })),
                    ),
            )
        })
}

/// İndirme kayıt klasörü: dosya diyalogundan seçilir; boşsa varsayılan uygulanır.
fn storage_card(cx: &mut Context<DashboardView>) -> Div {
    let current = cx.global::<DashboardData>().download_dir.clone();
    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::FolderOpen,
            "İndirme klasörü",
        ))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .truncate()
                .child(current.clone().unwrap_or_default()),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Boşsa İndirilenler\\personal-dashboard kullanılır."),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("download-dir-pick")
                        .small()
                        .icon(gpui_kit_assets::IconName::FolderOpen)
                        .label("Klasör seç…")
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.spawn(async move |this, cx| {
                                let picked = rfd::AsyncFileDialog::new()
                                    .set_title("İndirme klasörünü seç")
                                    .pick_folder()
                                    .await;
                                let Some(file) = picked else { return };
                                let path = file.path().to_path_buf();
                                this.update(cx, |_, cx| {
                                    cx.global_mut::<DashboardData>()
                                        .set_download_dir(Some(path.display().to_string()));
                                    cx.notify();
                                })
                                .ok();
                            })
                            .detach();
                        })),
                )
                .when(current.is_some(), |r| {
                    r.child(
                        Button::new("download-dir-default")
                            .small()
                            .icon(gpui_kit_assets::IconName::RefreshCw)
                            .label("Varsayılana dön")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.global_mut::<DashboardData>().set_download_dir(None);
                                cx.notify();
                            })),
                    )
                }),
        )
}

/// Tüm veriyi siler; uygulama stok seed değerlerle yeniden açılır gibi olur.
/// İlk tık silmeyi "arm" eder; ikinci tık gerçekten sıfırlar.
fn reset_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> Div {
    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::CircleAlert,
            "Veri sıfırlama",
        ))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "data.json silinir: alışkanlıklar, rutinler, notlar, programlar \
                     ve ayarlar stok başlangıç değerlerine döner.",
                ),
        )
        .child(
            Button::new("data-reset")
                .small()
                .danger()
                .icon(gpui_kit_assets::IconName::Trash)
                .label(if view.reset_armed {
                    "Emin misin? Tekrar tıkla"
                } else {
                    "Sıfırla"
                })
                .on_click(cx.listener(move |view, _, window, cx| {
                    if !view.reset_armed {
                        view.reset_armed = true;
                        cx.notify();
                        return;
                    }
                    view.reset_armed = false;
                    let fresh = crate::data::DashboardData::reset();
                    let dark = fresh.dark_mode;
                    cx.set_global(fresh);
                    let mode = if dark {
                        gpui_kit::component::ThemeMode::Dark
                    } else {
                        gpui_kit::component::ThemeMode::Light
                    };
                    gpui_kit::component::Theme::change(mode, Some(window), cx);
                    cx.notify();
                })),
        )
}

const WORK_OPTIONS: [u32; 3] = [15, 25, 50];
const BREAK_OPTIONS: [u32; 3] = [5, 10, 15];

fn pomodoro_card(cx: &mut Context<DashboardView>) -> Div {
    let work = cx.global::<DashboardData>().work_min;
    let break_min = cx.global::<DashboardData>().break_min;
    let work_idx = WORK_OPTIONS.iter().position(|o| *o == work).unwrap_or(1);
    let break_idx = BREAK_OPTIONS
        .iter()
        .position(|o| *o == break_min)
        .unwrap_or(0);

    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::Timer,
            "Pomodoro varsayılanları",
        ))
        .child(
            h_flex()
                .gap_6()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Odak süresi"),
                        )
                        .child(
                            RadioGroup::new("settings-work")
                                .children(["15 dk", "25 dk", "50 dk"])
                                .selected_index(Some(work_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let work = WORK_OPTIONS[*index];
                                    let data = cx.global::<DashboardData>().clone();
                                    cx.global_mut::<DashboardData>().set_settings(
                                        data.name.clone(),
                                        work,
                                        data.break_min,
                                    );
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Mola süresi"),
                        )
                        .child(
                            RadioGroup::new("settings-break")
                                .children(["5 dk", "10 dk", "15 dk"])
                                .selected_index(Some(break_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let break_min = BREAK_OPTIONS[*index];
                                    let data = cx.global::<DashboardData>().clone();
                                    cx.global_mut::<DashboardData>().set_settings(
                                        data.name.clone(),
                                        data.work_min,
                                        break_min,
                                    );
                                    cx.notify();
                                })),
                        ),
                ),
        )
}

const VIZ_BAR_OPTIONS: [u8; 5] = [16, 24, 32, 48, 64];
const VIZ_COLOR_OPTIONS: [&str; 6] = [
    "accent", "primary", "success", "warning", "danger", "rainbow",
];
const VIZ_COLOR_LABELS: [&str; 6] = ["Vurgu", "Ana", "Yeşil", "Sarı", "Kırmızı", "Gökkuşağı"];

/// Görselleştirici kartını özelleştirme (CAVA tarzı ses görselleştirici):
/// bar sayısı, bar rengi, aynalanma ve tepe göstergesi.
fn visualizer_card(cx: &mut Context<DashboardView>) -> Div {
    let (bars, color, mirror, peaks) = {
        let d = cx.global::<DashboardData>();
        (d.viz_bars, d.viz_color.clone(), d.viz_mirror, d.viz_peaks)
    };
    let bar_idx = VIZ_BAR_OPTIONS.iter().position(|o| *o == bars).unwrap_or(1);
    let color_idx = VIZ_COLOR_OPTIONS
        .iter()
        .position(|o| *o == color)
        .unwrap_or(0);

    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::AudioLines,
            "Görselleştirici görünümü",
        ))
        .child(
            h_flex()
                .gap_6()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Bar sayısı"),
                        )
                        .child(
                            RadioGroup::new("viz-bars")
                                .children(VIZ_BAR_OPTIONS.iter().map(|b| b.to_string()))
                                .selected_index(Some(bar_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let bars = VIZ_BAR_OPTIONS[*index];
                                    cx.global_mut::<DashboardData>().set_viz_bars(bars);
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Bar rengi"),
                        )
                        .child(
                            RadioGroup::new("viz-color")
                                .children(VIZ_COLOR_LABELS)
                                .selected_index(Some(color_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let color = VIZ_COLOR_OPTIONS[*index];
                                    cx.global_mut::<DashboardData>().set_viz_color(color);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            h_flex()
                .gap_4()
                .child(
                    Checkbox::new("viz-mirror")
                        .label("Aynalanmış (ortada)")
                        .checked(mirror)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_viz_mirror(*checked);
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("viz-peaks")
                        .label("Tepe göstergesi")
                        .checked(peaks)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_viz_peaks(*checked);
                            cx.notify();
                        })),
                ),
        )
}

/// Büyük saat (peaclock) kartını özelleştirme: saat biçimi, saniye,
/// yanıp sönen iki nokta, tarih satırı ve rakam rengi.
fn peaclock_card(cx: &mut Context<DashboardView>) -> Div {
    let (h24, secs, blink, date, color) = {
        let d = cx.global::<DashboardData>();
        (
            d.clock_24h,
            d.clock_seconds,
            d.clock_blink,
            d.clock_date,
            d.clock_color.clone(),
        )
    };
    let color_idx = VIZ_COLOR_OPTIONS
        .iter()
        .position(|o| *o == color)
        .unwrap_or(0);

    card(cx)
        .gap_3()
        .child(card_title(
            gpui_kit_assets::IconName::Clock,
            "Büyük saat görünümü",
        ))
        .child(
            h_flex()
                .gap_6()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Saat biçimi"),
                        )
                        .child(
                            RadioGroup::new("clock-24h")
                                .children(["24 saat", "12 saat"])
                                .selected_index(Some(if h24 { 0 } else { 1 }))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    cx.global_mut::<DashboardData>().set_clock_24h(*index == 0);
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Rakam rengi"),
                        )
                        .child(
                            RadioGroup::new("clock-color")
                                .children(VIZ_COLOR_LABELS)
                                .selected_index(Some(color_idx))
                                .on_change(cx.listener(|_, index, _, cx| {
                                    let color = VIZ_COLOR_OPTIONS[*index];
                                    cx.global_mut::<DashboardData>().set_clock_color(color);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            h_flex()
                .gap_4()
                .child(
                    Checkbox::new("clock-seconds")
                        .label("Saniyeler")
                        .checked(secs)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_clock_seconds(*checked);
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("clock-blink")
                        .label("Yanıp sönen iki nokta")
                        .checked(blink)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_clock_blink(*checked);
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("clock-date")
                        .label("Tarih satırı")
                        .checked(date)
                        .on_change(cx.listener(|_, checked, _, cx| {
                            cx.global_mut::<DashboardData>().set_clock_date(*checked);
                            cx.notify();
                        })),
                ),
        )
}
