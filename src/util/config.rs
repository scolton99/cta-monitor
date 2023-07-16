use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub mysql_connect_uri: String,
    pub cta_api_key: String,
    pub gtfs_link: String
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mysql_connect_uri: "mysql://localhost:3306/cta-monitor".parse().unwrap(),
            cta_api_key: "".into(),
            gtfs_link: "https://www.transitchicago.com/downloads/sch_data/google_transit.zip".into()
        }
    }
}
