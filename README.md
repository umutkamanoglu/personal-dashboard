# Personal Dashboard

Rust + GPUI ile yazılmış kişisel dashboard uygulaması.
UI katmanı [GPUI Kit](https://github.com/longbridge/gpui-component) (`gpui-kit` crate) üzerine kurulu.

## Özellikler (v0.1)

- **Bugün sayfası:** günlük ilerleme çubuğu, günlük rutin checklist'i, günlük / haftalık / aylık alışkanlık bölümleri
- **Alışkanlık takibi:** günlük, haftalık ve aylık frekansta alışkanlıklar; seri (streak) göstergesi
- **Günlük rutinler:** her gün sıfırlanan rutin checklist'i
- **Kalıcı veri:** veriler `%APPDATA%\personal-dashboard\data.json` içinde JSON olarak saklanır

## Geliştirme

```bash
cargo run
```

## Yol Haritası

- istatistik/grafik görünümü (haftalık & aylık grafikler)
- notlar, hatırlatıcılar ve Pomodoro modülü
- tema (açık/koyu) geçişi ve özelleştirme
