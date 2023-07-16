use crate::util::{Config};
use crate::task::load_gtfs::load_gtfs;

mod task;
mod model;
mod util;

#[tokio::main]
async fn main() -> Result<(), mysql::Error> {
    let cfg: Config = confy::load("cta-monitor", None).unwrap();

    // let pool = Pool::new(cfg.mysql_connect_uri.as_ref())?;
    //
    // let mut stop = Stop {
    //     stop_id: Some(1),
    //     stop_code: 1,
    //     stop_name: "Jackson & Austin Terminal".to_string(),
    //     stop_desc: "Jackson & Austin Terminal, Northeastbound, Bus Terminal".to_string(),
    //     stop_lat: "41.87632184".to_string(),
    //     stop_lon: "-87.77410482".to_string(),
    //     location_type: 0,
    //     parent_station: None,
    //     wheelchair_boarding: 1,
    // };
    //
    // let mut db = DB::Pooled(pool.get_conn()?);
    //
    // match stop.save(&mut db) {
    //     Err(e) => println!("{}", e),
    //     Ok(_) => {}
    // };
    //
    // let all = Stop::all(&mut db).unwrap_or_else(|e| {
    //     println!("{}", e);
    //     panic!();
    // });
    //
    // println!("{}", all[0].stop_name.as_str());
    //
    // match Stop::destroy_all(&mut db) {
    //     Err(e) => println!("{}", e),
    //     Ok(_) => {}
    // }

    if let Err(e) = load_gtfs(&cfg).await {
        println!("{}", e.to_string());
    };

    Ok(())
}
