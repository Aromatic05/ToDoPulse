use crate::data::{Event, TaskTime};

pub trait EventFilter {
    fn matches(&self, event: &Event) -> bool;
}

pub trait SetFilter {
    fn matches(&self, name: &str) -> bool;
}

pub struct TimeFilter {
    pub ddl: u64,
}

impl EventFilter for TimeFilter {
    fn matches(&self, event: &Event) -> bool {
        match &event.task_time {
            TaskTime::Deadline(d) => *d <= self.ddl,
            TaskTime::Duration(d) => d.end <= self.ddl,
        }
    }
}

pub struct NameFilter {
    pub name: String,
}

impl SetFilter for NameFilter {
    fn matches(&self, name: &str) -> bool {
        self.name == name
    }
}
