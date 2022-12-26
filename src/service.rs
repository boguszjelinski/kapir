use std::cmp;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use deadpool_postgres::Client;
use tokio_postgres::Row;
use crate::model::{Cab, CabStatus, Order, Stop, Leg, Route, RouteStatus, Stats, Stat,
        get_cab_status, get_order_status, get_route_status, OrderStatus, RouteWithOrders, RouteWithEta, StopTraffic};
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

pub async fn select_cabs_by_stop(c: Client, stop_id: i32) -> Vec<Cab> {
    let sql = "SELECT id, name FROM cab WHERE location=$1 AND status=1".to_string(); // 1=FREE
    let mut ret: Vec<Cab> = Vec::new();
    for row in c.query(&sql, &[&stop_id]).await.unwrap() {
        ret.push(Cab { id: row.get(0), location: stop_id, status: get_cab_status(1) });
    }
    return ret;
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

pub async fn select_route_by_cab(c: Client, id: i64) -> Route {
    return select_route_by_cab_ref(&c, id).await;
}

pub async fn select_route_by_cab_ref(c: &Client, id: i64) -> Route {
    let sql = "SELECT id FROM route WHERE cab_id=$1 AND status=1 ORDER BY id LIMIT 1".to_string(); //status=1: ASSIGNED
    // TODO: cab's name
    // TODO: LIMIT 1, an error rather if there are more
    // SELECT r.id, c.name FROM route r, cab c WHERE r.cab_id=$1 AND r.status=1 and r.cab_id=c.id ORDER BY id LIMIT 1
    match c.query_one(&sql, &[&id]).await {
        Ok(row) => {
            return select_route_ref(c, row.get(0)).await;
        }
        Err(_err) => {
            //println!("{}", err);
            return Route { ..Default::default() }
        }
    }
} 

pub async fn select_route_by_id(c: Client, id: i64) -> Route {
    return select_route_ref(&c, id).await;
}

pub async fn select_route_ref(c: &Client, id: i64) -> Route {
    let mut legs: Vec<Leg> = vec![];
    // TODO: maybe a join and one DB call?
    let leg_sql = "SELECT id, from_stand, to_stand, place, distance, started, completed, status \
                            FROM leg WHERE route_id=$1 ORDER by place".to_string();
    for row in c.query(&leg_sql, &[&id]).await.unwrap() {
        legs.push(Leg { 
            id:     row.get(0), 
            route_id: id, 
            from:   row.get(1), 
            to:     row.get(2), 
            place:  row.get(3), 
            dist:   row.get(4), 
            started:row.get(5), 
            completed:row.get(6), 
            status: get_route_status(row.get(7))
        });
    }
    // Cab details
    let cab: Cab;
    let sql = "SELECT c.id, c.location, c.status FROM cab c, route r \
                        WHERE r.id=$1 and c.id = r.cab_id".to_string();
    match c.query_one(&sql, &[&id]).await {
        Ok(row) => {
            cab = Cab { id: row.get(0), location: row.get(1),
                        status: get_cab_status(row.get(2)) };
        }
        Err(_err) => {
            //println!("{}", err);
            cab = Cab { ..Default::default() }
        }
    }

    return Route {
        id: id,
        status: RouteStatus::ASSIGNED, // see WHERE above
        legs: legs,
        cab: cab
    }
}

pub async fn select_route_with_orders(c: Client, id: i64) -> RouteWithOrders {
    let route: Route = select_route_by_cab_ref(&c, id).await;
    let orders: Vec<Order> = select_orders_by_route(c, route.id).await;
    return RouteWithOrders {route, orders};
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
    return select_orders_by_what(&c, id, "customer_id=$1").await;
} 

pub async fn select_orders_by_route(c: Client, id: i64) -> Vec<Order> {
    return select_orders_by_what(&c, id, "route_id=$1").await;
} 

pub async fn select_orders_by_what(c: &Client, id: i64, clause: &str) -> Vec<Order> {
    let sql = "SELECT from_stand, to_stand, max_wait, max_loss, distance, shared, in_pool, received, started, completed, \
        at_time, eta, o.status, cab_id, customer_id, o.id, c.location, c.status, route_id, leg_id \
        FROM taxi_order as o LEFT JOIN cab as c ON o.cab_id = c.id
        WHERE ".to_string() + clause + " AND (o.status<3 OR o.status>6)";
    let mut ret: Vec<Order> = Vec::new();
    for row in c.query(&sql, &[&id]).await.unwrap() {
        // some basic info about an order
        let mut o: Order = build_order(row.get(15), &row); // TODO: these indices are ugly & errorprone
        // assigned Cab
        let cab: Option<i64> = row.get(13);
        match cab {
            Some(cab_id) => { 
                o.cab = Cab { id: cab_id, location: row.get(16), status: get_cab_status(row.get(17)) };
            }
            None => { // not assigned
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
        cust_id: row.get(14),
        route_id: row.get(18),
        leg_id: row.get(19),
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

pub async fn select_traffik(c: Client, stand_id: i64) -> StopTraffic {
    let mut legs: Vec<Leg> = vec![];
    let stop_id:i32 = stand_id as i32;
    let leg_sql = "SELECT id, from_stand, to_stand, place, distance, started, completed, status, route_id \
                            FROM leg WHERE (from_stand=$1 OR to_stand=$1) AND status=1".to_string(); // 1=ASSIGNED
    for row in c.query(&leg_sql, &[&stop_id]).await.unwrap() {
        legs.push(Leg { 
            id:     row.get(0), 
            from:   row.get(1), 
            to:     row.get(2), 
            place:  row.get(3), 
            dist:   row.get(4), 
            started:row.get(5), 
            completed:row.get(6), 
            status: get_route_status(row.get(7)),
            route_id: row.get(8)
        });
    }
   
    let mut routes: Vec<RouteWithEta> = vec![];
    
    'outer: for (i, l) in legs.iter().enumerate() {
        // find duplicates
        for (j, l2) in legs.iter().enumerate() {
            if j <= i { continue; } // checked earlier
            if l2.route_id == l.route_id { 
                continue 'outer; // ignore it
            }
        }
        let r = select_route_ref(&c, l.route_id).await;
        println!("DEBUG: l.route_id={}", l.route_id);
        routes.push(RouteWithEta{eta: calculate_eta(stop_id as i32, &r), route: r});
    }
    // the nearest cab should appear first
    routes.sort_by(|a, b| b.eta.cmp(&a.eta));
    let st = unsafe { STOPS.iter().find(| &x| x.id == stand_id) };
    let stop: Option<Stop> = match st {
        Some(s) => { Some( s.clone() ) }
        None => {
            println!("Stop ID not found: {}", stop_id);
            None
        }
    };
    // finally find free cabs standing at the stop and waiting for assignments
    let cabs = select_cabs_by_stop(c, stop_id as i32).await;
    return StopTraffic{ stop, routes, cabs };
} 

pub async fn select_stats(c: Client, user_id: i64) -> Stats {
    if user_id < 0 { // TODO: authorize here too 
        return Stats { kpis: vec![], orders: vec![], cabs: vec![] }
    }
    return Stats { kpis: select_stats_kpis(&c).await, 
                    orders: select_stats_orders(&c).await, 
                    cabs: select_stats_cabs(&c).await
                }
}

pub async fn select_stats_kpis(c: &Client) -> Vec<Stat> {
    let sql = "SELECT name, int_val FROM stat".to_string();
    let mut ret: Vec<Stat> = Vec::new();
    for row in c.query(&sql, &[]).await.unwrap() {
        ret.push(Stat { name: row.get(0), int_val: row.get(1) });
    }
    return ret;
}

pub async fn select_stats_orders(c: &Client) -> Vec<Stat> {
    let sql = "select status,count(*) from taxi_order group by status".to_string();
    let mut ret: Vec<Stat> = Vec::new();
    for row in c.query(&sql, &[]).await.unwrap() {
        let status: i32 = row.get(0);
        let count: i64 = row.get(1);
        ret.push(Stat { name: get_order_status(status).to_string(), int_val: count as i32 });
    }
    return ret;
}

pub async fn select_stats_cabs(c: &Client) -> Vec<Stat> {
    let sql = "select status,count(*) from cab group by status".to_string();
    let mut ret: Vec<Stat> = Vec::new();
    for row in c.query(&sql, &[]).await.unwrap() {
        let status: i32 = row.get(0);
        let count: i64 = row.get(1);
        ret.push(Stat { name: get_cab_status(status).to_string(), int_val: count as i32 });
    }
    return ret;
}

pub fn calculate_eta(stand_id: i32, route: &Route) -> i16 {
    if route.id == -1 {
        return -1;
    }
    let mut eta = 0;
    let route_cpy = route.clone();
    for leg in route_cpy.legs {
        if leg.from == stand_id {
            break; // if standId happens to be toStand in the last leg and
            // this break never occurs - that is just OK
        }
        // there are two situations - active (currently executed) leg and legs waiting for pick-up
        //let distance = unsafe { DIST[leg.From][leg.To] }; 
        if leg.status == RouteStatus::STARTED {
            if leg.started == None { // some error
            eta += leg.dist;
            } else {
            let minutes: i32 = (get_elapsed(leg.started)/60) as i32;
            if minutes != -1 {
                eta += cmp::max(leg.dist - minutes, 0);
            }
            // it has taken longer than planned
            // TASK: assumption 1km = 1min, see also CabRunnable: waitMins(getDistance
            }
        } else if leg.status == RouteStatus::ASSIGNED {
            eta += leg.dist;
        } else {
            println!("Leg {} is in not STARTED, nor ASSIGNED {}", leg.id, route.id);
        }
    }
    return eta as i16;
}

pub fn get_elapsed(val: Option<SystemTime>) -> i64 {
    match val {
        Some(x) => { 
            match x.elapsed() {
                Ok(elapsed) => elapsed.as_secs() as i64,
                Err(_) => -1
            }
        }
        None => -1
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
