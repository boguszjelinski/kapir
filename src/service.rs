use std::time::SystemTime;
use chrono::{DateTime, Local};
use deadpool_postgres::Client;
use tokio_postgres::Row;
use crate::model::{Cab, CabStatus, Order, Stop, Leg, Route, RouteStatus, get_cab_status, get_order_status, get_route_status, OrderStatus};
use crate::distance::{STOPS, DIST};

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

pub async fn update_leg(c: Client, leg: Leg) -> Leg {
    if leg.status == RouteStatus::STARTED { 
        let sql = "UPDATE leg SET status=$1, started=$2 WHERE id=$3".to_string();
        check_result(c.execute(&sql, &[&(leg.status as i32), &(SystemTime::now()), &leg.id]).await);
    } else if leg.status == RouteStatus::COMPLETED { 
        let sql = "UPDATE leg SET status=$1, completed=$2 WHERE id=$3".to_string();
        check_result(c.execute(&sql, &[&(leg.status as i32), &(SystemTime::now()), &leg.id]).await);
    } else { 
        let sql = "UPDATE leg SET status=$1 WHERE id=$3".to_string();
        check_result(c.execute(&sql, &[&(leg.status as i32), &leg.id]).await);
    }
    return leg.clone();
}

pub async fn update_route(c: Client, route: Route) -> Route {
    let sql = "UPDATE route SET status=$1 WHERE id=$2".to_string(); 
    check_result(c.execute(&sql, &[&(route.status as i32), &route.id]).await);
    return route.clone();
}

pub async fn select_route(c: Client, id: i64) -> Route {
    let sql = "SELECT id FROM route WHERE cab_id=$1 AND status=1 ORDER BY id LIMIT 1".to_string(); //status=1: ASSIGNED
    match c.query_one(&sql, &[&id]).await {
        Ok(row) => {
            let route_id = row.get(0);
            let mut legs: Vec<Leg> = vec![];
            // TODO: maybe a join and one DB call?
            let leg_sql = "SELECT id, from_stand, to_stand, place, distance, started, completed, status \
                                 FROM leg WHERE route_id=$1".to_string();
            for row in c.query(&leg_sql, &[&route_id]).await.unwrap() {
                legs.push(Leg { 
                    id:     row.get(0), 
                    route_id: route_id, 
                    from:   row.get(1), 
                    to:     row.get(2), 
                    place:  row.get(3), 
                    dist:   row.get(4), 
                    started:row.get(5), 
                    completed:row.get(6), 
                    status: get_route_status(row.get(7))
                });
            }
            return Route {
                id: route_id,
                status: RouteStatus::ASSIGNED, // see WHERE above
                legs: legs
            }
        }
        Err(_err) => {
            //println!("{}", err);
            return Route { ..Default::default() }
        }
    }
} 

pub async fn select_order(c: Client, id: i64) -> Order {
    let sql = "SELECT from_stand, to_stand, max_wait, max_loss, distance, shared, in_pool, \
                received, started, completed, at_time, eta, status, cab_id, customer_id FROM taxi_order WHERE id=$1".to_string();
    match c.query_one(&sql, &[&id]).await {
        Ok(row) => {
            let mut o = build_order(id, &row);
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

pub async fn select_orders(c: Client, id: i64) -> Vec<Order> {
    let sql = "SELECT from_stand, to_stand, max_wait, max_loss, distance, shared, in_pool,\
        received, started, completed, at_time, eta, o.status, cab_id, customer_id, o.id, c.location, c.status \
        FROM taxi_order as o LEFT JOIN cab as c ON o.cab_id = c.id
        WHERE customer_id=$1 AND o.status!=8 AND o.status!=3".to_string();
    let mut ret: Vec<Order> = Vec::new();
    for row in c.query(&sql, &[&id]).await.unwrap() {
        let mut o: Order = build_order(row.get(15), &row);
        let cab: Option<i64> = row.get(13);
        match cab {
            Some(id) => { 
                o.cab = Cab { id, location: row.get(16), status: get_cab_status(row.get(17)) };
            }
            None => {
                // NULL
            }
        }
        ret.push(o);
    }
    return ret;
} 

fn build_order(id: i64, row: &Row) -> Order {
    return Order {
        id,
        from: row.get(0),
        to: row.get(1),
        wait: row.get(2),
        loss: row.get(3),
        distance: row.get(4),
        shared: row.get(5),
        in_pool: row.get(6),
        received: systime_to_dttime(row.get::<usize,Option<SystemTime>>(7)),
        started: systime_to_dttime(row.get::<usize,Option<SystemTime>>(8)),
        completed: systime_to_dttime(row.get::<usize,Option<SystemTime>>(9)),
        at_time: systime_to_dttime(row.get::<usize,Option<SystemTime>>(10)),
        eta: row.get(11),
        status: get_order_status(row.get(12)),
        cab: Cab { id: -1, location: -1, status: CabStatus::CHARGING },
        cust_id: row.get(14)
    };
}

fn systime_to_dttime(time: Option<SystemTime>) -> Option<DateTime<Local>> {
    match time {
        Some(t) => Some(t.clone().into()),
        None => None
    }
}

pub async fn update_order(c: Client, order: Order) -> Order {
    if order.status == OrderStatus::PICKEDUP {
        let sql = "UPDATE taxi_order SET status=$1, started=$2 WHERE id=$3".to_string(); 
        check_result(c.execute(&sql, &[&(order.status as i32), &(SystemTime::now()), &order.id]).await);
    } else if order.status == OrderStatus::COMPLETED {
        let sql = "UPDATE taxi_order SET status=$1, completed=$2 WHERE id=$3".to_string(); 
        check_result(c.execute(&sql, &[&(order.status as i32), &(SystemTime::now()), &order.id]).await);
    } else {
        let sql = "UPDATE taxi_order SET status=$1 WHERE id=$2".to_string(); 
        check_result(c.execute(&sql, &[&(order.status as i32), &order.id]).await);
    }


    return order.clone();
}

pub async fn insert_order(c: Client, o: Order) -> Order {
    if o.from == o.to {
        println!("a joker");
        return Order{ ..Default::default() }
    }
    let sql = "INSERT INTO taxi_order (from_stand, to_stand, max_loss, max_wait, shared, in_pool, eta,\
                    status, received, distance, customer_id) VALUES ($1,$2,$3,$4,$5,false,-1,$6,$7,$8,$9) RETURNING (id)".to_string(); 
    let dist: i32;
    unsafe { dist = DIST[o.from as usize][o.to as usize] as i32; }
    match c.query_one(&sql, &[
        &o.from, &o.to, &o.loss, &o.wait, &o.shared, &(OrderStatus::RECEIVED as i32), &(SystemTime::now()), 
        &dist, &o.cust_id]).await {
        Ok(row) => {
            let mut ret: Order = o.clone();
            ret.distance = dist;
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

pub async fn init_read_stops(client: Client) {
  unsafe { 
    for row in client.query("SELECT id, latitude, longitude, bearing, name FROM stop", &[]).await.unwrap() {
        STOPS.push(Stop {
            id: row.get(0),
            latitude: row.get(1),
            longitude: row.get(2),
            bearing: row.get(3),
            name: Some(row.get(4)),
        });
    }
  }
}
