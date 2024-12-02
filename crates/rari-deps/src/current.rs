use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Current {
    pub latest_last_check: Option<DateTime<Utc>>,
    pub version: String,
}
