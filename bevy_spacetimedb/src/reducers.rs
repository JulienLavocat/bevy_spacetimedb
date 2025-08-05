use crate::{AddEventChannelAppExtensions, ReducerResultEvent, StdbPlugin};
use bevy::app::App;
use spacetimedb_sdk::__codegen as spacetime_codegen;
use std::sync::mpsc::{Sender, channel};

/// Trait for making a reducer registerable into the bevy application.
pub trait RegisterableReducerEvent<
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
> where
    Self: Sized,
{
    /// The function that should the stdb callback behaviour, and send a bevy event through sender.
    fn set_stdb_callback(reducers: &C::Reducers, sender: Sender<ReducerResultEvent<Self>>);
}

impl<
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
> StdbPlugin<C, M>
{
    /// Registers a reducer event <E> for the bevy application.
    pub fn add_reducer<E: RegisterableReducerEvent<C, M> + Send + Sync + 'static>(
        mut self,
    ) -> Self {
        // This callback manages the registration of the event.
        let register_fn = move |app: &mut App, reducers: &C::Reducers| {
            let (send, recv) = channel::<ReducerResultEvent<E>>();
            app.add_event_channel(recv);
            E::set_stdb_callback(reducers, send);
        };

        // The register_fn will get called once the connection is built.
        self.reducer_registers.push(Box::new(register_fn));

        self
    }
}
