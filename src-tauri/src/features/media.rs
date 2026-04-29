use serde::Serialize;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

#[derive(Serialize, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album_art: String, // Boş string olarak dönecek
    pub is_playing: bool,
}

pub async fn get_current_media_info() -> Option<MediaInfo> {
    // Manager ve Session'ı alıyoruz
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .ok()?
        .await
        .ok()?;
    let session = manager.GetCurrentSession().ok()?;

    // Temel özellikleri çekiyoruz (Bunlar Send uyumludur)
    let info = session.TryGetMediaPropertiesAsync().ok()?.await.ok()?;
    let playback = session.GetPlaybackInfo().ok()?;

    Some(MediaInfo {
        title: info.Title().unwrap_or_default().to_string(),
        artist: info.Artist().unwrap_or_default().to_string(),
        album_art: String::new(), // Thumbnail işlemleri kaldırıldı
        is_playing: playback.PlaybackStatus().ok()?.0 == 4,
    })
}

pub async fn process_media_command(command: String) {
    let op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().ok();
    if let Some(m_async) = op {
        if let Ok(manager) = m_async.await {
            if let Ok(session) = manager.GetCurrentSession() {
                match command.as_str() {
                    "toggle" => {
                        let _ = session.TryTogglePlayPauseAsync();
                    }
                    "next" => {
                        let _ = session.TrySkipNextAsync();
                    }
                    "prev" => {
                        let _ = session.TrySkipPreviousAsync();
                    }
                    _ => {}
                }
            }
        }
    }
}
