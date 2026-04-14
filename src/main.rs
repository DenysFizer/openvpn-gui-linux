mod app;
mod config;
mod error;
mod openvpn;
mod settings;
mod ui;

fn main() -> iced::Result {
    env_logger::init();
    app::run()
}
