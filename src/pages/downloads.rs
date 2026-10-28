use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    radio::RadioGroup,
    v_flex, ActiveTheme, Disableable as _, Sizable as _,
};
use gpui_kit::img;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, Context, ImageSource, IntoElement as _, ParentElement as _, Styled as _,
};
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
        .child(preview_card(view, cx))
        .child(job_list_card(cx))
        .into_any_element()
}

/// Yeni indirme formu: URL girişi + "Bilgi Al" (önizleme) → tür seçimi + İndir.
fn new_download_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
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
                    Button::new("download-info")
                        .small()
                        .icon(IconName::Search)
                        .label(if view.preparing {
                            "Getiriliyor…"
                        } else {
                            "Bilgi Al"
                        })
                        .disabled(view.preparing)
                        .on_click(cx.listener(move |view, _, _, cx| {
                            let url = view.download_url_input.read(cx).value().trim().to_string();
                            if url.is_empty() || view.preparing {
                                return;
                            }
                            view.preparing = true;
                            view.preview_error = None;
                            if let Ok(mut p) = cx.global::<DownloadState>().preview.lock() {
                                *p = None;
                            }
                            cx.notify();
                            cx.spawn(async move |this, cx| {
                                let prep = cx
                                    .background_executor()
                                    .spawn(async move { downloads::prepare_preview(url) })
                                    .await;
                                this.update(cx, |view, cx| {
                                    view.preparing = false;
                                    match prep {
                                        Ok(p) => {
                                            if let Ok(mut slot) =
                                                cx.global::<DownloadState>().preview.lock()
                                            {
                                                *slot = Some(p);
                                            }
                                            view.preview_error = None;
                                        }
                                        Err(e) => {
                                            view.preview_error =
                                                Some(format!("Bilgi alınamadı: {e}"))
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
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "YouTube, Instagram, Vimeo, TikTok ve tüm yt-dlp kaynakları desteklenir. \
                     \"Bilgi Al\" başlık ve kapağı gösterir; ardından neyin ineceğini seçersin.",
                ),
        )
}

/// Önizleme: kapak + başlık + kanal; tür seçimi ve indirme onayı.
fn preview_card(view: &DashboardView, cx: &mut Context<DashboardView>) -> gpui_kit::Div {
    if let Some(err) = &view.preview_error {
        return card(cx).gap_2().child(
            div()
                .text_sm()
                .text_color(cx.theme().danger)
                .child(err.clone()),
        );
    }
    let Some(preview) = cx
        .global::<DownloadState>()
        .preview
        .lock()
        .ok()
        .and_then(|p| {
            p.as_ref().map(|p| {
                (
                    p.title.clone(),
                    p.channel.clone(),
                    p.thumb.clone(),
                    p.url.clone(),
                )
            })
        })
    else {
        return card(cx).gap_2().child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Önizleme yok — bağlantı yapıştırıp \"Bilgi Al\"."),
        );
    };
    let (title, channel, thumb, url) = preview;

    let mut card = card(cx).gap_3().child(
        h_flex()
            .gap_3()
            .items_center()
            .when_some(thumb, |r, image| {
                r.child(
                    img(ImageSource::Image(image))
                        .size(px(96.))
                        .rounded_sm()
                        .flex_shrink_0(),
                )
            })
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .truncate()
                            .child(title.clone()),
                    )
                    .when_some(channel, |r, channel| {
                        r.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(channel),
                        )
                    }),
            ),
    );

    card = card.child(
        v_flex()
            .gap_2()
            .child(
                RadioGroup::new("download-kind")
                    .children(["Video (MP4)", "Sadece ses (MP3)"])
                    .selected_index(Some(if view.down_audio { 1 } else { 0 }))
                    .on_change(cx.listener(move |view, index, _, cx| {
                        view.down_audio = *index == 1;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("download-go")
                    .small()
                    .primary()
                    .icon(IconName::Download)
                    .label("İndir")
                    .on_click(cx.listener(move |view, _, window, cx| {
                        let audio = view.down_audio;
                        downloads::submit(url.clone(), audio, Some(title.clone()));
                        // girilen adı temizle, önizlemeyi kapat
                        view.download_url_input.update(cx, |input, cx| {
                            input.set_value("", window, cx);
                        });
                        if let Ok(mut p) = cx.global::<DownloadState>().preview.lock() {
                            *p = None;
                        }
                        cx.notify();
                    })),
            ),
    );
    card
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
