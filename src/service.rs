use std::cmp;
use chrono::{NaiveDateTime, Local};
use mysql::*;
use mysql::prelude::*;
use log::{debug, info, warn};
use crate::model::{Cab, CabStatus, Order, Stop, Leg, Route, RouteStatus, Stats, Stat,
        get_cab_status, get_order_status, get_route_status, OrderStatus, RouteWithOrders, RouteWithEta, StopTraffic};
use crate::distance::{STOPS, DIST};
use crate::stats::{add_avg_pickup, add_avg_complete, save_status};

pub const STOP_WAIT : i32 = 1;

pub fn select_cab(user_id: i64, c: &mut PooledConn, id: i64) -> Cab {
    debug!("select_cab, user_id={}", user_id);
    let res = c.exec_map("SELECT location, status FROM cab WHERE id=?", (id,), 
            |(location, stat)| { Cab { id, location, status: get_cab_status(stat)}});
    match res {
        Ok(rows) => {
           return rows[0];
        }
        Err(_) => { return Cab { ..Default::default() } }
    }
} 

pub fn select_cabs_by_stop(user_id: i64, c: &mut PooledConn, stop_id: i32) -> Vec<Cab> {
    debug!("select_cabs_by_stop, user_id={}", user_id);
    let res = c.exec_map("SELECT id  FROM cab WHERE location=? AND status=1", (stop_id,), 
                                |(id,)| { Cab { id, location: stop_id, status: get_cab_status(1)}});
    match res {
        Ok(rows) => {
            return rows;
        }
        Err(_) => { return Vec::new() }
    }
} 

pub fn update_cab(user_id: i64, c: &mut PooledConn, cab: Cab) -> Cab {
    if user_id == cab.id {
        let res = c.exec_iter("UPDATE cab SET status=?, location=? WHERE id=?", 
                                                        (cab.status as i32, cab.location, cab.id));
        check_result(res);
    } else {
        info!("update_cab not authorised, user_id={}, cab_id={}", user_id, cab.id);
    }
    return cab.clone();
}

pub fn update_leg(user_id: i64, c: &mut PooledConn, leg: Leg) -> Leg {
    // these strange looking updates should authorize access
    if leg.status == RouteStatus::STARTED { 
        check_result(c.exec_iter("UPDATE leg l, route r SET l.status=?, l.started=? WHERE l.id=? AND r.id=l.route_id AND r.cab_id=?",
                    (leg.status as i32, Local::now().naive_local(), leg.id, user_id)));
    } else if leg.status == RouteStatus::COMPLETED { 
        debug!("update_leg COMPLETED, user_id={} leg_id={}, status={}", user_id, leg.id, leg.status);
        check_result(c.exec_iter("UPDATE leg l, route r SET l.status=?, l.completed=? WHERE l.id=? AND r.id=l.route_id AND r.cab_id=?",
                    (leg.status as i32, Local::now().naive_local(), leg.id, user_id)));
    } else { 
        debug!("update_leg with unknown status, user_id={} leg_id={}, status={}", user_id, leg.id, leg.status);
        check_result(c.exec_iter("UPDATE leg l, route r SET l.status=? WHERE l.id=? AND r.id=l.route_id AND r.cab_id=?", 
                        (leg.status as i32, leg.id, user_id)));
    }
    return leg.clone();
}

pub fn update_route(user_id: i64, c: &mut PooledConn, route: Route) -> Route {
    check_result(c.exec_iter("UPDATE route SET status=? WHERE id=? AND cab_id=?",
                    (route.status as i32, route.id, user_id)));
    return route.clone();
}

pub fn select_route_by_cab(user_id: i64, c:  &mut PooledConn, id: i64) -> Route {
    debug!("select_route_by_cab, user={}", user_id);
    return select_route_by_cab_ref(c, id);
}

pub fn select_route_by_cab_ref(c: &mut PooledConn, id: i64) -> Route {
    // TODO: cab's name
    // TODO: LIMIT 1, an error rather if there are more
    // SELECT r.id, c.name FROM route r, cab c WHERE r.cab_id=$1 AND r.status=1 and r.cab_id=c.id ORDER BY id LIMIT 1
    // !! Kab will need to show 1 & 5 separately when "assign on last leg" will be implemented in Kern
    let sql = format!("SELECT id FROM route WHERE cab_id={} AND (status=1 or status=5) ORDER BY id LIMIT 1", id);

    let res: Result<Option<i64>>  = c.query_first(sql);
    return match res {
        Ok(o) => { 
            match o {
                Some(route_id) => { 
                    select_route_ref(c, route_id) }
                None => { Route { ..Default::default()} }
            }
        }
        Err(_) => { Route { ..Default::default() } }
    };
} 

pub fn select_route_by_id(user_id: i64, c: &mut PooledConn, id: i64) -> Route {
    debug!("select_route_by_id, user={}", user_id);
    return select_route_ref(c, id);
}

pub fn select_route_ref(c: &mut PooledConn, id: i64) -> Route {
    // TODO: maybe a join and one DB call?              started, completed, 
    let res: Result<Vec<Row>> = c.exec("SELECT id, from_stand, to_stand, place, distance, started, completed, status \
                                                        FROM leg WHERE route_id=? ORDER by place", (id,));
    let mut legs: Vec<Leg> = Vec::new();
    match res {
        Ok(rows) => { 
            for r in rows {
                legs.push(
                    Leg { 
                        id: r.get(0).unwrap(), 
                        route_id: id, 
                        from:   r.get(1).unwrap(), 
                        to:     r.get(2).unwrap(), 
                        place: r.get(3).unwrap(), 
                        dist:   r.get(4).unwrap(), 
                        started: get_naivedate(&r, 5), 
                        completed: get_naivedate(&r, 6),
                        status: get_route_status(r.get(7).unwrap())
                    });
            }
        }
        Err(_) => { }
    };
    return Route { id, status: RouteStatus::ASSIGNED, legs, cab: select_cab_by_route_id(c, id) }
}

pub fn select_route_with_orders(user_id: i64, c: &mut PooledConn, id: i64) -> RouteWithOrders {
    let route: Route = select_route_by_cab_ref(c, id);
    let orders: Vec<Order> = select_orders_by_route(user_id, c, route.id);
    return RouteWithOrders {route, orders};
}

pub fn select_order(user_id: i64, c: &mut PooledConn, id: i64) -> Order {
    debug!("select_order, user_id={}", user_id);
    let orders: Vec<Order> = select_orders_by_what(c, id, "o.id=?");
    return orders[0];
} 

pub fn select_orders(user_id: i64, c: &mut PooledConn, id: i64) -> Vec<Order> {
    debug!("select_orders, user_id={}", user_id);
    return select_orders_by_what(c, id, "customer_id=? AND (o.status<3 OR o.status>6)");
} 

pub fn select_orders_by_route(user_id: i64, c: &mut PooledConn, id: i64) -> Vec<Order> {
    debug!("select_orders_by_route, user_id={}", user_id);
    return select_orders_by_what(c, id, "route_id=? AND (o.status<3 OR o.status>6)");
} 

fn get_naivedate(row: &Row, index: usize) -> Option<NaiveDateTime> {
    let val: Option<mysql::Value> = row.get(index);
    return match val {
        Some(x) => {
            if x == Value::NULL {
                None
            } else {
                row.get(index)
            }
        }
        None => None
    };
}

fn get_i64(row: &Row, index: usize) -> i64 {
    let val: Option<mysql::Value> = row.get(index);
    return match val {
        Some(x) => {
            if x == Value::NULL {
                -1
            } else {
                row.get(index).unwrap()
            }
        }
        None => -1
    };
}

pub fn select_orders_by_what(c: &mut PooledConn, id: i64, clause: &str) -> Vec<Order> {
    let sql = "SELECT from_stand, to_stand, max_wait, max_loss, distance, shared, in_pool, received, started, completed, \
        at_time, eta, o.status, cab_id, customer_id, o.id, c.location, c.status, route_id, leg_id \
        FROM taxi_order as o LEFT JOIN cab as c ON o.cab_id = c.id WHERE ".to_string() 
        + clause + " ORDER BY received desc"; // AND (o.status<3 OR o.status>6)
    let mut ret: Vec<Order> = Vec::new();
    let selected: Result<Vec<Row>> = c.exec(sql, (id,));
    match selected {
        Ok(sel) => {
            for r in sel {
                let cab_id: Option<i64> = r.get(13).unwrap();
                ret.push(Order {
                    id: r.get(15).unwrap(),
                    from: r.get(0).unwrap(),
                    to: r.get(1).unwrap(),
                    wait: r.get(2).unwrap(),
                    loss: r.get(3).unwrap(),
                    distance: r.get(4).unwrap(),
                    shared: r.get(5).unwrap(),
                    in_pool: r.get(6).unwrap(),
                    received: get_naivedate(&r, 7),
                    started: get_naivedate(&r, 8),
                    completed: get_naivedate(&r, 9),
                    at_time: get_naivedate(&r, 10),
                    eta: r.get(11).unwrap(),
                    status: get_order_status(r.get(12).unwrap()),
                    cab: match cab_id {
                        Some(cab_id) => { 
                            Cab { id: cab_id, location: r.get(16).unwrap(), 
                                    status: get_cab_status(r.get(17).unwrap()) }
                        }
                        None => { // not assigned
                            Cab { id: -1, location: -1, status: CabStatus::CHARGING }
                        }
                    },
                    cust_id: r.get(14).unwrap(),
                    route_id: get_i64(&r, 18),
                    leg_id: get_i64(&r, 19),
                });
            }
        },
        Err(error) => warn!("Problem reading row: {:?}", error),
    }
    return ret;
}

pub fn update_order(user_id: i64, c: &mut PooledConn, order: Order) -> Order {
    if order.status == OrderStatus::PICKEDUP {
        check_result(c.exec_iter("UPDATE taxi_order SET status=?, started=? WHERE id=? AND customer_id=?", 
            (order.status as i32, Local::now().naive_local(), order.id, user_id)));
        add_avg_pickup(get_elapsed_dt(order.received));
    } else if order.status == OrderStatus::COMPLETED {
        check_result(c.exec_iter("UPDATE taxi_order SET status=?, completed=? WHERE id=? AND customer_id=?",
         (order.status as i32, Local::now().naive_local(), order.id, user_id)));
        add_avg_complete(get_elapsed_dt(order.received));
    } else {
        check_result(c.exec_iter("UPDATE taxi_order SET status=? WHERE id=? AND customer_id=?", 
            (order.status as i32, order.id, user_id)));
    }
    return order.clone();
}

pub fn insert_order(user_id: i64, c: &mut PooledConn, o: Order) -> Order {
    if o.from == o.to {
        println!("a joker");
        return Order{ ..Default::default() }
    } else if o.cust_id != user_id {
        println!("a hacker");
        return Order{ ..Default::default() }
    }
    let orders = select_orders_by_what(c, o.cust_id, "customer_id=? AND (o.status<3 OR o.status = 7)");
    if orders.len() > 0 {
        println!("POST order failed for usr_id={}, orders exist", o.cust_id);
        return Order { ..Default::default() }
    }
    let dist: i32;
    unsafe { dist = DIST[o.from as usize][o.to as usize] as i32; }

    let res = c.exec_drop(
        "INSERT INTO taxi_order (from_stand, to_stand, max_loss, max_wait, shared, in_pool, eta,\
                status, received, distance, customer_id) VALUES ( \
                :from_stand, :to_stand, :max_loss, :max_wait, :shared, false, -1, :status, :received, :distance, :customer_id)",
        params! {
            "from_stand" => o.from,
            "to_stand" => o.to,
            "max_loss" => o.loss,
            "max_wait" => o.wait,
            "shared"   => o.shared,
            "status"   => OrderStatus::RECEIVED as i32,
            "received" => Local::now().naive_local(),
            "distance" => dist,
            "customer_id" => o.cust_id
        },
    )
    .and_then(|_| Ok(c.last_insert_id()));
    
    match res {
        Ok(ins_id) => {
            let mut ret: Order = o.clone();
            ret.distance = dist;
            ret.id = ins_id as i64;
            ret.received = Some(Local::now().naive_local()); // it is not exactly the same as in DB but good enough for KPIs - client will send it back on PICKUP and COMPLETE
            return ret;
        }
        Err(err) => {
            println!("{}", err);
            return Order { ..Default::default() }
        }
    }                    
}

pub fn select_traffik(user_id: i64, c: &mut PooledConn, stand_id: i64) -> StopTraffic {  
    let stop_id:i32 = stand_id as i32;
    // the inner part of the SQL finds routes that have something to do with the stop and will be visited, are not passed
    // the outer part receives all legs of these routes, not only these with that stop (we have to count ETA) 
    let leg_sql = 
        "SELECT l.id, l.from_stand, l.to_stand, l.place, l.distance, l.started, l.completed, l.status, l.route_id \
        FROM leg l WHERE l.route_id IN ( \
            SELECT route_id FROM leg WHERE (from_stand=? AND status in (1,2)) OR (to_stand=? AND status IN (1,2,5)) ) \
        AND l.status IN (1,2,5) ORDER by l.route_id, l.place".to_string(); // 1=ASSIGNED, ACCEPTED, STARTED
    let res = c.exec_map(leg_sql, (stop_id, stop_id,), 
        |(id, from, to, place, dist, started, completed, status, route_id)| {
            Leg { 
                id, 
                from, 
                to, 
                place, 
                dist, 
                started, 
                completed, 
                status: get_route_status(status),
                route_id
            }
        });
    let legs = match res {
        Ok(l) => { l }
        Err(_) => { Vec::new() }
    };
    let mut routes: Vec<RouteWithEta> = vec![];
    if legs.len() > 0 {
        // partition the data into routes
        let mut route_legs: Vec<Leg> = Vec::new();
        let mut prev_route_id: i64 = legs[0].route_id;
        for l in legs.iter() {
            if l.route_id != prev_route_id {
                if route_legs.len() > 0 {
                    routes.push(get_route_with_eta(c, prev_route_id, stop_id, route_legs));
                    route_legs = Vec::new();
                }
                prev_route_id = l.route_id;
            }
            route_legs.push(*l);
        }
        // last route
        if route_legs.len() > 0 {
            routes.push(get_route_with_eta(c, prev_route_id, stop_id, route_legs));
        }
        // the nearest cab should appear first
        routes.sort_by(|a, b| a.eta.cmp(&b.eta));
    }
    let st = unsafe { STOPS.iter().find(| &x| x.id == stand_id) };
    let stop: Option<Stop> = match st {
        Some(s) => { Some( s.clone() ) }
        None => {
            println!("Stop ID not found: {}", stop_id);
            None
        }
    };
    // finally find free cabs standing at the stop and waiting for assignments
    let cabs = select_cabs_by_stop(user_id, c, stop_id as i32);
    return StopTraffic{ stop, routes, cabs };
} 

pub fn select_cab_by_route_id(c: &mut PooledConn, id: i64) -> Cab {
    // Cab details
    let res 
        = c.exec_map("SELECT c.id, c.location, c.status FROM cab c, route r WHERE r.id=? and c.id = r.cab_id", (id,), 
        |(id, location, status)| { Cab { id, location, status: get_cab_status(status) }});
    return match res {
        Ok(row) => { row[0] }
        Err(_err) => { Cab { ..Default::default() }}
    };
}

pub fn get_route_with_eta(c: &mut PooledConn, id: i64, stop_id: i32, legs: Vec<Leg>) -> RouteWithEta {
    let cab = select_cab_by_route_id(c, id);
    let route = Route { id, status: RouteStatus::ASSIGNED, legs, cab };
    return RouteWithEta{eta: calculate_eta(stop_id as i32, &route), route};
}

pub fn select_stats(user_id: i64, c: &mut PooledConn, usr_id: i64) -> Stats {
    debug!("select_stats, user_id={}", user_id);
    if usr_id < 0 { // TODO: authorize here too 
        return Stats { kpis: vec![], orders: vec![], cabs: vec![] }
    }
    let sql = save_status();
    match c.query_iter(sql) {
        Ok(_) => {}
        Err(err) => {
            warn!("SQL failed to run, err: {}", err);
        }
    }
    return Stats { kpis: select_stats_kpis(c), 
                    orders: select_stats_orders(c), 
                    cabs: select_stats_cabs(c)
                }
}

pub fn select_stats_kpis(c: &mut PooledConn) -> Vec<Stat> {
    let res= c.exec_map("SELECT name, int_val FROM stat", (), 
                        |(name, int_val)| { Stat { name, int_val }});
    return match res {
        Ok(rows) => { rows }
        Err(_) => { Vec::new() }
    };
}

pub fn select_stats_orders(c: &mut PooledConn) -> Vec<Stat> {
    let sql = "select status, count(*) from taxi_order group by status".to_string();
    let res = c.exec_map(sql, (), |(status,count,)| {
        Stat { name: get_order_status(status).to_string(), int_val: count}
    });
    return match res {
        Ok(rows) => { rows }
        Err(_) => { Vec::new() }
    };
}

pub fn select_stats_cabs(c: &mut PooledConn) -> Vec<Stat> {
    let sql = "select status,count(*) from cab group by status".to_string();
    let res = c.exec_map(sql, (), |( status, int_val )|
                { Stat { name: get_cab_status(status).to_string(), int_val}});
    return match res {
        Ok(rows) => { rows }
        Err(_) => { Vec::new() }
    };
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
                eta += leg.dist + STOP_WAIT;
            } else {
                let minutes: i32 = (get_elapsed(leg.started)/60) as i32;
                if minutes != -1 {
                    eta += cmp::max(leg.dist - minutes, 0);
                }
                // it has taken longer than planned
                // TASK: assumption 1km = 1min, see also CabRunnable: waitMins(getDistance
            }
        } else if leg.status == RouteStatus::ASSIGNED {
            eta += leg.dist + STOP_WAIT;
        }
    }
    return eta as i16 - STOP_WAIT as i16; // minus wait time at the stand_id
}

pub fn get_elapsed(val: Option<NaiveDateTime>) -> i64 {
    match val {
        Some(x) => { 
            let now = Local::now().naive_local();
            return (now - x).num_seconds();
        }
        None => -1
    }
}

pub fn get_elapsed_dt(val: Option<NaiveDateTime>) -> i64 {
    match val {
        Some(x) => { 
            let t = Local::now().time() - x.time() ;
            t.num_seconds()
        }
        None => -1
    }
}

fn check_result(res: Result<QueryResult<'_,'_,'_, Binary>>) -> u64 {
    return match res {
        Ok(q) => {
            println!("Updated rows: {}", q.affected_rows());
            q.affected_rows()
        }
        Err(err) => {
            println!("{}", err);
            0
        }
    }
}

pub async fn init_read_stops(mut client: PooledConn) {
    let res = client.exec_map("SELECT id, latitude, longitude, bearing, name FROM stop", (), 
        |(id, latitude, longitude, bearing, name)|
            { Stop { id, latitude, longitude, bearing, name: Some(name) }});
    match res {
        Ok(rows) => { unsafe { STOPS = rows; }}
        Err(_) => {  }
    };
}
