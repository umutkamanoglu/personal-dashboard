# ⚡ Personal Dashboard

Rust + GPUI ile yazılmış kişisel kart arayüzü.

Türkçe arayüz, sistem temasını izleyen açık/koyu tema, özelleştirilebilir kart ızgarası, medya denetimi, alışkanlık takibi, sistem sensörleri ve yt-dlp ile hızlı video/ses indirmeleri.

---

## 🌟 Öne Çıkanlar

| | |
|---|---|
| 🗓️ **Anasayfa ızgarası** | Saat, program ve durum kartları. Ayarlar'da canlı önizlemeli kart koleksiyonu: ekle/çıkar, sırala, 1/2/3 sütun seç |
| 🌗 **Tema** | Açık / Koyu / **Sistem** (Windows'un açık-koyu tercihini izler) |
| 📅 **Saatler** | Haftalık zaman çizelgesi, canlı "şimdi" çizgisi, fare tekerleğiyle zoom (0.5×–4×) |
| ✅ **Alışkanlıklar** | Günlük / haftalık / aylık frekansta takip, seri (streak) göstergesi |
| 🔁 **Rutinler** | Her gün sıfırlanan günlük rutin listesi |
| 📊 **İstatistik** | Son 7 gün tamamlama sütun grafiği + alışkanlık başına dönem geçmişi |
| ⏱️ **Pomodoro** | 15/25/50 dk odak, otomatik mola, saniyelik geri sayım |
| 🎵 **Medya denetimi** | SMTC ile Spotify / tarayıcı / VLC kontrolü: kapak, parça, ileri-geri, ana ses slider'ı ve mute |
| 📥 **İndirmeler** | yt-dlp ile YouTube / Instagram / Vimeo / TikTok… MP4 veya MP3; **"Bilgi Al"** adımı kapak + başlık gösterir, sonra tür seçilir |
| 🌈 **Görselleştirici** | WASAPI loopback + FFT ile canlı frekans barları (renk, ayna, tepe seçenekleri) |
| ⛅ **Hava durumu** | Anlık durum + saatlik + 5 günlük tahmin kartları; Open-Meteo (anahtarsız) veya OpenWeatherMap (anahtar Ayarlar'dan) |
| 🕐 **Büyük saat** | Peaclock tarzı dev saat kartı: 24/12 sa, saniye, tarih, rakam rengi |
| 💾 **Yedekleme** | Bütün verinin tek JSON'da dışa/içe aktarımı; veri sıfırlama (stok seed) |

## 🖥️ Sistem Bilgisi

Çekirdek başına CPU kullanımı + sıcaklık · RAM · NVIDIA GPU (NVML): kullanım / VRAM / güç / fan · disk dolulukları · tüm donanım sıcaklık sensörleri · ağ (anlık indirme/yükleme hızları + oturum toplamı)

## 📦 Kurulum

Gereksinim: [Rust](https://rustup.rs).

```bash
git clone https://github.com/umutkamanoglu/personal-dashboard
cd personal-dashboard
cargo run --release
```

Ayarlar ve tüm takipler `%APPDATA%\personal-dashboard\data.json` dosyasına JSON olarak kaydedilir. İndirme motorunun ikili dosyaları (`yt-dlp.exe`, `ffmpeg.exe`) ilk indirmede otomatik indirilir (`%APPDATA%\personal-dashboard\libs`) — kullanıcı terminal kullanmaz. İndirme klasörü Ayarlar'dan değiştirilebilir.

## 🏗️ Mimari

```
src/
├── main.rs          açılış, tema ve motor kurulumu
├── app.rs           görünüm, sayfa yönlendirme, yan menü
├── data.rs          DashboardData: tüm kalıcılık (JSON), yedekleme/sıfırlama
├── audio.rs         WASAPI loopback görselleştirici + ana ses (MTA COM)
├── media.rs         SMTC medya yoklaması
├── hardware.rs      sysinfo + NVML
├── downloads.rs     yt-dlp indirme motoru ve CLI fallback
└── pages/           home, schedule, habits, routines, stats,
                     notes, pomodoro, system, downloads, settings
```

## ⚙️ İndirme Motoru

Uygulama [`yt-dlp`](https://crates.io/crates/yt-dlp) crate'ini kullanır ve TikTok gibi TLS parmak izi isteyen CDN'lere karşı iki önlem içerir:

- Crate, `patches/yt-dlp` içinde yamalanıp `[patch.crates-io]` ile bağlanır: bazı çıkarıcıların döndürmediği `age_limit` / `live_status` / `playable_in_embed` alanları ve format bazlı `Referer` / `cookies` başlıkları artık esnetilmiş Deserialize ile çözülür
- Native indirme 403'e düşerse motor aynı klasördeki `yt-dlp.exe`'yi (curl_cffi chrome impersonation) süreç olarak çalıştırır — terminal görünmez, akış aynıdır

## 🗺️ Yol Haritası

- [ ] haftalık/aylık özet e-postası veya dışa aktarma (CSV)
- [x] alışkanlık için ısı haritası görünümü
- [x] hava durumu kartları (open-meteo / OpenWeatherMap, konum + anahtar ayarları)
- [ ] indirme kuyruğunda yeniden sıralama ve iptal

## 📜 Lisans

Kişisel kullanım ve öğrenme amaçlıdır.
