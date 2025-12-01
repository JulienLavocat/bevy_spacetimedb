// Source: https://github.com/bevyengine/bevy/issues/8983
// This introduces message channels, on one side of which is mpsc::Sender<T>, and on another
// side is bevy's MessageReader<T>, and it automatically bridges between the two.

use bevy::prelude::*;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

#[derive(Resource, Deref, DerefMut)]
struct ChannelReceiver<T>(Mutex<Receiver<T>>);

/// Allows to register a message channel backed by a `mpsc::Receiver<T>`.
/// This is useful in multithreaded applications where you want to send messages from a different thread
pub trait AddMessageChannelAppExtensions {
    /// Allows you to create bevy messages using mpsc Sender
    fn add_message_channel<T: Message>(&mut self, receiver: Receiver<T>) -> &mut Self;
}

impl AddMessageChannelAppExtensions for App {
    fn add_message_channel<T: Message>(&mut self, receiver: Receiver<T>) -> &mut Self {
        assert!(
            !self.world().contains_resource::<ChannelReceiver<T>>(),
            "this SpacetimeDB message channel is already initialized",
        );

        self.add_message::<T>();
        self.add_systems(PreUpdate, channel_to_message::<T>);
        self.insert_resource(ChannelReceiver(Mutex::new(receiver)));
        self
    }
}

fn channel_to_message<T: 'static + Send + Sync + Message>(
    receiver: Res<ChannelReceiver<T>>,
    mut writer: MessageWriter<T>,
) {
    // this should be the only system working with the receiver,
    // thus we always expect to get this lock
    let messages = receiver.lock().expect("unable to acquire mutex lock");

    writer.write_batch(messages.try_iter());
}
