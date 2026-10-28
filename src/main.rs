mod app;
mod audio;
mod data;
mod downloads;
mod hardware;
mod media;
mod pages;

use gpui_kit::component::TitleBar;
use gpui_kit::*;

fn main() {
    let data = data::DashboardData::load();

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            cx.set_global(data);
            cx.set_global(hardware::HardwareInfo::new());
            cx.set_global(media::MediaState::default());
            cx.set_global(downloads::start());
            // ses görselleştirici: arka plan yakalama iş parçacığını başlat
            cx.set_global(audio::start());

            // kayıtlı tema tercihi uygulanır
            {
                let data = cx.global::<data::DashboardData>();
                if data.dark_mode {
                    gpui_kit::component::Theme::change(
                        gpui_kit::component::ThemeMode::Dark,
                        None,
                        cx,
                    );
                }
            }

            let mut window_options = TitleBar::window_options();
            window_options.window_bounds =
                Some(WindowBounds::centered(size(px(1120.), px(760.)), cx));

            gpui_kit::open_window(window_options, cx, |window, cx| {
                cx.new(|cx| app::DashboardView::new(window, cx))
            })
            .expect("Failed to open window");
        });
}
