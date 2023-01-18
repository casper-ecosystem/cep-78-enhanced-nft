pub mod events_cep47;
pub mod events_cep78;

use crate::TokenIdentifier;

pub(crate) enum Event<'token_id> {
    Cep47(events_cep47::CEP47Event),
    Cep78(&'token_id TokenIdentifier, events_cep78::CEP78Event),
}

pub(crate) fn record_event(event_enum: Event) {
    match event_enum {
        Event::Cep47(event) => events_cep47::record_event_dictionary(&event),
        Event::Cep78(token_identifier, event) => {
            events_cep78::record_event(token_identifier.clone(), event)
        }
    }
}
