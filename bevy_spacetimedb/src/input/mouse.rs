use spacetimedb::SpacetimeType;

/// SpacetimeType equivalent to [bevy::input::mouse::MouseButton](https://docs.rs/bevy/latest/bevy/input/mouse/enum.MouseButton.html)
#[derive(SpacetimeType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}
