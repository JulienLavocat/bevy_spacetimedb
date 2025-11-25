// Source: https://github.com/bevyengine/bevy/issues/8983
// This introduces event channels, on one side of which is mpsc::Sender<T>, and on another
// side is bevy's EventReader<T>, and it automatically bridges between the two.

use bevy::prelude::*;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

#[derive(Resource, Deref, DerefMut)]
struct ChannelReceiver<T>(Mutex<Receiver<T>>);

/// Allows to register an event channel backed by a `mpsc::Receiver<T>`.
/// This is useful in multithreaded applications where you want to send events from a different thread
pub trait AddEventChannelAppExtensions {
    /// Allows you to create bevy events using mpsc Sender
    fn add_event_channel<T: Message>(&mut self, receiver: Receiver<T>) -> &mut Self;
}

impl AddEventChannelAppExtensions for App {
    fn add_event_channel<T: Message>(&mut self, receiver: Receiver<T>) -> &mut Self {
        assert!(
            !self.world().contains_resource::<ChannelReceiver<T>>(),
            "this SpacetimeDB event channel is already initialized",
        );

        self.add_message::<T>();
        self.add_systems(PreUpdate, channel_to_event::<T>);
        self.insert_resource(ChannelReceiver(Mutex::new(receiver)));
        self
    }
}

fn channel_to_event<T: 'static + Send + Sync + Message>(
    receiver: Res<ChannelReceiver<T>>,
    mut writer: MessageWriter<T>,
) {
    // this should be the only system working with the receiver,
    // thus we always expect to get this lock
    let events = receiver.lock().expect("unable to acquire mutex lock");

    writer.write_batch(events.try_iter());
}
