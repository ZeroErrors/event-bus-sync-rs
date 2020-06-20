use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub trait Handler<E> {
    fn handle(&self, event: &mut E);
}

pub struct EventBus {
    handler_cells: HashMap<TypeId, Vec<Box<dyn Any>>>
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            handler_cells: HashMap::new()
        }
    }

    pub fn register_fn<E, F>(&mut self, handler: F)
        where E: 'static,
              F: Fn(&mut E) + 'static {
        self.register_handler(FnHandler::from(handler));
    }

    pub fn register_handler<E, F>(&mut self, handler: F)
        where E: 'static,
              F: Handler<E> + 'static {
        let b: Box<dyn Handler<E>>  = Box::new(handler); // This is required so that the Any type is consistent
        self.handler_cells.entry(TypeId::of::<E>())
            .or_insert_with(|| Vec::default())
            .push(Box::new(b));
    }

    pub fn call_event<E>(&self, event: &mut E)
        where E: 'static {
        let type_id = TypeId::of::<E>();
        if let Some(vec) = self.handler_cells.get(&type_id) {
            for handler in vec {
                let handler: &Box<dyn Handler<E>> = (*handler).downcast_ref().unwrap();
                handler.handle(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        EventBus::new()
    }
}

/// A handler implementation to make it easy to use closures
struct FnHandler<E, F: Fn(&mut E)> {
    dyn_fn: F,
    event: PhantomData<E>,
}

impl<E, F: Fn(&mut E)> Handler<E> for FnHandler<E, F> {
    fn handle(&self, event: &mut E) {
        let dyn_fn = &self.dyn_fn;
        dyn_fn(event)
    }
}

impl<F, E> From<F> for FnHandler<E, F>
    where F: Fn(&mut E) {
    fn from(dyn_fn: F) -> Self {
        FnHandler { dyn_fn, event: PhantomData::default() }
    }
}

#[cfg(test)]
mod tests {
    use crate::EventBus;

    #[derive(Debug)]
    struct SomeEvent {
        some_data: u32,
    }

    #[derive(Debug)]
    struct NonRegisteredEvent {
        some_data: u32,
    }

    #[test]
    fn counter() {
        let mut event_bus = EventBus::new();

        event_bus.register_fn(|e: &mut SomeEvent| {
            e.some_data += 2; // 0 -> 2
        });
        event_bus.register_fn(|e: &mut SomeEvent| {
            e.some_data *= 4; // 2 -> 8
        });
        event_bus.register_fn(|e: &mut SomeEvent| {
            e.some_data -= 2; // 8 -> 6
        });
        event_bus.register_fn(|e: &mut SomeEvent| {
            e.some_data /= 2; // 6 -> 3
        });

        let mut some_event = SomeEvent {
            some_data: 0,
        };
        event_bus.call_event(&mut some_event);

        assert_eq!(some_event.some_data, 3);
    }

    #[test]
    fn non_registered() {
        let event_bus = EventBus::new();

        let mut event = NonRegisteredEvent {
            some_data: 0,
        };
        event_bus.call_event(&mut event);

        assert_eq!(event.some_data, 0);
    }
}
