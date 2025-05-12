mod desktop;
mod stamp;

use std::sync::mpsc;
use std::time::Duration;
use std::str::FromStr;
use cron::Schedule;

fn get_schedule(times: Vec<&str>) -> Vec<Schedule> {
    let mut schedules = Vec::new();
    for time in times {
        if let Ok(schedule) = Schedule::from_str(time) {
            schedules.push(schedule);
        } else {
            log::error!("Invalid cron expression: {}", time);
        }
    }
    schedules
}