//! TikTok indirme akışı elle doğrulama (gerçek ağ + ikili kurulum).
//! `cargo run --example dl_tiktok -- <tiktok-URL>`

use yt_dlp::download::config::postprocess::{AudioCodec, PostProcessConfig};
use yt_dlp::Downloader;

fn main() {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://www.tiktok.com/@tiktok/video/7106594312292457478".to_string());
    let base = dirs::data_dir().unwrap().join("personal-dashboard");
    let libs = base.join("libs");
    let out = dirs::download_dir()
        .unwrap_or(base)
        .join("personal-dashboard");
    println!("OUT DIR: {}", out.display());

    let dl = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(Downloader::with_new_binaries(libs, out.clone()))
    {
        Ok(b) => match tokio::runtime::Runtime::new().unwrap().block_on(b.build()) {
            Ok(d) => d,
            Err(e) => {
                println!("BUILD ERR: {e}");
                return;
            }
        },
        Err(e) => {
            println!("INSTALL ERR: {e}");
            return;
        }
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let video = match dl.fetch_video_infos(&url).await {
            Ok(v) => v,
            Err(e) => {
                println!("FETCH ERR: {e}");
                return;
            }
        };
        println!("TITLE: {}", video.title);
        for f in video.formats.iter() {
            println!(
                "FMT id={} atype={:?} vtype={:?} url_ok={}",
                f.format_id,
                f.codec_info.audio_codec,
                f.codec_info.video_codec,
                f.download_info.url.is_some()
            );
        }
        let out_name = format!("{}.mp4", video.title.chars().take(40).collect::<String>());
        match dl.download_video(&video, out_name).await {
            Ok(p) => println!("OK VIDEO: {}", p.display()),
            Err(e) => println!("DOWNLOAD ERR: {e}"),
        }
        match dl.download_audio_stream(&video, "audio_raw.webm").await {
            Ok(raw) => {
                let cfg = PostProcessConfig::new()
                    .with_audio_codec(AudioCodec::MP3)
                    .with_audio_bitrate("192k");
                match dl.postprocess_audio(raw, "deneme.mp3", cfg).await {
                    Ok(p) => println!("OK AUDIO: {}", p.display()),
                    Err(e) => println!("AUDIO PP ERR: {e}"),
                }
            }
            Err(e) => println!("AUDIO STREAM ERR: {e}"),
        }
    });
}
