use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub timing: Timing,
    pub audio: Audio,
    pub keymap: toml::value::Table
}

#[derive(Deserialize, Serialize)]
pub struct Timing {
    pub clock_frequency: f64,
    pub compensation_frequency: f64,
    pub window_update_frequency: f64
}

#[derive(Deserialize, Serialize)]
pub struct Audio {
    pub beeper_enabled: bool,
    pub sound_blaster_enabled: bool
}
