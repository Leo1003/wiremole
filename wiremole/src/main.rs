#![forbid(unsafe_code)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use tokio::runtime::Runtime;
use rocket_sync_db_pools::diesel::MysqlConnection;

type AnyResult<T> = Result<T, anyhow::Error>;

mod api;
mod db;

#[database("mysql")]
#[derive(Debug)]
struct DbConn(MysqlConnection);

fn main() -> AnyResult<()> {
    let runtime = Runtime::new()?;

    runtime.block_on(async move { launch_web_server().await })?;

    Ok(())
}

async fn launch_web_server() -> Result<(), rocket::Error> {
    rocket::build()
        .attach(DbConn::fairing())
        .mount("/api", api::routes())
        .launch()
        .await
}
