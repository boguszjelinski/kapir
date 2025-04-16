use self::Stat::*;
use log::info;
use std::fmt;
use std::slice::Iter;

pub static mut STATS: [i64; AvgOrderCompleteTime as usize + 1] =
    [0; AvgOrderCompleteTime as usize + 1];
const INT: Vec<i64> = vec![];
pub static mut AVG_ELEMENTS: [Vec<i64>; AvgOrderCompleteTime as usize + 1] =
    [INT; AvgOrderCompleteTime as usize + 1];

#[derive(Debug, Copy, Clone)]
pub enum Stat {
    AvgOrderPickupTime,
    AvgOrderCompleteTime,
}

impl Stat {
    pub fn iterator() -> Iter<'static, Stat> {
        static RET: [Stat; 2] = [AvgOrderPickupTime, AvgOrderCompleteTime];
        RET.iter()
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub fn update_val(key: Stat, value: i64) {
    unsafe {
        STATS[key as usize] = value;
    }
}

pub fn add_avg_element(key: Stat, time: i64) {
    unsafe {
        AVG_ELEMENTS[key as usize].push(time);
    }
}

pub fn count_average(key: Stat) -> i64 {
    unsafe {
        let list: Vec<i64> = AVG_ELEMENTS[key as usize].to_vec();
        let length = list.len();
        if length == 0 {
            return 0;
        }
        let mut suma: i64 = 0;
        for i in list {
            suma += i;
        }

        return (suma / length as i64) as i64;
    }
}

pub fn save_status() -> String {
    let mut sql: String = String::from("");
    update_val(
        Stat::AvgOrderPickupTime,
        count_average(Stat::AvgOrderPickupTime),
    );
    update_val(
        Stat::AvgOrderCompleteTime,
        count_average(Stat::AvgOrderCompleteTime),
    );
    unsafe {
        for s in Stat::iterator() {
            sql += &format!(
                "UPDATE stat SET int_val={} WHERE UPPER(name)=UPPER('{}');",
                STATS[*s as usize],
                s.to_string()
            );
        }
    }
    return sql;
}

pub fn add_avg_pickup(value: i64) {
    if value == -1 {
        info!("Warn: add_avg_pickup called with -1");
    } else {
        add_avg_element(Stat::AvgOrderPickupTime, value);
    }
}

pub fn add_avg_complete(value: i64) {
    if value == -1 {
        info!("Warn: add_avg_complete called with -1");
    } else {
        add_avg_element(Stat::AvgOrderCompleteTime, value);
    }
}
