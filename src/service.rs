use std::time::SystemTime;
use deadpool_postgres::Client;
use crate::model::{Cab, CabStatus, Order, Stop, Leg, Route, RouteStatus, get_cab_status, get_order_status, get_route_status, OrderStatus};
use crate::distance::{STOPS, STOPS_LEN, DIST};

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
    let sql = "UPDATE leg SET status=$1 WHERE id=$2".to_string(); 
    check_result(c.execute(&sql, &[&(leg.status as i32), &leg.id]).await);
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
        Err(err) => {
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
    let sql = "INSERT INTO taxi_order (from_stand, to_stand, max_loss, max_wait, shared, in_pool, eta,\
                    status, received, distance, customer_id) VALUES ($1,$2,$3,$4,$5,false,-1,$6,$7,$8,$9) RETURNING (id)".to_string(); 
    let dist: i32;
    unsafe { dist = DIST[o.from as usize][o.to as usize] as i32; }
    match c.query_one(&sql, &[
        &o.from, &o.to, &o.loss, &o.wait, &o.shared, &(OrderStatus::RECEIVED as i32), &(SystemTime::now()), 
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
    unsafe {
        for i in 0 .. ret.len() {
            STOPS[i] = ret[i];
        }
        STOPS_LEN = ret.len();
    }
    return ret;
}

pub async fn select_stops() -> Vec<Stop> {
    let mut ret: Vec<Stop> = Vec::new();
    unsafe {
        for i in 0 .. STOPS_LEN {
            ret.push(STOPS[i]);
        }
    }
    return ret;
}
