use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Cab {
    pub id: i64,
	pub location: i32,
    pub status: CabStatus
}

#[repr(i8)]
#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum CabStatus {
    ASSIGNED = 0,
    FREE = 1,
    CHARGING =2, // out of order, ...
}

pub fn get_cab_status(idx: i8) -> CabStatus {
    let s: CabStatus = unsafe { ::std::mem::transmute(idx) };
    return s
}

impl Default for Cab {
    fn default() -> Cab { 
        Cab { id: -1, location: -1, status: CabStatus::CHARGING }
    }
}
// ORDER
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Order {
    pub id: i64,
	pub from: i32,
    pub to: i32,
	pub wait: i32,
	pub loss: i32,
	pub dist: i32,
    pub shared: bool,
    pub in_pool: bool,
    pub status: OrderStatus,
    pub received: Option<SystemTime>,
    pub started: Option<SystemTime>,
    pub completed: Option<SystemTime>,
    pub at_time: Option<SystemTime>,
    pub eta: i32,
    pub cab: Cab,
    pub cust_id: i32
}

#[repr(i8)]
#[derive(Copy, Clone, Deserialize, Serialize)]
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

pub fn get_order_status(idx: i8) -> OrderStatus {
    let s: OrderStatus = unsafe { ::std::mem::transmute(idx) };
    return s
}

impl Default for Order {
    fn default() -> Order {
        Order {
            id: -1,
                from: 0,
                to: 0,
                wait: 0,
                loss: 0,
                dist: 0,
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

// STOP

#[derive(Copy, Clone)]
pub struct Stop {
    pub id: i64,
    pub bearing: i32,
	pub latitude: f64,
    pub longitude: f64
}