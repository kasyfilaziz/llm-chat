mod app;
mod db;
mod components;
mod domains;
mod utils;

use app::App;

fn main() {
    // Initialize logger
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    // Initialize env
    utils::env::load_env();
    
    // Launch app
    dioxus::launch(App);
}
