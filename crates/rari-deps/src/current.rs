use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Current {
    pub latest_last_check: Option<DateTime<Utc>>,
    pub current_version: Option<Version>,
}
