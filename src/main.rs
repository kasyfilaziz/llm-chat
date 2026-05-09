mod app;
mod db;
mod components;
mod domains;
mod utils;

use app::App;

fn main() {
    // Initialize env
    utils::env::load_env();
    
    // Launch app
    dioxus::launch(App);
}
