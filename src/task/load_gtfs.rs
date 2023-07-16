use crate::util::{Config, Record};
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use std::{fs, vec};
use std::cmp::Ordering;
use log::{debug, info, trace};
use mysql::{Pool, TxOpts};
use mysql::prelude::Queryable;
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
    info!("Started loading GTFS data from {}", &conf.gtfs_link);

    let response = reqwest::get(&conf.gtfs_link).await?;
    let response_bytes = response.bytes().await?;

    debug!("Loaded {} bytes of compressed GTFS data", response_bytes.len());

    let gtfs_dir = tempdir()?;
    zip_extract::extract(Cursor::new(response_bytes), gtfs_dir.path(), true)?;

    let mut stops: Vec<Stop> = conv(&gtfs_dir, "stops.txt")?;
    debug!("Loaded {} stops into memory from file stops.txt", stops.len());

    let mut agencies: Vec<Agency> = conv(&gtfs_dir, "agency.txt")?;
    debug!("Loaded {} agencies into memory from file agency.txt", agencies.len());

    let mut routes: Vec<Route> = conv(&gtfs_dir, "routes.txt")?;
    debug!("Loaded {} routes into memory from file routes.txt", routes.len());

    let mut frequencies: Vec<Frequency> = conv(&gtfs_dir, "frequencies.txt")?;
    debug!("Loaded {} frequencies into memory from file frequencies.txt", frequencies.len());

    let mut shapes: Vec<Shape> = conv(&gtfs_dir, "shapes.txt")?;
    debug!("Loaded {} shapes into memory from file shapes.txt", shapes.len());

    let mut trips: Vec<Trip> = conv(&gtfs_dir, "trips.txt")?;
    debug!("Loaded {} trips into memory from file trips.txt", trips.len());

    let mut calendars: Vec<Calendar> = conv(&gtfs_dir, "calendar.txt")?;
    debug!("Loaded {} calendars into memory from file calendar.txt", calendars.len());

    let mut calendar_dates: Vec<CalendarDate> = conv(&gtfs_dir, "calendar_dates.txt")?;
    debug!("Loaded {} calendar dates into memory from file calendar_dates.txt", calendar_dates.len());

    let mut stop_times: Vec<StopTime> = conv(&gtfs_dir, "stop_times.txt")?;
    debug!("Loaded {} stop times into memory from file stop_times.txt", stop_times.len());

    let mut transfers: Vec<Transfer> = conv(&gtfs_dir, "transfers.txt")?;
    debug!("Loaded {} transfers into memory from file transfers.txt", transfers.len());

    stops.sort_by(|a, b| {
        match (&a.parent_station, &b.parent_station) {
            (Some(a), Some(b)) => a.cmp(b),
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => match (&a.stop_id, &b.stop_id) {
                (Some(a), Some(b)) => a.cmp(b),
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (None, None) => Ordering::Equal
            }
        }
    });
    trace!("Sorted stops to avoid foreign key errors");

    let pool = Pool::new(conf.mysql_connect_uri.as_str())?;
    let mut conn = pool.get_conn()?;
    debug!("Connection established");

    let mut tx = conn.start_transaction(TxOpts::default())?;
    trace!("Transaction started");

    tx.query_drop("SET FOREIGN_KEY_CHECKS = 0")?;
    trace!("Disabled foreign key checks before truncations begin");

    trace!("Clearing old transfers...");
    Transfer::destroy_all(&mut tx)?;
    trace!("Transfers cleared.");

    trace!("Clearing old frequencies...");
    Frequency::destroy_all(&mut tx)?;
    trace!("Frequencies cleared.");

    trace!("Clearing old stop times...");
    StopTime::destroy_all(&mut tx)?;
    trace!("Stop times cleared.");

    trace!("Clearing old trips...");
    Trip::destroy_all(&mut tx)?;
    trace!("Trips cleared.");

    trace!("Clearing old calendar dates...");
    CalendarDate::destroy_all(&mut tx)?;
    trace!("Calendar dates cleared.");

    trace!("Clearing old calendars...");
    Calendar::destroy_all(&mut tx)?;
    trace!("Calendars cleared.");

    trace!("Clearing old routes...");
    Route::destroy_all(&mut tx)?;
    trace!("Routes cleared.");

    trace!("Clearing old stops...");
    Stop::destroy_all(&mut tx)?;
    trace!("Stops cleared.");

    trace!("Clearing old shapes...");
    Shape::destroy_all(&mut tx)?;
    trace!("Shapes cleared.");

    trace!("Clearing old agencies...");
    Agency::destroy_all(&mut tx)?;
    trace!("Agencies cleared.");

    tx.query_drop("SET FOREIGN_KEY_CHECKS = 1")?;
    trace!("Re-enabled foreign key checks post-truncations");

    debug!("Inserting agencies...");
    Agency::save_all(&mut tx, agencies.as_mut_slice())?;
    debug!("Agencies inserted.");

    debug!("Inserting shapes...");
    Shape::save_all(&mut tx, shapes.as_mut_slice())?;
    debug!("Shapes inserted.");

    debug!("Inserting stops...");
    Stop::save_all(&mut tx, stops.as_mut_slice())?;
    debug!("Stops inserted.");

    debug!("Inserting routes...");
    Route::save_all(&mut tx, routes.as_mut_slice())?;
    debug!("Routes inserted.");

    debug!("Inserting calendars...");
    Calendar::save_all(&mut tx, calendars.as_mut_slice())?;
    debug!("Calendars inserted.");

    debug!("Inserting calendar dates...");
    CalendarDate::save_all(&mut tx, calendar_dates.as_mut_slice())?;
    debug!("Calendar dates inserted.");

    debug!("Inserting trips...");
    Trip::save_all(&mut tx, trips.as_mut_slice())?;
    debug!("Trips inserted.");

    debug!("Inserting stop times...");
    StopTime::save_all(&mut tx, stop_times.as_mut_slice())?;
    debug!("Stop times inserted.");

    debug!("Inserting frequencies...");
    Frequency::save_all(&mut tx, frequencies.as_mut_slice())?;
    debug!("Frequencies inserted.");

    debug!("Inserting transfers...");
    Transfer::save_all(&mut tx, transfers.as_mut_slice())?;
    debug!("Transfers inserted.");

    tx.commit()?;

    Ok(())
}