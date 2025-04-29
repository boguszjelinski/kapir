use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cab {
    pub id: i64,
    pub location: i32,
    pub status: CabStatus,
    #[serde(default)]
    pub seats: i8,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CabAssign {
    pub cust_id: i64,
    pub from: i32,
    pub to: i32,
    pub loss: i32,
    pub shared: bool,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CabStatus {
    ASSIGNED = 0,
    FREE = 1,
    CHARGING = 2, // out of order, ...
}

impl fmt::Display for CabStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_cab_status(idx: i32) -> CabStatus {
    let s: CabStatus = unsafe { ::std::mem::transmute(idx) };
    return s;
}

impl Default for Cab {
    fn default() -> Cab {
        Cab {
            id: -1,
            location: -1,
            status: CabStatus::CHARGING,
            seats: -1,
        }
    }
}

// ORDER
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Order {
    #[serde(default)]
    pub id: i64,
    pub from: i32,
    pub to: i32,
    pub wait: i32,
    pub loss: i32,
    #[serde(default)]
    pub distance: i32,
    #[serde(default)]
    pub shared: bool,
    #[serde(default)]
    pub in_pool: bool,
    #[serde(default)]
    pub status: OrderStatus,
    #[serde(default)]
    pub received: Option<NaiveDateTime>,
    #[serde(default)]
    pub started: Option<NaiveDateTime>,
    #[serde(default)]
    pub completed: Option<NaiveDateTime>,
    #[serde(default)]
    pub at_time: Option<NaiveDateTime>,
    #[serde(default)]
    pub eta: i32,
    #[serde(default)]
    pub cab: Cab,
    #[serde(default)]
    pub cust_id: i64,
    #[serde(default)]
    pub route_id: i64,
    #[serde(default)]
    pub leg_id: i64,
    //    #[serde(default)]
    //    pub route: Route,
    //    #[serde(default)]
    //    pub leg: Leg,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum OrderStatus {
    RECEIVED = 0,
    ASSIGNED = 1,
    ACCEPTED = 2,
    CANCELLED = 3,
    REJECTED = 4,
    ABANDONED = 5,
    REFUSED = 6,
    PICKEDUP = 7,
    COMPLETED = 8,
}

impl Default for OrderStatus {
    fn default() -> Self {
        OrderStatus::REFUSED
    }
}

pub fn get_order_status(idx: i32) -> OrderStatus {
    let s: OrderStatus = unsafe { ::std::mem::transmute(idx) };
    return s;
}

impl Default for Order {
    fn default() -> Order {
        Order {
            id: -1,
            from: -1,
            to: -1,
            wait: 0,
            loss: 0,
            distance: 0,
            shared: false,
            in_pool: false,
            received: None,
            started: None,
            completed: None,
            at_time: None,
            eta: 0,
            status: OrderStatus::REFUSED,
            cab: Cab {
                id: -1,
                location: -1,
                status: CabStatus::CHARGING,
                seats: -1,
            },
            route_id: -1,
            leg_id: -1,
            //    route: Route { ..Default::default() },
            //    leg: Leg { ..Default::default()},
            cust_id: -1,
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// STOP
#[derive(Clone, Deserialize, Serialize)]
pub struct Stop {
    pub id: i64,
    pub bearing: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub name: Option<String>,
}

// LEG
#[derive(Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Leg {
    pub id: i64,
    #[serde(default)]
    pub route_id: i64,
    #[serde(default)]
    pub from: i32,
    #[serde(default)]
    pub to: i32,
    #[serde(default)]
    pub place: i32,
    #[serde(default)]
    pub dist: i32,
    #[serde(default)]
    pub started: Option<NaiveDateTime>,
    #[serde(default)]
    pub completed: Option<NaiveDateTime>,
    pub status: RouteStatus,
    #[serde(default)]
    pub passengers: i32,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum RouteStatus {
    PLANNED = 0,   // proposed by Pool
    ASSIGNED = 1,  // not confirmed, initial status
    ACCEPTED = 2,  // plan accepted by customer, waiting for the cab
    REJECTED = 3,  // proposal rejected by customer(s)
    ABANDONED = 4, // cancelled after assignment but before 'PICKEDUP'
    STARTED = 5,   // status needed by legs
    COMPLETED = 6,
}

impl Default for RouteStatus {
    fn default() -> Self {
        RouteStatus::REJECTED
    }
}

impl fmt::Display for RouteStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_route_status(idx: i32) -> RouteStatus {
    let s: RouteStatus = unsafe { ::std::mem::transmute(idx) };
    return s;
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Route {
    pub id: i64,
    pub status: RouteStatus,
    #[serde(default)]
    pub legs: Vec<Leg>,
    #[serde(default)]
    pub cab: Cab,
}

impl Default for Route {
    fn default() -> Route {
        Route {
            id: -1,
            status: RouteStatus::REJECTED,
            legs: vec![],
            cab: Cab {
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RouteWithOrders {
    pub route: Route,
    pub orders: Vec<Order>,
    pub cab: Cab, // for Kaut app
}

#[derive(Clone, Deserialize, Serialize)]
pub struct StopTraffic {
    pub stop: Option<Stop>,
    pub routes: Vec<RouteWithEta>,
    pub cabs: Vec<Cab>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RouteWithEta {
    pub eta: i16,
    pub route: Route,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Stats {
    pub kpis: Vec<Stat>,
    pub orders: Vec<Stat>,
    pub cabs: Vec<Stat>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Stat {
    pub name: String,
    pub int_val: i32,
}
