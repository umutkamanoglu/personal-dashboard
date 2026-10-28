//! yt-dlp tabanlı medya indirme motoru (youtube, instagram, vimeo, tiktok…).
//!
//! `yt-dlp` crate'i yt-dlp ve ffmpeg ikili dosyalarını otomatik indirir;
//! terminale dokunma yok. İşler bir arka plan iş parçacığındaki tokio
//! runtime'ında sırayla işlenir; durumlar gpui Global'i (`DownloadState`)
//! üzerinden arayüze aktarılır.

use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use yt_dlp::download::config::postprocess::{AudioCodec, PostProcessConfig};
use yt_dlp::model::Video;
use yt_dlp::Downloader;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct DownloadEntry {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub audio_only: bool,
    pub status: DlStatus,
    /// 0..=1; yt-dlp ilerleme geri araması bulursa doldurulur
    pub progress: f32,
    pub file: Option<String>,
    pub error: Option<String>,
    /// başlatılma zamanı ("27.10 14:22")
    pub started: String,
}

impl gpui_kit::Global for DownloadState {}

#[derive(Default)]
pub struct DownloadState {
    pub entries: Arc<Mutex<Vec<DownloadEntry>>>,
    pub jobs: Arc<Mutex<Vec<Job>>>,
    /// altyapı kurulumunda (ikili indirme) oluşan hata
    pub engine_error: Arc<Mutex<Option<String>>>,
}

impl DownloadState {
    /// İndirme listesi kopyası (en yenisi başta).
    pub fn snapshot(&self) -> Vec<DownloadEntry> {
        let mut v = self.entries.lock().map(|e| e.clone()).unwrap_or_default();
        v.reverse();
        v
    }

    /// Devam eden iş sayısı (kart rozeti için).
    pub fn active_count(&self) -> usize {
        self.snapshot()
            .iter()
            .filter(|e| matches!(e.status, DlStatus::Pending | DlStatus::Running))
            .count()
    }
}

pub struct Job {
    pub url: String,
    pub audio_only: bool,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// İndirme işi kuyruğa alınır; arka plan motoru sırayla işler.
pub fn submit(url: String, audio_only: bool) {
    // Global gpui'de kayıtlı; gpui'nin app yazıcısı yoksa sessizce çıkar.
    let Ok(mut list) = JOBS.lock() else {
        return;
    };
    list.push(Job { url, audio_only });
}

/// Global kuyruk (gpui Global'i kurulmadan önce motor döngüsü kendi kuruşunu kullanır).
static JOBS: Mutex<Vec<Job>> = Mutex::new(Vec::new());

fn clean_name(title: &str) -> String {
    title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .chars()
        .take(120)
        .collect::<String>()
}

fn now_stamp() -> String {
    chrono::Local::now().format("%m.%d %H:%M").to_string()
}

fn set_entry<F: FnOnce(&mut DownloadEntry)>(entries: &Mutex<Vec<DownloadEntry>>, id: u64, f: F) {
    if let Ok(mut all) = entries.lock() {
        if let Some(e) = all.iter_mut().find(|e| e.id == id) {
            f(e);
        }
    }
}

/// Uygulama açılışında çağrılır: motor iş parçacığını başlatır.
pub fn start() -> DownloadState {
    let state = DownloadState {
        entries: Arc::new(Mutex::new(Vec::new())),
        jobs: Arc::new(Mutex::new(Vec::new())),
        engine_error: Arc::new(Mutex::new(None)),
    };
    let entries = state.entries.clone();
    let engine_error = state.engine_error.clone();
    let jobs_global = state.jobs.clone();

    std::thread::Builder::new()
        .name("download-worker".into())
        .spawn(move || {
            let base = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("personal-dashboard");
            let libs_dir = base.join("libs");
            let out_dir = dirs::download_dir()
                .unwrap_or_else(|| base.clone())
                .join("personal-dashboard");
            let _ = std::fs::create_dir_all(&out_dir);

            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    if let Ok(mut s) = engine_error.lock() {
                        *s = Some(format!("tokio runtime kurulamadı: {e}"));
                    }
                    return;
                }
            };

            // tek runtime'da tüm indirmeler sırayla
            rt.block_on(async {
                // altyapıyı başta kur: yoksa yt-dlp + ffmpeg ikili dosyaları
                // ağdan otomatik iner (LibraryInstaller), varsa aynen kullanılır
                let builder =
                    match Downloader::with_new_binaries(libs_dir.clone(), out_dir.clone()).await {
                        Ok(b) => b,
                        Err(e) => {
                            if let Ok(mut s) = engine_error.lock() {
                                *s = Some(format!("yt-dlp kurulumu: {e}"));
                            }
                            // motor olmadan iş kabul edemeyiz; gelen işleri hata yaparız
                            loop {
                                drain_jobs_to_error(&entries, &engine_error).await;
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            }
                        }
                    };
                let downloader = match builder
                    .with_user_agent(
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                         (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                    )
                    .build()
                    .await
                {
                    Ok(d) => d,
                    Err(e) => {
                        if let Ok(mut s) = engine_error.lock() {
                            *s = Some(format!("yt-dlp hazırlığı: {e}"));
                        }
                        loop {
                            drain_jobs_to_error(&entries, &engine_error).await;
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        }
                    }
                };

                loop {
                    // global kuyruk + yerel kuyruk birleşik besleme
                    let job_local = jobs_global.lock().ok().and_then(|mut q| {
                        if !q.is_empty() {
                            Some(q.remove(0))
                        } else {
                            None
                        }
                    });
                    let job_static = JOBS.lock().ok().and_then(|mut q| {
                        if !q.is_empty() {
                            Some(q.remove(0))
                        } else {
                            None
                        }
                    });
                    let Some(job) = job_static.or(job_local) else {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        continue;
                    };

                    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                    let stamp = now_stamp();
                    if let Ok(mut list) = entries.lock() {
                        list.push(DownloadEntry {
                            id,
                            url: job.url.clone(),
                            title: job.url.chars().take(60).collect(),
                            audio_only: job.audio_only,
                            status: DlStatus::Pending,
                            progress: 0.,
                            file: None,
                            error: None,
                            started: stamp,
                        });
                    }
                    run_job(
                        &downloader,
                        &out_dir,
                        &libs_dir,
                        id,
                        &job.url,
                        job.audio_only,
                        entries.clone(),
                    )
                    .await;
                }
            });
        })
        .ok();
    state
}

async fn drain_jobs_to_error(
    entries: &Mutex<Vec<DownloadEntry>>,
    engine_error: &Mutex<Option<String>>,
) {
    let jobs: Vec<Job> = JOBS
        .lock()
        .ok()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    if jobs.is_empty() {
        return;
    }
    let msg = engine_error
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .unwrap_or_else(|| "indirme motoru kurulamadı".to_string());
    for job in jobs {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut list) = entries.lock() {
            list.push(DownloadEntry {
                id,
                url: job.url,
                title: "(indirilemedi)".into(),
                audio_only: job.audio_only,
                status: DlStatus::Failed,
                progress: 0.,
                file: None,
                error: Some(msg.clone()),
                started: now_stamp(),
            });
        }
    }
}

/// Tek iş: bilgiyi al, indir (vidyo) ya da ses çıkarıp mp3'e dönüştür.
#[allow(clippy::too_many_arguments)]
async fn run_job(
    downloader: &Downloader,
    out_dir: &Path,
    libs_dir: &Path,
    id: u64,
    url: &str,
    audio_only: bool,
    entries: Arc<Mutex<Vec<DownloadEntry>>>,
) {
    set_entry(&entries, id, |e| {
        e.status = DlStatus::Running;
    });

    let video = match downloader.fetch_video_infos(url).await {
        Ok(v) => v,
        Err(e) => {
            set_entry(&entries, id, |en| {
                en.status = DlStatus::Failed;
                en.error = Some(format!("bilgi alınamadı: {e}"));
            });
            return;
        }
    };

    set_entry(&entries, id, |e| {
        e.title = video_title(&video);
    });

    if audio_only {
        // ses: ham akış → ffmpeg ile mp3
        match downloader
            .download_audio_stream(&video, "audio_raw.webm")
            .await
        {
            Ok(raw_path) => {
                let out_name = clean_name(&video_title(&video)) + ".mp3";
                let config = PostProcessConfig::new()
                    .with_audio_codec(AudioCodec::MP3)
                    .with_audio_bitrate("192k");
                match downloader
                    .postprocess_audio(raw_path, &out_name, config)
                    .await
                {
                    Ok(final_path) => set_entry(&entries, id, |en| {
                        en.status = DlStatus::Done;
                        en.progress = 1.;
                        en.file = Some(final_path.display().to_string());
                    }),
                    Err(e) => set_entry(&entries, id, |en| {
                        en.status = DlStatus::Failed;
                        en.error = Some(format!("dönüştürme: {e}"));
                    }),
                }
            }
            Err(e) => {
                match cli_fallback(libs_dir, url, &clean_name(&video_title(&video)), true).await {
                    Ok(final_path) => set_entry(&entries, id, |en| {
                        en.status = DlStatus::Done;
                        en.progress = 1.;
                        en.file = Some(final_path.display().to_string());
                    }),
                    Err(_fb) => set_entry(&entries, id, |en| {
                        en.status = DlStatus::Failed;
                        en.error = Some(format!("ses akışı: {e}"));
                    }),
                }
            }
        }
    } else {
        // vidyo: en iyi kalite + canlı yüzde (ilerleme geri araması)
        let out_name = clean_name(&video_title(&video)) + ".mp4";
        let cb_entries = entries.clone();
        let cb_id = id;
        let result = downloader
            .download(&video, out_name.clone())
            .with_progress(move |f| {
                set_entry(&cb_entries, cb_id, |en| en.progress = f as f32);
            })
            .execute()
            .await;
        match result {
            Ok(final_path) => set_entry(&entries, id, |en| {
                en.status = DlStatus::Done;
                en.progress = 1.;
                en.file = Some(final_path.display().to_string());
            }),
            Err(e) => {
                match cli_fallback(libs_dir, url, &clean_name(&video_title(&video)), false).await {
                    Ok(final_path) => set_entry(&entries, id, |en| {
                        en.status = DlStatus::Done;
                        en.progress = 1.;
                        en.file = Some(final_path.display().to_string());
                    }),
                    Err(_fb) => set_entry(&entries, id, |en| {
                        en.status = DlStatus::Failed;
                        en.error = Some(format!("indirme: {e}"));
                    }),
                }
            }
        }
    }

    let _ = out_dir;
}

/// Dahili yt-dlp.exe fallback'i: belirli CDN'ler (ör. TikTok) TLS parmak izine
/// bakıp reqwest'e 403 döndürür; CLI ikilisi curl_cffi impersonation ile başarılı.
/// Terminale dokunmaz: ikili dosyanın kendisini süreç olarak çalıştırır.
async fn cli_fallback(
    libs_dir: &Path,
    url: &str,
    stem: &str,
    audio_only: bool,
) -> Result<PathBuf, String> {
    let exe = libs_dir.join("yt-dlp.exe");
    if !exe.exists() {
        return Err("yt-dlp.exe bulunamadı".to_string());
    }
    let dir = libs_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let target_dir = dirs::download_dir()
        .unwrap_or(dir)
        .join("personal-dashboard");
    let _ = std::fs::create_dir_all(&target_dir);
    let template = target_dir.join(format!("{stem}.%(ext)s"));

    let mut cmd = tokio::process::Command::new(&exe);
    if audio_only {
        cmd.arg("-x")
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg("192K");
    } else {
        cmd.arg("-N")
            .arg("4")
            .arg("-f")
            .arg("bestvideo*+bestaudio/best");
    }
    cmd.arg("--ffmpeg-location")
        .arg(libs_dir.join("ffmpeg.exe"))
        .arg("--no-playlist")
        .arg("--no-part")
        .arg("-o")
        .arg(&template)
        .arg(url);

    let out = cmd
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| format!("çalıştırma: {e}"))?;
    if !out.status.success() {
        let tail = String::from_utf8_lossy(&out.stderr);
        let tail: Vec<&str> = tail.lines().rev().take(3).collect();
        let tail = tail.into_iter().rev().collect::<Vec<_>>().join(" | ");
        return Err(format!("yt-dlp (fallback): {tail}"));
    }

    // `<stem>.<ext>` üretildi; birebir eşleşen en yeni dosyayı bul
    let mut found: Option<PathBuf> = None;
    if let Ok(rd) = std::fs::read_dir(&target_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(stem)
                && found.as_ref().is_none_or(|f| {
                    e.metadata().ok().and_then(|m| m.modified().ok())
                        > std::fs::metadata(f).ok().and_then(|m| m.modified().ok())
                })
            {
                found = Some(e.path());
            }
        }
    }
    found.ok_or_else(|| "dosya bulunamadı (fallback)".to_string())
}

fn video_title(video: &Video) -> String {
    video.title.clone()
}
