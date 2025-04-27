use crate::data::Event;

pub trait EventFilter {
    fn matches(&self, event: &Event) -> bool;
}

pub trait SetFilter {
    fn matches(&self, name: &str) -> bool;
}

pub struct TimeFilter {
    pub ddl: u64,
}



pub struct NameFilter {
    pub name: String,
}

impl SetFilter for NameFilter {
    fn matches(&self, name: &str) -> bool {
        self.name == name
    }
}
