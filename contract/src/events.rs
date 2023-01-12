pub mod events_cep47;
pub mod events_cep78;

pub(crate) enum Event {
    Cep47(events_cep47::CEP47Event),
    Cep47Dict(events_cep47::CEP47Event),
    Cep78,
}

pub(crate) fn record_event(event_enum: Event) {
    match event_enum {
        Event::Cep47(cep47_style) => events_cep47::record_event(&cep47_style),
        Event::Cep47Dict(cep47_style) => events_cep47::record_event_dictionary(&cep47_style),
        Event::Cep78 => (),
    }
}
