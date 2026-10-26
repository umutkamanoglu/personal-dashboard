# Personal Dashboard

Rust + GPUI ile yazılmış kişisel dashboard uygulaması.
UI katmanı [GPUI Kit](https://github.com/longbridge/gpui-component) (`gpui-kit` crate) üzerine kurulu.

## Özellikler

- **Anasayfa:** saat, günün programı ve genel durum kartları ile kart grid düzeni
- **Saatler:** günde hangi saatte ne var? haftalık zaman çizelgesi (timeline) görünümü, canlı "şimdi" çizgisi
- **Alışkanlık takibi:** günlük, haftalık ve aylık frekansta alışkanlıklar; seri (streak) göstergesi
- **Günlük rutinler:** her gün sıfırlanan rutin checklist'i
- **İstatistik:** son 7 gün tamamlama sütun grafiği ve alışkanlık başına dönem geçmişi
- **Tema:** sağ üstteki düğmeyle açık/koyu tema geçişi (tercih kaydedilir)
- **Kalıcı veri:** veriler `%APPDATA%\personal-dashboard\data.json` içinde JSON olarak saklanır

## Geliştirme

```bash
cargo run
```

## Yol Haritası

- notlar, hatırlatıcılar ve Pomodoro modülü
