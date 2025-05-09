use crate::model::Stop;

pub const MAXSTOPSNUMB: usize = 5200;
pub const CAB_SPEED: i16 = 30; // km/h

pub static mut STOPS: Vec<Stop> = vec![];
pub static mut DIST: [[i16; MAXSTOPSNUMB]; MAXSTOPSNUMB] = [[0; MAXSTOPSNUMB]; MAXSTOPSNUMB];
const M_PI: f64 = 3.14159265358979323846264338327950288;
const M_PI_180: f64 = M_PI / 180.0;
const REV_M_PI_180: f64 = 180.0 / M_PI;

fn deg2rad(deg: f64) -> f64 {
    return deg * M_PI_180;
}
fn rad2deg(rad: f64) -> f64 {
    return rad * REV_M_PI_180;
}

// https://dzone.com/articles/distance-calculation-using-3
fn dist(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let theta = lon1 - lon2;
    let mut dist = deg2rad(lat1).sin() * deg2rad(lat2).sin()
        + deg2rad(lat1).cos() * deg2rad(lat2).cos() * deg2rad(theta).cos();
    dist = dist.acos();
    dist = rad2deg(dist);
    dist = dist * 60.0 * 1.1515;
    dist = dist * 1.609344;
    return dist;
}

pub fn init_distance() {
    unsafe {
        for i in 0..STOPS.len() {
            DIST[i][i] = 0;
            for j in i + 1..STOPS.len() {
                let mut d = dist(
                    STOPS[i].latitude,
                    STOPS[i].longitude,
                    STOPS[j].latitude,
                    STOPS[j].longitude,
                ) * (60.0 / CAB_SPEED as f64);
                if d as i16 == 0 {
                    d = 1.0;
                } // a transfer takes at least one minute.
                DIST[STOPS[i].id as usize][STOPS[j].id as usize] = d as i16; // TASK: we might need a better precision - meters/seconds
                DIST[STOPS[j].id as usize][STOPS[i].id as usize] =
                    DIST[STOPS[i].id as usize][STOPS[j].id as usize];
            }
        }
    }
}
