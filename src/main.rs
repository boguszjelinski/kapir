use std::future::Future;
use derive_more::{Display, From};
use actix_web::{get,put, post, web, App, HttpServer, HttpResponse, Result, Error}; // Responder
use actix_web_httpauth::extractors::basic::BasicAuth;
use deadpool_postgres::{Client, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use tokio_postgres::NoTls;
use tokio_postgres::error::Error as PGError;
use tokio_pg_mapper::Error as PGMError;
use serde::Serialize;
use log::{info,LevelFilter};
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
use service::{select_cab, select_order, update_cab, update_order, insert_order, 
            init_read_stops, select_stops, update_leg, update_route, select_route};
mod model;
use model::{Cab, Order, Leg, Route};
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
            .service(put_cab) // curl -H "Content-type: application/json" -H "Accept: application/json"  -X PUT -u cab1:cab1 -d '{ "id":2, "location": 123, "status":"FREE", "name":"A2"}' http://localhost:8080/cabs
            .service(put_cab2) // {"Id":0,"Location":0,"Status":"FREE","Name":""}
            .service(get_cab) // curl -u cab1:cab1 http://localhost:8080/cabs/1916
            .service(get_order) // curl -u cab2:cab2 http://localhost:8080/orders/51150
            .service(put_order) // curl -H "Content-type: application/json" -H "Accept: application/json"  -X PUT -u cab1:cab1 -d '{ "id":51150, "status":"ASSIGNED"}' http://localhost:8080/orders
            .service(put_order2)
            .service(post_order) //curl -H "Content-type: application/json" -H "Accept: application/json"  -X POST -u "cust28:cust28" -d '{"fromStand":4082, "toStand":4083, "maxWait":10, "maxLoss":90, "shared": true}' http://localhost:8080/orders
            .service(post_order2) // {"Id":-1,"fromStand":1363,"toStand":1362,"Eta":0,"InPool":true,"Cab":{"Id":0,"Location":0,"Status":"","Name":""},"Status":"RECEIVED","MaxWait":15,"MaxLoss":50,"Distance":0}
            .service(put_leg) // curl -H "Content-type: application/json" -H "Accept: application/json"  -X PUT -u cab1:cab1 -d '{ "id":17081, "status":"STARTED"}' http://localhost:8080/legs
            .service(put_leg2)
            .service(put_route) // curl -H "Content-type: application/json" -H "Accept: application/json"  -X PUT -u cab1:cab1 -d '{ "id":9724, "status":"ASSIGNED"}' http://localhost:8080/routes
            .service(put_route2)
            .service(get_route) // curl -u cab2:cab2 http://localhost:8080/routes
            .service(get_route2)
            .service(get_stops) // curl -u cab2:cab2 http://localhost:8080/stops
            .service(get_stops2)
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

// CONTROLLERS, most duplicated to respond to a slash at the end too
#[get("/cabs/{id}")]
async fn get_cab(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> { // -> impl Responder
    let myid: i64 = id.abs(); // TODO: how to unwrap?
    info!("GET cab cab_id={} usr_id={}", myid, auth.user_id());
    return get_object(myid, db_pool, select_cab).await;
}

#[put("/cabs")]
async fn put_cab(obj: web::Json<Cab>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_cab(obj, auth, db_pool).await;
}
#[put("/cabs/")]
async fn put_cab2(obj: web::Json<Cab>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_cab(obj, auth, db_pool).await;
}

#[put("/legs")]
async fn put_leg(obj: web::Json<Leg>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_leg(obj, auth, db_pool).await;
}
#[put("/legs/")]
async fn put_leg2(obj: web::Json<Leg>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_leg(obj, auth, db_pool).await;
}

#[get("/routes")] // id will come from auth
async fn get_route(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_route(auth, db_pool).await;
}
#[get("/routes/")] // id will come from auth
async fn get_route2(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_route(auth, db_pool).await;
}

#[put("/routes")]
async fn put_route(obj: web::Json<Route>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_route(obj, auth, db_pool).await;
}
#[put("/routes/")]
async fn put_route2(obj: web::Json<Route>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_route(obj, auth, db_pool).await;
}

#[get("/orders/{id}")]
async fn get_order(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let myid: i64 = id.abs(); // TODO: how to unwrap?
    info!("GET order order_id={} usr_id={}", myid, auth.user_id());
    return get_object(myid, db_pool, select_order).await;
}

#[put("/orders")]
async fn put_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_order(obj, auth, db_pool).await;
}
#[put("/orders/")]
async fn put_order2(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_put_order(obj, auth, db_pool).await;
}

#[post("/orders")]
async fn post_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_post_order(obj, auth, db_pool).await;
}
#[post("/orders/")]
async fn post_order2(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_post_order(obj, auth, db_pool).await;
}

#[get("/stops")]
async fn get_stops() -> Result<HttpResponse, Error> {
    return Ok(HttpResponse::Ok().json(select_stops().await));
}
#[get("/stops/")]
async fn get_stops2() -> Result<HttpResponse, Error> {
    return Ok(HttpResponse::Ok().json(select_stops().await));
}

async fn just_put_cab(obj: web::Json<Cab>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Cab = obj.into_inner();
    info!("PUT cab cab_id={} status={} location={} usr_id={}", o.id, o.status, o.location, auth.user_id());
    return update_object(o, db_pool, update_cab).await;
}

async fn just_put_leg(obj: web::Json<Leg>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Leg = obj.into_inner();
    info!("PUT leg leg_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
    return update_object(o, db_pool, update_leg).await;
}

async fn just_get_route(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    info!("GET route usr_id={}", auth.user_id());
    return get_object2(get_auth_id(auth.user_id()), db_pool, select_route).await;
}

async fn just_put_route(obj: web::Json<Route>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Route = obj.into_inner();
    info!("PUT route route_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
    return update_object(o, db_pool, update_route).await;
}

async fn just_put_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Order = obj.into_inner();
    info!("PUT order order_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
    return update_object(o, db_pool, update_order).await;
}

async fn just_post_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let mut o: Order = obj.into_inner();
    info!("POST order from={} to={} usr_id={}", o.from, o.to, auth.user_id());
    o.cust_id = get_auth_id(auth.user_id());
    return update_object(o, db_pool, insert_order).await;
}

async fn get_object<Fut, T>(myid: i64, db_pool: web::Data<Pool>, f: impl FnOnce(Client, i64) -> Fut) 
            -> Result<HttpResponse, Error>
where Fut: Future<Output = T>, T: Serialize {
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(c) => {
            let obj: T = f(c, myid).await as T;
            return Ok(HttpResponse::Ok().json(obj));
        }
        Err(err) => { return Ok(HttpResponse::Ok().json(format!("{}", err))); }
    };
}

async fn get_object2<Fut, T>(myid: i64, db_pool: web::Data<Pool>, f: impl FnOnce(Client, i64) -> Fut) 
            -> Result<HttpResponse, Error>
where Fut: Future<Output = T>, T: Serialize {
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(c) => {
            let obj: T = f(c, myid).await as T;
            return Ok(HttpResponse::Ok().json(obj));
        }
        Err(err) => { return Ok(HttpResponse::Ok().json(format!("{}", err))); }
    };
}

async fn update_object<Fut, T>(o: T, db_pool: web::Data<Pool>, f: impl FnOnce(Client, T) -> Fut) 
            -> Result<HttpResponse, Error>
where Fut: Future<Output = T>, T: Serialize { 
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(c) => {
            let obj: T = f(c, o).await as T;
            return Ok(HttpResponse::Ok().json(obj));
        }
        Err(err) => { return Ok(HttpResponse::Ok().json(format!("{}", err))); }
    };
}

fn get_auth_id(id: &str) -> i64 {
    if id.len() < 4 { // cab0
        return -1;
    }
    if id.starts_with("cab") || id.starts_with("adm") {
       return id[3..].parse().unwrap();
    }
    if id.starts_with("cust") {
        return id[4..].parse().unwrap();
    }
    return -1;
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
                .build(LevelFilter::Info),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);
}

