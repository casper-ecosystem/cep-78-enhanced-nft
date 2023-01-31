pub mod events_cep47;

pub(crate) enum Event {
    Cep47(events_cep47::CEP47Event),
}

pub(crate) fn record_event(event_enum: Event) {
    match event_enum {
        Event::Cep47(event) => events_cep47::record_event_dictionary(&event),
    }
}
