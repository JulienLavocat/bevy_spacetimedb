use crate::{AddMessageChannelAppExtensions, ReducerResultMessage, StdbPlugin};
use bevy::app::App;
use spacetimedb_sdk::__codegen as spacetime_codegen;
use std::sync::mpsc::{Sender, channel};

/// Trait for making a reducer registerable into the bevy application.
pub trait RegisterableReducerMessage<
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
> where
    Self: Sized,
{
    /// The function that should define the stdb callback behaviour, and send a bevy message through sender.
    fn set_stdb_callback(reducers: &C::Reducers, sender: Sender<ReducerResultMessage<Self>>);
}

impl<
    C: spacetime_codegen::DbConnection<Module = M> + spacetimedb_sdk::DbContext,
    M: spacetime_codegen::SpacetimeModule<DbConnection = C>,
> StdbPlugin<C, M>
{
    /// Registers a reducer message <E> for the bevy application.
    pub fn add_reducer<E: RegisterableReducerMessage<C, M> + Send + Sync + 'static>(
        mut self,
    ) -> Self {
        // This callback manages the registration of the message.
        let register_fn = move |app: &mut App, reducers: &C::Reducers| {
            let (send, recv) = channel::<ReducerResultMessage<E>>();
            app.add_message_channel(recv);
            E::set_stdb_callback(reducers, send);
        };

        // The register_fn will get called once the connection is built.
        self.reducer_registers.lock().unwrap().push(Box::new(register_fn));

        self
    }
}
