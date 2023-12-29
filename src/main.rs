use std::env;
use std::collections::HashMap;
use derive_more::{Display, From};
use actix_web::{get,put, post, web, App, HttpServer, HttpResponse, Result, Error}; // Responder
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_cors::Cors;
use mysql::*;
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
use service::{select_cab, select_order, update_cab, update_order, insert_order, select_orders,
            init_read_stops, update_leg, update_route, select_route_by_id, select_route_by_cab, select_traffik, select_stats};
mod model;
use model::{Cab, Order, Leg, Route};
mod distance;
use distance::init_distance;
use crate::{distance::STOPS, service::select_route_with_orders};
mod stats;

#[derive(Display, From, Debug)]
pub enum MyError {
    NotFound,
    PoolError
}
impl std::error::Error for MyError {}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    
    let mut dbhost: String;
    let mut bind_host: String;
    let bind_port: u16;

    let settings = config::Config::builder()
        .add_source(config::File::with_name("kapir.toml"))
        .build()
        .unwrap();
    let cfg = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    dbhost = cfg["dbhost"].clone();
    bind_host = cfg["myhost"].clone();
    bind_port = cfg["myport"].clone().parse::<u16>().unwrap();

    // possible to overwrite config file
    let args: Vec<String> = env::args().collect();
    if args.len() >1  { 
        dbhost = args[1].to_string(); 
    } 
    if args.len() >2  { 
        bind_host = args[2].to_string(); 
    }

    setup_logger("kapi.log".to_string());

    //let pool: Pool = mysql_async::Pool::new("mysql://kabina:kaboot@localhost:3306/kabina");
    let pool = Pool::new(dbhost).unwrap();
        
    init_dist_service(&pool).await;

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .service(put_cab) // curl -H "Content-type: application/json" -u cab2:cab2 -X PUT -d '{ "Id":2, "Location":123, "Status":"FREE"}' http://localhost:8080/cabs
            .service(put_cab2) // {"Id":0,"Location":0,"Status":"FREE","Name":""}
            .service(get_cab) // curl -u cab1:cab1 http://localhost:8080/cabs/1916
            .service(get_order) // curl -u cab2:cab2 http://localhost:8080/orders/51150
            .service(get_order2) // curl -u cab2:cab2 http://localhost:8080/orders
            .service(get_order3) // curl -u cust1:cust1 http://localhost:8080/orders/
            .service(put_order) // curl -H "Content-type: application/json" -X PUT -u cust1:cust1 -d '{ "Id":775791, "Status":"ASSIGNED", "From":0,"To":0,"Wait":0,"Loss":0}' http://localhost:8080/orders
            .service(put_order2)
            .service(post_order) //curl -H "Content-type: application/json" -H "Accept: application/json"  -X POST -u "cust28:cust28" -d '{"From":4001, "To":4002, "Wait":10, "Loss":90, "Shared": true}' http://localhost:8080/orders
            .service(post_order2) 
            .service(put_leg) // curl -H "Content-type: application/json" -H "Accept: application/json"  -X PUT -u cab1:cab1 -d '{ "Id":17081, "Status":"STARTED"}' http://localhost:8080/legs
            .service(put_leg2)
            .service(put_route) // curl -H "Content-type: application/json" -H "Accept: application/json"  -X PUT -u cab1:cab1 -d '{ "Id":9724, "Status":"ASSIGNED"}' http://localhost:8080/routes
            .service(put_route2)
            .service(get_route) // curl -u cab2:cab2 http://localhost:8080/routes
            .service(get_route2)
            .service(get_route_by_id)
            .service(get_route_with_orders) //http://localhost:8080/routeswithorders
            .service(get_route_with_orders2)
            .service(get_stops) // curl -u cab2:cab2 http://localhost:8080/stops
            .service(get_stops2)
            .service(get_traffic)
            .service(get_stats)
    })
    .bind((bind_host, bind_port))?
    .run()
    .await
}

async fn init_dist_service(pool: &Pool) {
    init_read_stops(pool.get_conn().unwrap()).await;
    init_distance();
}

// CONTROLLERS, most duplicated to respond to a slash at the end too
#[get("/cabs/{id}")]
async fn get_cab(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> { // -> impl Responder
    let myid: i64 = id.abs(); // TODO: how to unwrap?
    info!("GET cab cab_id={} usr_id={}", myid, auth.user_id());
    return get_object(get_auth_id(auth.user_id()), myid, db_pool, select_cab);
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

#[get("/routes/{id}")]
async fn get_route_by_id(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let myid: i64 = id.abs(); // TODO: how to unwrap?
    info!("GET route route_id={} usr_id={}", myid, auth.user_id());
    return get_object(get_auth_id(auth.user_id()), myid, db_pool, select_route_by_id);
}

#[get("/routes")] // id will come from auth
async fn get_route(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_route(auth, db_pool).await;
}
#[get("/routes/")] // id will come from auth
async fn get_route2(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_route(auth, db_pool).await;
}
#[get("/routeswithorders")] // just to keep compatibility with Java
async fn get_route_with_orders(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_route_with_orders(auth, db_pool).await;
}
#[get("/routeswithorders/")] // just to keep compatibility with Java
async fn get_route_with_orders2(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_route_with_orders(auth, db_pool).await;
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
    return get_object(get_auth_id(auth.user_id()), myid, db_pool, select_order);
}

#[get("/orders")]
async fn get_order2(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_orders(auth, db_pool).await;
}

#[get("/orders/")]
async fn get_order3(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    return just_get_orders(auth, db_pool).await;
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
    return Ok(HttpResponse::Ok().json(unsafe { STOPS.clone()}));
}
#[get("/stops/")]
async fn get_stops2() -> Result<HttpResponse, Error> {
    return Ok(HttpResponse::Ok().json(unsafe { STOPS.clone()}));
}

#[get("/stops/{id}/traffic")]
async fn get_traffic(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> { // -> impl Responder
    let myid: i64 = id.abs(); // TODO: how to unwrap?
    info!("GET traffik for stop={} usr_id={}", myid, auth.user_id());
    return get_object(get_auth_id(auth.user_id()), myid, db_pool, select_traffik);
}

#[get("/stats")]
async fn get_stats(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> { // -> impl Responder
    info!("GET stats for usr_id={}", auth.user_id());
    let user_id:i64 = get_auth_id(auth.user_id());
    return get_object(user_id, user_id, db_pool, select_stats);
}

async fn just_put_cab(obj: web::Json<Cab>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Cab = obj.into_inner();
    // authorization
    if auth.user_id() == format!("cab{}", o.id) {
        info!("PUT cab cab_id={} status={} location={} usr_id={}", o.id, o.status, o.location, auth.user_id());
        return update_object(get_auth_id(auth.user_id()), o, db_pool, update_cab);
    }
    info!("PUT cab FORBIDDEN cab_id={} status={} location={} usr_id={}", o.id, o.status, o.location, auth.user_id());
    return Ok(HttpResponse::Forbidden().json("Not owner"));
}

async fn just_put_leg(obj: web::Json<Leg>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Leg = obj.into_inner();
    // authorization continues in service
    if auth.user_id().starts_with("cab") {
        info!("PUT leg leg_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
        return update_object(get_auth_id(auth.user_id()), o, db_pool, update_leg);
    }
    info!("PUT leg FORBIDDEN leg_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
    return Ok(HttpResponse::Forbidden().json("Only a cab is allowed to update a leg"));
}

async fn just_get_route(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    info!("GET route usr_id={}", auth.user_id());
    let user_id:i64 = get_auth_id(auth.user_id());
    return get_object(user_id, user_id, db_pool, select_route_by_cab); // get_object2
}
async fn just_get_route_with_orders(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    info!("GET route with orders usr_id={}", auth.user_id());
    let user_id:i64 = get_auth_id(auth.user_id());
    return get_object(user_id, user_id, db_pool, select_route_with_orders);
}

async fn just_get_orders(auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    info!("GET orders usr_id={}", auth.user_id());
    let user_id:i64 = get_auth_id(auth.user_id());
    return get_object(user_id, user_id, db_pool, select_orders); // get_object2
}

async fn just_put_route(obj: web::Json<Route>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Route = obj.into_inner();
    // authorization continues in service
    if auth.user_id().starts_with("cab") {
        info!("PUT route route_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
        return update_object(get_auth_id(auth.user_id()), o, db_pool, update_route);
    }
    info!("PUT route FORBIDDEN route_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
    return Ok(HttpResponse::Forbidden().json("Only a cab is allowed to update a route"));
}

async fn just_put_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let o: Order = obj.into_inner();
    info!("PUT order order_id={} status={} usr_id={}", o.id, o.status, auth.user_id());
    return update_object(get_auth_id(auth.user_id()), o, db_pool, update_order);
}

async fn just_post_order(obj: web::Json<Order>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let mut o: Order = obj.into_inner();
    info!("POST order from={} to={} usr_id={}", o.from, o.to, auth.user_id());
    let user_id:i64 = get_auth_id(auth.user_id());
    o.cust_id = user_id; // authorisation ;)
    return update_object(user_id, o, db_pool, insert_order);
}

fn get_object<T>(user_id: i64, object_id: i64, db_pool: web::Data<Pool>, f: impl FnOnce(i64, &mut PooledConn, i64) -> T) 
            -> Result<HttpResponse, Error>
            where T: Serialize {
    match db_pool.get_conn() {
        Ok(mut c) => {
            let obj: T = f(user_id, &mut c, object_id) as T;
            return Ok(HttpResponse::Ok().json(obj));
        }
        Err(err) => { 
            return Ok(HttpResponse::Ok().insert_header(("Access-Control-Allow-Origin","*")).json(format!("{}", err))); 
        }
    };
}

fn update_object<T>(user_id: i64, o: T, db_pool: web::Data<Pool>, f: impl FnOnce(i64, &mut PooledConn, T) -> T) 
            -> Result<HttpResponse, Error>
            where T: Serialize { 
    match db_pool.get_conn() {
        Ok(mut c) => {
            let obj: T = f(user_id, &mut c, o) as T;
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
