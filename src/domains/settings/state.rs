use dioxus::prelude::*;
use crate::domains::settings::repo::AppSettings;

pub static SETTINGS: GlobalSignal<AppSettings> = Signal::global(|| AppSettings::default());