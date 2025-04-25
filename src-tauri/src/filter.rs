use crate::storage::{Event, TaskTime};

pub trait EventFilter {
    fn matches(&self, event: &Event) -> bool;
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

impl EventFilter for NameFilter {
    fn matches(&self, event: &Event) -> bool {
        event.title.contains(&self.name) || event.content.contains(&self.name)
    }
}