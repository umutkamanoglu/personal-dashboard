# Personal Dashboard

Rust + GPUI ile yazılmış kişisel dashboard uygulaması.
UI katmanı [GPUI Kit](https://github.com/longbridge/gpui-component) (`gpui-kit` crate) üzerine kurulu.

## Özellikler

- **Anasayfa:** saat, günün programı ve genel durum kartları ile kart grid düzeni; tüm kartlar tıklanabilir. **Kart koleksiyonu:** Ayarlar'dan kart ekle/çıkar, sırala, genişliğini seç (1/2/3 sütun — 3 = tam satır); granüler CPU/RAM/GPU/disk/sıcaklık/bilgisayar/ağ kartları da koleksiyonda; ayarlarda kartların canlı önizlemesi var, tıklayınca eklenir

- **Ayarlar:** kart artık grid düzeninde satır satır dizilir (profil+tema, pomodoro+büyük saat yan yana), görselleştirici ve kart koleksiyonu tam satır
- **Saatler:** günde hangi saatte ne var? haftalık zaman çizelgesi (timeline) görünümü, canlı "şimdi" çizgisi; fare tekerleğiyle zoom (0.5×–4×)
- **Alışkanlık takibi:** günlük, haftalık ve aylık frekansta alışkanlıklar; seri (streak) göstergesi
- **Günlük rutinler:** her gün sıfırlanan rutin checklist'i
- **İstatistik:** son 7 gün tamamlama sütun grafiği ve alışkanlık başına dönem geçmişi
- **Tema:** sağ üstteki düğmeyle açık/koyu tema geçişi (tercih kaydedilir)
- **Pomodoro:** 15/25/50 dk odak + 5 dk otomatik mola, saniyelik geri sayım, ani geçiş ve tur sayısı
- **Notlar & hatırlatıcılar:** serbest notlar ve haftanın günlerine atanan hatırlatıcılar; bugün olanlar anasayfada zil ikonuyla listelenir
- **Sistem bilgisi:** CPU (model/frekans/çekirdek başına kullanım/sıcaklık), RAM, GPU (NVML üzerinden NVIDIA; kullanım/VRAM/Güç/fan), disk dolulukları, tüm sıcaklık sensörleri, **ağ (anlık indirme/yükleme hızları + oturum toplamı)** ve genel bilgisayar bilgileri — anasayfada özet kartı da var
- **Medya kontrolü:** Windows SMTC üzerinden herhangi bir uygulamanın (Spotify, tarayıcı, Groove, VLC…) çaldığı müzik anasayfadaki kartta görünür: çalan uygulama, parça/sanatçı/album adı, kapak resmi; önce/duraklat-oynat/sonra düğmeleriyle uzaktan kontrol; **ses seviyesi slider'ı** ile ana çıkış sesi anında ayarlanır (hoparlör ikonuna tıklayarak ses aç/kapa)

- **İndirmeler (yt-dlp):** İndirmeler sayfasından YouTube, Instagram, Vimeo, TikTok ve tüm yt-dlp kaynaklarından video (en iyi kalite MP4) ya da sadece ses (MP3, 192 kbps) indirilir; yt-dlp ve ffmpeg ikili dosyaları ilk işte otomatik indirilir (kullanıcı terminal kullanmaz), ilerleme yüzdesi canlı görünür; anasayfada son işleri gösteren İndirmeler kartı da var. Dosyalar `İndirilenler\personal-dashboard` klasörüne kaydedilir
- **Ses görselleştirici (CAVA tarzı):** WASAPI loopback ile sistem sesi yakalanıp FFT ile frekans bantlarına bölünür; anasayfada bar grafik olarak canlı (yumuşak interpolasyonla) çizilir. Bar sayısı (16-64), bar rengi (vurgu/ana/yeşil/sarı/kırmızı/gökkuşağı), aynalanma ve tepe göstergesi ayarlardan özelleştirilir
- **Büyük saat (Peaclock tarzı):** anasayfada dev rakamlarla duvar saati kartı; 24/12 saat biçimi, saniyeler, yanıp sönen iki nokta, tarih satırı ve rakam rengi ayarlardan seçilir
- **Menü:** gruplu, tıklayınca açılan/kapanan alt menüler; başındaki düğmeyle simge moduna sıkıştırılabilir
- **Ana içerik** geniş ekranda ortalanır, sağda boşluk kalmaz
- **Kalıcı veri:** veriler `%APPDATA%\personal-dashboard\data.json` içinde JSON olarak saklanır

## Geliştirme

```bash
cargo run
```

## Yol Haritası

- haftalık/aylık özet e-postası veya dışa aktarma (ör. CSV)
