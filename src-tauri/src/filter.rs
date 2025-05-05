mod of_event;


pub use of_event::*;

type Filter<T> = fn(&T) -> bool;