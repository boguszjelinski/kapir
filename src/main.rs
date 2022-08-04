use std::future::Future;
use derive_more::{Display, From};

use actix_web::{get,put, post, web, App, HttpServer, HttpResponse, Result, Error}; // Responder
use actix_web_httpauth::extractors::basic::BasicAuth;
use deadpool_postgres::{Client, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use tokio_postgres::NoTls;
use tokio_postgres::error::Error as PGError;
use tokio_pg_mapper::Error as PGMError;
use serde::{Deserialize, Serialize};

use log::{info,warn,debug,error,LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
mod service;
use service::{select_cab, select_order, update_cab, update_order, insert_order, init_read_stops};

mod model;
use model::{Cab, Order};

mod distance;
use distance::{init_distance};

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
    setup_logger("kapi.log".to_string());

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

    init_dist_service(&pool).await;
  
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(put_cab)
            .service(put_cab2)
            .service(get_cab)
            .service(get_order)
            .service(put_order)
            .service(put_order2)
            .service(post_order)
            .service(post_order2)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn init_dist_service(pool: &Pool) {
    match pool.get().await.map_err(MyError::PoolError) {
        Ok(c) => {
            let stops = init_read_stops(c).await;
            init_distance(&stops);
        }
        Err(err) => {
            panic!("Distance service could not start {}", err);
        }
    };
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

#[get("/orders/{id}")]
async fn get_order(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return get_object(id, db_pool, select_order).await;
}

#[put("/orders")]
async fn put_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return update_object(obj, db_pool, update_order).await;
}

#[put("/orders/")]
async fn put_order2(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return update_object(obj, db_pool, update_order).await;
}

#[post("/orders")]
async fn post_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return update_object(obj, db_pool, insert_order).await;
}

#[post("/orders/")]
async fn post_order2(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return update_object(obj, db_pool, insert_order).await;
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

fn setup_logger(file_path: String) {
    let level = log::LevelFilter::Info;
    // Build a stderr logger.
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l} - {m}\n")))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);
}
