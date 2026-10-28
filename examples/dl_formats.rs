//! Format sınıflaması elle doğrulama.
//! `cargo run --example dl_formats -- <URL>`

use yt_dlp::Downloader;

fn main() {
    let url = std::env::args().nth(1).expect("URL gerekli");
    let base = dirs::data_dir().unwrap().join("personal-dashboard");
    let libs = base.join("libs");
    let out = dirs::download_dir()
        .unwrap_or(base)
        .join("personal-dashboard");

    let dl = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(Downloader::with_new_binaries(libs, out))
    {
        Ok(b) => match tokio::runtime::Runtime::new().unwrap().block_on(
            b.with_user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            )
            .build(),
        ) {
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
        for f in video.formats.iter().take(20) {
            println!(
                "FMT id={} note={:?} atype={:?} vtype={:?} proto={:?} url_ok={}",
                f.format_id,
                f.format_note,
                f.codec_info.audio_codec,
                f.codec_info.video_codec,
                f.protocol,
                f.download_info.url.is_some()
            );
        }
    });
}
