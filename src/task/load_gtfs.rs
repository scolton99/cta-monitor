use crate::util::{Config, DB, Record};
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use std::{fs, vec};
use mysql::Pool;
use serde::de::DeserializeOwned;
use crate::model::gtfs::{Stop, Agency, Route, Frequency, Shape, Calendar, Trip, CalendarDate, StopTime, Transfer};

fn load_csv_data(buf: PathBuf) -> std::io::Result<String> {
    fs::read_to_string(buf)
}

fn sub(dir: &TempDir, file: &str) -> PathBuf {
    let mut path = dir.path().to_path_buf();
    path.push(file);
    path
}

fn conv<T: DeserializeOwned>(dir: &TempDir, file: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let csv_data = load_csv_data(sub(dir, file))?;

    let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
    let mut ret_vec: Vec<T> = vec![];

    for res in reader.deserialize() {
        ret_vec.push(res?);
    }

    Ok(ret_vec)
}

pub async fn load_gtfs(conf: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(&conf.gtfs_link).await?;
    let response_bytes = response.bytes().await?;

    let gtfs_dir = tempdir()?;
    zip_extract::extract(Cursor::new(response_bytes), gtfs_dir.path(), true)?;

    let mut stops: Vec<Stop> = conv(&gtfs_dir, "stops.txt")?;
    let mut agencies: Vec<Agency> = conv(&gtfs_dir, "agency.txt")?;
    let mut routes: Vec<Route> = conv(&gtfs_dir, "routes.txt")?;
    let mut frequencies: Vec<Frequency> = conv(&gtfs_dir, "frequencies.txt")?;
    let mut shapes: Vec<Shape> = conv(&gtfs_dir, "shapes.txt")?;
    let mut trips: Vec<Trip> = conv(&gtfs_dir, "trips.txt")?;
    let mut calendars: Vec<Calendar> = conv(&gtfs_dir, "calendar.txt")?;
    let mut calendar_dates: Vec<CalendarDate> = conv(&gtfs_dir, "calendar_dates.txt")?;
    let mut stop_times: Vec<StopTime> = conv(&gtfs_dir, "stop_times.txt")?;
    let mut transfers: Vec<Transfer> = conv(&gtfs_dir, "transfers.txt")?;

    println!("{}", stops[0].stop_name);
    println!("{}", agencies[0].agency_name.as_ref().unwrap());
    println!("{}", routes[0].route_long_name.as_ref().unwrap());
    println!("{}", frequencies[0].trip_id.as_ref().unwrap());
    println!("{}", shapes[0].shape_id.as_ref().unwrap());
    println!("{}", trips[0].trip_id.as_ref().unwrap());
    println!("{}", calendars[0].service_id.as_ref().unwrap());
    println!("{}", calendar_dates[0].service_id.as_ref().unwrap());
    println!("{}", stop_times[0].stop_id.as_str());
    println!("{}", transfers[0].from_stop_id.as_ref().unwrap());

    let pool = Pool::new(conf.mysql_connect_uri.as_str())?;
    let mut db = DB::Pooled(pool.get_conn()?);

    let search_trip_id = stop_times[0].trip_id.as_ref().unwrap();
    let search_stop_id = &stop_times[0].stop_id;

    let found_trip = trips.iter().position(|it| { it.trip_id.as_ref().unwrap_or(&"".to_owned()).eq(search_trip_id) })
        .expect(format!("Missing trip {}!", search_trip_id).as_str());

    let found_stop = stops.iter().position(|it| { it.stop_id.as_ref().unwrap_or(&"".to_owned()).eq(search_stop_id) })
            .expect(format!("Missing stop {}!", search_stop_id).as_str());

    let search_parent_stop_id = stops[found_stop].parent_station.as_ref().unwrap();

    let found_parent_stop = stops.iter().position(|it| { it.stop_id.as_ref().unwrap_or(&"".to_owned()).eq(search_parent_stop_id) })
        .expect(format!("Missing stop {}!", search_parent_stop_id).as_str());

    let search_transfer_from_stop_id = transfers[0].from_stop_id.as_ref().unwrap();
    let search_transfer_to_stop_id = transfers[0].to_stop_id.as_ref().unwrap();

    let found_transfer_from_stop = stops.iter().position(|it| { it.stop_id.as_ref().unwrap_or(&"".to_owned()).eq(search_transfer_from_stop_id) })
        .expect(format!("Missing stop {}!", search_transfer_from_stop_id).as_str());

    let found_transfer_to_stop = stops.iter().position(|it| { it.stop_id.as_ref().unwrap_or(&"".to_owned()).eq(search_transfer_to_stop_id) })
        .expect(format!("Missing stop {}!", search_transfer_to_stop_id).as_str());

    let search_from_parent_id = stops[found_transfer_from_stop].parent_station.as_ref().unwrap();
    let search_to_parent_id = stops[found_transfer_to_stop].parent_station.as_ref().unwrap();

    let found_from_parent = stops.iter().position(|it| { it.stop_id.as_ref().unwrap_or(&"".to_owned()).eq(search_from_parent_id) })
        .expect(format!("Missing stop {}!", search_from_parent_id).as_str());

    let found_to_parent = stops.iter().position(|it| { it.stop_id.as_ref().unwrap_or(&"".to_owned()).eq(search_to_parent_id) })
        .expect(format!("Missing stop {}!", search_to_parent_id).as_str());

    stops[found_from_parent].save(&mut db)?;
    stops[found_to_parent].save(&mut db)?;

    stops[found_transfer_from_stop].save(&mut db)?;
    stops[found_transfer_to_stop].save(&mut db)?;

    stops[found_parent_stop].save(&mut db)?;

    trips[found_trip].save(&mut db)?;
    stops[found_stop].save(&mut db)?;

    stops[0].save(&mut db)?;
    agencies[0].save(&mut db)?;
    routes[0].save(&mut db)?;

    frequencies[0].end_time = "05:31:00".to_owned();
    frequencies[0].save(&mut db)?;

    shapes[0].save(&mut db)?;
    trips[0].save(&mut db)?;
    calendars[0].save(&mut db)?;
    calendar_dates[0].save(&mut db)?;
    stop_times[0].save(&mut db)?;
    transfers[0].save(&mut db)?;

    Ok(())
}