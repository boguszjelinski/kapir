use derive_more::{Display, From};

use actix_web::{get, web, App, HttpServer, HttpResponse, Responder, Result, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use deadpool_postgres::{Client, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use tokio_postgres::NoTls;
use tokio_postgres::error::Error as PGError;
use tokio_pg_mapper::Error as PGMError;
use serde::{Deserialize, Serialize};

#[derive(Display, From, Debug)]
pub enum MyError {
    NotFound,
    PGError(PGError),
    PGMError(PGMError),
    PoolError(PoolError),
}
impl std::error::Error for MyError {}

#[derive(Deserialize, Serialize)]
pub struct Cab {
    pub id: i64,
	pub location: i32
}

#[get("/cabs/{id}")]
async fn getCab(id: web::Path<i64>, auth: BasicAuth, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> { // -> impl Responder

    // TODO: generic async func calling routines like 'get_cab'; this can be tough ...
    let myid = id.abs(); // TODO: how to unwrap?
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(mut c) => {
            return Ok(HttpResponse::Ok().json(get_cab(c, myid).await));
        }
        Err(err) => {
            return Ok(HttpResponse::Ok().json(format!("{}", err)));
        }
    };
}

/*
async fn getObj<T>(id: web::Path<i64>, db_pool: web::Data<Pool>, f: fn(Connection, i64) -> T) 
                -> Result<HttpResponse, Error> {
    let myid = id.abs(); // TODO: how to unwrap?
    match db_pool.get().await.map_err(MyError::PoolError) {
        Ok(mut c) => {
            return Ok(HttpResponse::Ok().json(f(c, myid).await));
        }
        Err(err) => {
            return Ok(HttpResponse::Ok().json(format!("{}", err)));
        }
    };
}
*/

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
            //.service(greet0)
            .service(getCab)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_cab(c: Client, id: i64) -> Cab {
    let qry = "SELECT location, status, name FROM cab WHERE id=$1".to_string();
    let row = c.query_one(&qry, &[&(id)]).await.unwrap();
    //let val:i32 = ;
    return Cab {
        id: id,
        location: row.get(0),
    };
} 
