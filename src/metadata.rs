use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time;

#[derive(Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub name: String,
    pub image: Option<PathBuf>,
    pub playtime: time::Duration,
    pub last_played: Option<time::SystemTime>,
    pub last_session_duration: Option<time::Duration>,
}
