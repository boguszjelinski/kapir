use std::{fmt, time::SystemTime};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cab {
    pub id: i64,
	pub location: i32,
    pub status: CabStatus
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CabStatus {
    ASSIGNED = 0,
    FREE = 1,
    CHARGING =2, // out of order, ...
}

impl fmt::Display for CabStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_cab_status(idx: i32) -> CabStatus {
    let s: CabStatus = unsafe { ::std::mem::transmute(idx) };
    return s
}

impl Default for Cab {
    fn default() -> Cab { 
        Cab { id: -1, location: -1, status: CabStatus::CHARGING }
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
    pub received: Option<DateTime<Local>>,
    #[serde(default)]
    pub started: Option<DateTime<Local>>,
    #[serde(default)]
    pub completed: Option<DateTime<Local>>,
    #[serde(default)]
    pub at_time: Option<DateTime<Local>>,
    #[serde(default)]
    pub eta: i32,
    #[serde(default)]
    pub cab: Cab,
    #[serde(default)]
    pub cust_id: i64
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum OrderStatus {
    RECEIVED = 0,
	ASSIGNED = 1,
	ACCEPTED = 2,  
	CANCELLED= 3,
	REJECTED = 4,
	ABANDONED= 5,
	REFUSED  = 6,
	PICKEDUP = 7,
	COMPLETED= 8,
}

impl Default for OrderStatus {
    fn default() -> Self { OrderStatus::REFUSED }
}

pub fn get_order_status(idx: i32) -> OrderStatus {
    let s: OrderStatus = unsafe { ::std::mem::transmute(idx) };
    return s
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
            cab: Cab { id: -1, location: -1, status: CabStatus::CHARGING },
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
    pub started: Option<SystemTime>,
    #[serde(default)]
    pub completed: Option<SystemTime>,
    pub status: RouteStatus
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
    COMPLETED = 6
}

impl Default for RouteStatus {
    fn default() -> Self { RouteStatus::REJECTED }
}

impl fmt::Display for RouteStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


pub fn get_route_status(idx: i32) -> RouteStatus {
    let s: RouteStatus = unsafe { ::std::mem::transmute(idx) };
    return s
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Route {
	pub id: i64,
    pub status: RouteStatus,
    #[serde(default)]
    pub legs: Vec<Leg>
}

impl Default for Route {
    fn default() -> Route {
        Route { id: -1, status: RouteStatus::REJECTED, legs: vec![] } 
    }
}
