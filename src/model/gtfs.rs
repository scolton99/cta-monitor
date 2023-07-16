use crate::util::Record;
use record_derive::Record;
use serde::Deserialize;

#[derive(Record, Deserialize)]
#[table_name = "gtfs_agency"]
pub struct Agency {
    #[primary] pub agency_name: Option<String>,
    pub agency_url: String,
    pub agency_timezone: String,
    pub agency_lang: String,
    pub agency_phone: String,
    pub agency_fare_url: String
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_route"]
pub struct Route {
    #[primary] pub route_id: Option<String>,
    pub route_short_name: Option<String>,
    pub route_long_name: Option<String>,
    pub route_type: u8,
    pub route_url: Option<String>,
    pub route_color: Option<String>,
    pub route_text_color: Option<String>
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_stop"]
pub struct Stop {
    #[primary] pub stop_id: Option<String>,
    pub stop_code: Option<String>,
    pub stop_name: String,
    pub stop_desc: Option<String>,
    pub stop_lat: String,
    pub stop_lon: String,
    pub location_type: u8,
    pub parent_station: Option<String>,
    pub wheelchair_boarding: u8
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_calendar"]
pub struct Calendar {
    #[primary] pub service_id: Option<String>,
    pub monday: u8,
    pub tuesday: u8,
    pub wednesday: u8,
    pub thursday: u8,
    pub friday: u8,
    pub saturday: u8,
    pub sunday: u8,
    pub start_date: String,
    pub end_date: String
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_calendar_date"]
pub struct CalendarDate {
    #[primary] pub service_id: Option<String>,
    #[primary] pub date: Option<String>,
    pub exception_type: u8
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_trip"]
pub struct Trip {
    pub route_id: String,
    pub service_id: String,
    #[primary] pub trip_id: Option<String>,
    pub direction_id: Option<u8>,
    pub block_id: Option<String>,
    pub shape_id: Option<String>,
    pub wheelchair_accessible: Option<u8>,
    pub schd_trip_id: Option<String>
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_frequency"]
pub struct Frequency {
    #[primary] pub trip_id: Option<String>,
    #[primary] pub start_time: Option<String>,
    pub end_time: String,
    pub headway_secs: usize
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_shape"]
pub struct Shape {
    #[primary] pub shape_id: Option<String>,
    pub shape_pt_lat: String,
    pub shape_pt_lon: String,
    #[primary] pub shape_pt_sequence: Option<usize>,
    pub shape_dist_traveled: Option<usize>
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_stop_time"]
pub struct StopTime {
    #[primary] pub trip_id: Option<String>,
    pub arrival_time: Option<String>,
    pub departure_time: Option<String>,
    pub stop_id: String,
    #[primary] pub stop_sequence: Option<usize>,
    pub stop_headsign: Option<String>,
    pub pickup_type: Option<u8>,
    pub shape_dist_traveled: Option<usize>
}

#[derive(Record, Deserialize)]
#[table_name = "gtfs_transfer"]
pub struct Transfer {
    #[primary] pub from_stop_id: Option<String>,
    #[primary] pub to_stop_id: Option<String>,
    pub transfer_type: u8
}
