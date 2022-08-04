use std::time::SystemTime;
use chrono::prelude::Utc;

use deadpool_postgres::Client;
use crate::model::{Cab, CabStatus, Order, Stop, get_cab_status, get_order_status, OrderStatus};
use crate::distance::{STOPS, STOPS_LEN, DIST};

// SERVICE
pub async fn select_cab(c: Client, id: i64) -> Cab {
    let sql = "SELECT location, status FROM cab WHERE id=$1".to_string();
    match c.query_one(&sql, &[&id]).await {
        Ok(row) => {
            return Cab {  id: id, 
                location: row.get(0), 
                status: get_cab_status(row.get(1))
            }
        }
        Err(err) => {
            println!("{}", err);
            return Cab { ..Default::default() }
        }
    }
} 

pub async fn update_cab(c: Client, cab: Cab) -> Cab {
    let sql = "UPDATE cab SET status=$1, location=$2 WHERE id=$3".to_string(); 
    check_result(c.execute(&sql, &[&(cab.status as i32), &cab.location, &cab.id]).await);
    return cab.clone();
}

pub async fn select_order(c: Client, id: i64) -> Order {
    let sql = "SELECT from_stand, to_stand, max_wait, max_loss, distance, shared, in_pool, \
                received, started, completed, at_time, eta, status, cab_id, cust_id FROM taxi_order WHERE id=$1".to_string();
    match c.query_one(&sql, &[&id]).await {
        Ok(row) => {
            let mut o = Order {
                id: id,
                from: row.get(0),
                to: row.get(1),
                wait: row.get(2),
                loss: row.get(3),
                dist: row.get(4),
                shared: row.get(5),
                in_pool: row.get(6),
                received: row.get::<usize,Option<SystemTime>>(7),
                started: row.get::<usize,Option<SystemTime>>(8),
                completed: row.get::<usize,Option<SystemTime>>(9),
                at_time: row.get::<usize,Option<SystemTime>>(10),
                eta: row.get(11),
                status: get_order_status(row.get(12)),
                cab: Cab { id: -1, location: -1, status: CabStatus::CHARGING },
                cust_id: row.get(14)
            };
            let cab: Option<i64> = row.get(13);
            match cab {
                Some(id) => { 
                    o.cab = select_cab(c, id).await;
                }
                None => {
                    // NULL
                }
            }
            return o;
        }
        Err(err) => {
            println!("{}", err);
            return Order { ..Default::default() };
        }
    }
} 

pub async fn update_order(c: Client, order: Order) -> Order {
    let sql = "UPDATE taxi_order SET status=$1 WHERE id=$2".to_string(); 
    check_result(c.execute(&sql, &[&(order.status as i32), &order.id]).await);
    return order.clone();
}

pub async fn insert_order(c: Client, o: Order) -> Order {
    if o.from == o.to {
        println!("a joker");
        return Order{ ..Default::default() }
    }
    let date_as_string = Utc::now().to_string();
    let sql = "INSERT INTO taxi_order (from_stand, to_stand, max_loss, max_wait, shared, in_pool, eta,\
                    status, received, distance, customer_id) VALUES ($1,$2,$3,$4,$5,false,-1,$7,'$8',$9,$10) RETURNING (id)".to_string(); 
    let mut dist: i16 = 0;
    unsafe { dist = DIST[o.from as usize][o.to as usize]; }
    match c.query_one(&sql, &[
        &o.from, &o.to, &o.loss, &o.wait, &o.shared, &(OrderStatus::ASSIGNED as i32), &date_as_string, 
        &dist, &o.cust_id]).await {
        Ok(row) => {
            let mut ret: Order = o.clone();
            ret.id = row.get(0);
            return ret;
        }
        Err(err) => {
            println!("{}", err);
            return Order { ..Default::default() }
        }
    }                    
}

fn check_result(res: Result<u64, tokio_postgres::Error>) {
    match res {
        Ok(count) => {
            println!("Updated rows: {}", count);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

pub async fn init_read_stops(client: Client) -> Vec<Stop> {
    let mut ret: Vec<Stop> = Vec::new();
    for row in client.query("SELECT id, latitude, longitude, bearing FROM stop", &[]).await.unwrap() {
        ret.push(Stop {
            id: row.get(0),
            latitude: row.get(1),
            longitude: row.get(2),
            bearing: row.get(3)
        });
    }
    let slice = ret.as_slice();
    unsafe {
    STOPS = match slice.try_into() {         
        Ok(arr) => arr,         
        Err(_) => panic!("Expected a Vec of length {} but it was {}", 32, ret.len()),     
    };     
    STOPS_LEN = ret.len();
    }
    return ret;
}
