use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    v_flex, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{div, px, AnyElement, Context, IntoElement as _, ParentElement as _, Styled as _};
use gpui_kit_assets::IconName;

use super::card;
use crate::app::DashboardView;
use crate::downloads::{self, DlStatus, DownloadState};

pub fn downloads_page(view: &DashboardView, cx: &mut Context<DashboardView>) -> AnyElement {
    v_flex()
        .gap_4()
        .w_full()
        .max_w(px(720.))
        .child(div().font_semibold().text_lg().child("İndirmeler"))
        .child(new_download_card(view, cx))
        .child(job_list_card(cx))
        .into_any_element()
}

/// Yeni indirme formu: URL girişi + ses/vidyo seçimi + İndir düğmesi.
fn new_download_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let audio_only = view.down_audio;
    card(cx)
        .gap_3()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Download)
                .child(div().font_semibold().text_sm().child("Medya indir"))
                .child({
                    let active = cx.global::<DownloadState>().active_count();
                    div()
                        .text_xs()
                        .px_1p5()
                        .rounded_full()
                        .bg(cx.theme().accent.opacity(0.4))
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} sıra", active))
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Input::new(&view.download_url_input).flex_1())
                .child(
                    Button::new("download-submit")
                        .label("İndir")
                        .primary()
                        .on_click(cx.listener(move |view, _, window, cx| {
                            let url = view.download_url_input.read(cx).value().trim().to_string();
                            if url.is_empty() {
                                return;
                            }
                            let audio = view.down_audio;
                            downloads::submit(url, audio);
                            view.download_url_input.update(cx, |input, cx| {
                                input.set_value("", window, cx);
                            });
                            cx.notify();
                        })),
                ),
        )
        .child(
            Checkbox::new("download-audio-only")
                .label("Sadece ses (MP3 olarak indir)")
                .checked(audio_only)
                .on_change(cx.listener(move |view, checked, _, cx| {
                    view.down_audio = *checked;
                    cx.notify();
                })),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "YouTube, Instagram, Vimeo, TikTok ve diğer tüm yt-dlp kaynakları \
                     desteklenir. Dosyalar İndirilenler\\personal-dashboard klasörüne kaydedilir.",
                ),
        )
}

/// Mevcut indirme işleri: durum, yüzde ve hata metinleri.
fn job_list_card(cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    let entries = cx.global::<DownloadState>().snapshot();
    let mut card = card(cx).gap_2().child(
        h_flex()
            .gap_2()
            .items_center()
            .child(IconName::Clock)
            .child(div().font_semibold().text_sm().child("İşler")),
    );

    if entries.is_empty() {
        card = card.child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Henüz bir indirme yok. Yukarıya bir bağlantı yapıştır."),
        );
        return card;
    }

    for e in entries.into_iter().take(8) {
        let pct = (e.progress.clamp(0., 1.) * 100.).round() as u32;
        let kind = if e.audio_only { "ses" } else { "vidyo" };
        let (status_text, status_color) = match e.status {
            DlStatus::Pending => ("Bekliyor".to_string(), cx.theme().muted_foreground),
            DlStatus::Running => (format!("{}% · İndiriliyor", pct), cx.theme().primary),
            DlStatus::Done => ("Tamam".to_string(), cx.theme().success),
            DlStatus::Failed => ("Hata".to_string(), cx.theme().danger),
        };
        let mut row = h_flex()
            .gap_2()
            .items_center()
            .py_1()
            .child(
                div()
                    .w(px(8.))
                    .h(px(8.))
                    .rounded_full()
                    .bg(status_color)
                    .flex_shrink_0(),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_sm()
                    .when(e.title.is_empty(), |t| {
                        t.child(
                            div()
                                .truncate()
                                .child(e.url.chars().take(50).collect::<String>()),
                        )
                    })
                    .when(!e.title.is_empty(), |t| {
                        t.child(div().truncate().child(e.title.clone()))
                    }),
            )
            .child(
                div()
                    .text_xs()
                    .whitespace_nowrap()
                    .text_color(cx.theme().muted_foreground)
                    .child(kind.to_string() + " · " + &e.started),
            )
            .child(
                div()
                    .text_xs()
                    .text_right()
                    .whitespace_nowrap()
                    .text_color(status_color)
                    .child(status_text),
            );
        if let Some(err) = &e.error {
            row = row.child(
                div()
                    .w_full()
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child(err.clone()),
            );
        } else if let Some(file) = &e.file {
            row = row.child(
                div()
                    .w_full()
                    .text_xs()
                    .truncate()
                    .text_color(cx.theme().muted_foreground)
                    .child(file.clone()),
            );
        }
        card = card.child(row);
    }
    card
}
