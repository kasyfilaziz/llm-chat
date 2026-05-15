use dotenvy::dotenv;
use std::env;

pub fn load_env() {
    dotenv().ok();
}

#[allow(dead_code)]
pub fn get_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{} must be set in .env", key))
}
pub fn get_var_optional(key: &str) -> Option<String> {
    env::var(key).ok()
}
