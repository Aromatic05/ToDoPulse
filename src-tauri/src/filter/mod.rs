mod of_event;

pub use of_event::map_filter;

type SimpleFilter<T> = fn(&T) -> bool;
type ClosureFilter<T> = Box<dyn Fn(&T) -> bool>;

pub enum Filter<T> {
    A(SimpleFilter<T>),
    B(ClosureFilter<T>),
}
