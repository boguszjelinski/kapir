use derive_more::{Display, From};

use actix_web::{get,put, web, App, HttpServer, HttpResponse, Result, Error}; // Responder
use actix_web_httpauth::extractors::basic::BasicAuth;
use deadpool_postgres::{Client, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use tokio_postgres::NoTls;
use tokio_postgres::error::Error as PGError;
use tokio_pg_mapper::Error as PGMError;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Display, From, Debug)]
pub enum MyError {
    NotFound,
    PGError(PGError),
    PGMError(PGMError),
    PoolError(PoolError),
}
impl std::error::Error for MyError {}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
   
    let mut pg_config = tokio_postgres::Config::new();
    pg_config.host("localhost");
    pg_config.user("kabina");
    pg_config.password("kaboot");
    pg_config.dbname("kabina");
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    let pool = Pool::new(mgr, 16);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(put_cab)
            .service(put_cab2)
            .service(get_cab)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// CONTROLLERS
#[get("/cabs/{id}")]
async fn get_cab(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> { // -> impl Responder
    println!("Hello, {}!", auth.user_id());
    return get_object(id, db_pool, select_cab).await;
}

#[put("/cabs")]
async fn put_cab(obj: web::Json<Cab>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return update_object(obj, db_pool, update_cab).await;
}

#[put("/cabs/")]
async fn put_cab2(obj: web::Json<Cab>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return update_object(obj, db_pool, update_cab).await;
}

async fn get_object<Fut, T>(id: web::Path<i64>, db_pool: web::Data<Pool>, f: impl FnOnce(Client, i64) -> Fut) 
            -> Result<HttpResponse, Error>
where
    Fut: Future<Output = T>,
    T: Serialize 
{
    let myid = id.abs(); // TODO: how to unwrap?
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(c) => {
            let obj: T = f(c, myid).await as T;
            return Ok(HttpResponse::Ok().json(obj));
        }
        Err(err) => {
            return Ok(HttpResponse::Ok().json(format!("{}", err)));
        }
    };
}

async fn update_object<Fut, T>(obj: web::Json<T>, db_pool: web::Data<Pool>, f: impl FnOnce(Client, T) -> Fut) 
            -> Result<HttpResponse, Error>
where
    Fut: Future<Output = T>,
    T: Serialize
{
    let o: T = obj.into_inner();
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(c) => {
            let obj: T = f(c, o).await as T;
            return Ok(HttpResponse::Ok().json(obj));
        }
        Err(err) => {
            return Ok(HttpResponse::Ok().json(format!("{}", err)));
        }
    };
}

// SERVICE
async fn select_cab(c: Client, id: i64) -> Cab {
    let sql = "SELECT location, status, name FROM cab WHERE id=$1".to_string();
    match c.query_one(&sql, &[&(id)]).await {
        Ok(row) => {
            return Cab { id: id, location: row.get(0), status: get_cab_status(row.get(1)) };
        }
        Err(err) => {
            println!("{}", err);
            return Cab { id: -1, location: -1, status: CabStatus::CHARGING }
        }
    }
} 

async fn update_cab(c: Client, cab: Cab) -> Cab {
    let sql = "UPDATE cab SET status=$1, location=$2 WHERE id=$3".to_string(); 
    match c.execute(&sql, &[&(cab.status as i32), &cab.location, &cab.id]).await {
        Ok(count) => {
            println!("Updated rows: {}", count);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
    return cab.clone();
}

// MODEL
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Cab {
    pub id: i64,
	pub location: i32,
    pub status: CabStatus
}

#[repr(i8)]
#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum CabStatus {
    ASSIGNED = 0,
    FREE = 1,
    CHARGING =2, // out of order, ...
}

pub fn get_cab_status(idx: i8) -> CabStatus {
    let s: CabStatus = unsafe { ::std::mem::transmute(idx) };
    return s
}