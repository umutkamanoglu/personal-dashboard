# Personal Dashboard

Rust + GPUI ile yazılmış kişisel dashboard uygulaması.
UI katmanı [GPUI Kit](https://github.com/longbridge/gpui-component) (`gpui-kit` crate) üzerine kurulu.

## Özellikler

- **Anasayfa:** saat, günün programı ve genel durum kartları ile kart grid düzeni; tüm kartlar tıklanabilir. **Kart koleksiyonu:** Ayarlar'dan kart ekle/çıkar, sırala, genişliğini seç (1/2/3 sütun — 3 = tam satır); granüler CPU/RAM/GPU/disk/sıcaklık/bilgisayar/ağ kartları da koleksiyonda; ayarlarda kartların canlı önizlemesi var, tıklayınca eklenir
- **Saatler:** günde hangi saatte ne var? haftalık zaman çizelgesi (timeline) görünümü, canlı "şimdi" çizgisi; fare tekerleğiyle zoom (0.5×–4×)
- **Alışkanlık takibi:** günlük, haftalık ve aylık frekansta alışkanlıklar; seri (streak) göstergesi
- **Günlük rutinler:** her gün sıfırlanan rutin checklist'i
- **İstatistik:** son 7 gün tamamlama sütun grafiği ve alışkanlık başına dönem geçmişi
- **Tema:** sağ üstteki düğmeyle açık/koyu tema geçişi (tercih kaydedilir)
- **Pomodoro:** 15/25/50 dk odak + 5 dk otomatik mola, saniyelik geri sayım, ani geçiş ve tur sayısı
- **Notlar & hatırlatıcılar:** serbest notlar ve haftanın günlerine atanan hatırlatıcılar; bugün olanlar anasayfada zil ikonuyla listelenir
- **Sistem bilgisi:** CPU (model/frekans/çekirdek başına kullanım/sıcaklık), RAM, GPU (NVML üzerinden NVIDIA; kullanım/VRAM/Güç/fan), disk dolulukları, tüm sıcaklık sensörleri, **ağ (anlık indirme/yükleme hızları + oturum toplamı)** ve genel bilgisayar bilgileri — anasayfada özet kartı da var
- **Medya kontrolü:** Windows SMTC üzerinden herhangi bir uygulamanın (Spotify, tarayıcı, Groove, VLC…) çaldığı müzik anasayfadaki kartta görünür: çalan uygulama, parça/sanatçı/album adı, kapak resmi; önce/duraklat-oynat/sonra düğmeleriyle uzaktan kontrol
- **Ses görselleştirici (CAVA tarzı):** WASAPI loopback ile sistem sesi yakalanıp FFT ile frekans bantlarına bölünür; anasayfada bar grafik olarak canlı çizilir. Bar sayısı (16-64), bar rengi (vurgu/ana/yeşil/sarı/kırmızı/gökkuşağı), aynalanma ve tepe göstergesi ayarlardan özelleştirilir
- **Menü:** gruplu, tıklayınca açılan/kapanan alt menüler; başındaki düğmeyle simge moduna sıkıştırılabilir
- **Ana içerik** geniş ekranda ortalanır, sağda boşluk kalmaz
- **Kalıcı veri:** veriler `%APPDATA%\personal-dashboard\data.json` içinde JSON olarak saklanır

## Geliştirme

```bash
cargo run
```

## Yol Haritası

- haftalık/aylık özet e-postası veya dışa aktarma (ör. CSV)
