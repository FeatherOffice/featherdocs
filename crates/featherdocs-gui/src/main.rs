use featherdocs_gui::app::App;

fn main() -> iced::Result {
    let app = iced::daemon(App::titulo, App::update, App::view).subscription(App::subscription).theme(App::tema).antialiasing(true);
    featherdocs_gui::FONTES_DO_TEXTO.iter().fold(app, |app, f| app.font(*f)).default_font(iced::Font::with_name("Liberation Sans")).run_with(App::novo)
}
