mod of_event;


use crate::entity::Entity;
pub use of_event::*;

type Filter<T:Entity> = fn(&T) -> bool;