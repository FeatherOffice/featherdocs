use featherdocs_gui::app::App;

fn main() -> iced::Result {
    iced::daemon(App::titulo, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::tema)
        .antialiasing(true)
        .font(include_bytes!("../../../src/assets/fontes/LiberationSans-Regular.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationSans-Bold.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationSans-Italic.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationSans-BoldItalic.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationSerif-Regular.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationSerif-Bold.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationSerif-Italic.ttf").as_slice())
        .font(
            include_bytes!("../../../src/assets/fontes/LiberationSerif-BoldItalic.ttf").as_slice(),
        )
        .font(include_bytes!("../../../src/assets/fontes/LiberationMono-Regular.ttf").as_slice())
        .font(include_bytes!("../../../src/assets/fontes/LiberationMono-Bold.ttf").as_slice())
        .default_font(iced::Font::with_name("Liberation Sans"))
        .run_with(App::novo)
}
