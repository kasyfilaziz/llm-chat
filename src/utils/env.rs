use dotenvy::dotenv;
use std::env;

pub fn load_env() {
    dotenv().ok();
}

pub fn get_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{} must be set in .env", key))
}
