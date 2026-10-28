//! Windows SMTC (Global System Media Transport Controls) üzerinden
//! çalan medyanın bilgisi + oynatma kontrolü.
//!
//! `MediaState` bir gpui Global'i; saniyelik tick içinde arka planda
//! yoklanır (poll), MediaCmd ise düğme tıklamalarında arka planda yollanır.

use std::sync::Arc;

use gpui_kit::component::slider::SliderState;
use gpui_kit::Entity;

use windows::core::Result;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionMediaProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

#[derive(Default)]
pub struct MediaState {
    /// aktif medya oturumu var mı
    pub has_session: bool,
    /// çalıyor mu (false = duraklatılmış)
    pub playing: bool,
    /// oynatan uygulama (mümkünse dostane isim, örn. Spotify)
    pub app: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// kapak resmi (png/jpg baytları), gpui Image olarak hazır
    pub thumb: Option<Arc<ThumbImage>>,
    /// örnekleme hatası varsa (kartta göstermek için)
    pub error: Option<String>,
    /// ana ses seviyesi (0..=1), yoklanırken güncellenir
    pub volume: f32,
    /// ses kapalı mı
    pub muted: bool,
    /// medya kartındaki ses slider state'i (açılışta kurulur)
    pub volume_slider: Option<Entity<SliderState>>,
}

/// Kapak resmi baytları (ID, format, veri).
pub struct ThumbImage {
    pub id: u64,
    pub format: &'static str, // "png" veya "jpeg"
    pub bytes: Vec<u8>,
}

impl gpui_kit::Global for MediaState {}

#[derive(Debug, Clone, Copy)]
pub enum MediaCmd {
    Toggle,
    Next,
    Prev,
}

fn friendly_app(id: &str) -> String {
    let l = id.to_lowercase();
    let known = [
        ("spotify", "Spotify"),
        ("chromium", "Tarayıcı"),
        ("chrome", "Chrome"),
        ("msedge", "Edge"),
        ("firefox", "Firefox"),
        ("brave", "Brave"),
        ("opera", "Opera"),
        ("ytmusic", "YouTube Music"),
        ("youtube", "YouTube"),
        ("deezer", "Deezer"),
        ("tidal", "Tidal"),
        ("amazonmusic", "Amazon Music"),
        ("soundcloud", "SoundCloud"),
        ("wmplayer", "Windows Media Player"),
        ("vlc", "VLC"),
        ("foobar", "foobar2000"),
        ("music.ui", "Groove"),
        ("zunemusic", "Groove"),
        ("netease", "NetEase"),
        ("qqmusic", "QQ Music"),
    ];
    for (needle, name) in known {
        if l.contains(needle) {
            return name.to_string();
        }
    }
    // AUMID: "Publisher.AppName!App" → sondaki anlamlı kısım
    l.rsplit('!').next().unwrap_or(id).to_string()
}

fn session_manager() -> Result<GlobalSystemMediaTransportControlsSessionManager> {
    GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()
}

fn hstr(r: windows::core::Result<windows::core::HSTRING>) -> String {
    r.map(|h| h.to_string()).unwrap_or_default()
}

/// SMTC durumunu yoklar (arka plan iş parçacığında çalışır, bloklar).
pub fn poll() -> MediaState {
    let mut state = MediaState {
        error: None,
        ..Default::default()
    };

    let run = || -> Result<MediaState> {
        // ses seviyesi: WASAPI endpoint (Win32) — WinRT denemesinden bağımsız
        let (volume, muted) = crate::audio::master_volume().unwrap_or((0., false));

        let mgr = session_manager()?;
        let session: GlobalSystemMediaTransportControlsSession = mgr.GetCurrentSession()?;

        let props: GlobalSystemMediaTransportControlsSessionMediaProperties =
            session.TryGetMediaPropertiesAsync()?.join()?;

        let playing = session.GetPlaybackInfo()?.PlaybackStatus()?
            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        let app = friendly_app(
            &session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string(),
        );

        // kapak: IRandomAccessStreamReference -> byte dizisi
        let thumb = props.Thumbnail().ok().and_then(|st_ref| {
            let st = st_ref.OpenReadAsync().ok()?.join().ok()?;
            let size = st.Size().ok()?;
            let reader = DataReader::CreateDataReader(&st).ok()?;
            let loaded = reader.LoadAsync(size as u32).ok()?.join().ok()? as usize;
            let mut buf = vec![0u8; loaded];
            reader.ReadBytes(&mut buf).ok()?;
            let format = if buf.starts_with(&[0x89, b'P', b'N', b'G']) {
                "png"
            } else {
                "jpeg"
            };
            // kapak önbelleği: içerik karmasından üretilir
            let id = content_hash(&buf);
            Some(Arc::new(ThumbImage {
                id,
                format,
                bytes: buf,
            }))
        });

        Ok(MediaState {
            has_session: true,
            playing,
            app,
            title: hstr(props.Title()),
            artist: hstr(props.Artist()),
            album: hstr(props.AlbumTitle()),
            thumb,
            error: None,
            volume,
            muted,
            ..Default::default()
        })
    };

    match run() {
        Ok(s) => s,
        Err(e) => {
            state.error = if e.to_string().contains("0x80131512") {
                None
            } else {
                Some(e.to_string())
            };
            // medya yoksa bile ses seviyesi bilgisi güncel kalsın
            if let Some((v, m)) = crate::audio::master_volume() {
                state.volume = v;
                state.muted = m;
            }
            state
        }
    }
}

fn content_hash(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    // FNV-1a (ucuz, çakışma riski pratikte önemsiz)
    for (i, byte) in b.iter().take(4096).enumerate() {
        if i % 97 == 0 {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h = h.wrapping_mul(b.len() as u64 | 1);
    h
}

/// Düğmelerden gelen komutu arka planda yollar.
pub fn send(cmd: MediaCmd) {
    let run = || -> Result<()> {
        let mgr = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()?;
        let session = mgr.GetCurrentSession()?;
        match cmd {
            MediaCmd::Next => {
                let _ = session.TrySkipNextAsync()?.join()?;
            }
            MediaCmd::Prev => {
                let _ = session.TrySkipPreviousAsync()?.join()?;
            }
            MediaCmd::Toggle => {
                let status = session.GetPlaybackInfo()?.PlaybackStatus()?;
                if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                    let _ = session.TryPauseAsync()?.join()?;
                } else {
                    let _ = session.TryPlayAsync()?.join()?;
                }
            }
        }
        Ok(())
    };
    // hata önemli değil; sonraki poll yeni durumu getirir
    let _ = run();
}
