use bevy::prelude::*;

use bevy_mod_picking::prelude::*;


// ------
// Events
// ------

#[derive(Event)]
pub struct EventHoverSquare(pub Entity);

impl From<ListenerInput<Pointer<Over>>> for EventHoverSquare {
    fn from(event: ListenerInput<Pointer<Over>>) -> Self {
        EventHoverSquare(event.target)
    }
}

#[derive(Event)]
pub struct EventClickSquare(pub Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickSquare {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickSquare(event.target)
    }
}

#[derive(Event)]
pub struct EventClickDraught(pub Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickDraught {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickDraught(event.target)
    }
}

#[derive(Event)]
pub struct EventClickCircle(pub Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickCircle {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickCircle(event.target)
    }
}

#[derive(Event)]
pub struct EventClickWatchtower(pub Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickWatchtower {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickWatchtower(event.target)
    }
}
