use spacetimedb::SpacetimeType;

/// SpacetimeType equivalent to [bevy::input::gamepad::GamepadButton](https://docs.rs/bevy/latest/bevy/input/gamepad/enum.GamepadButton.html)
#[derive(SpacetimeType, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,

    C,
    Z,

    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,

    LeftThumb,
    RightThumb,

    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,

    Other(u8),
}

/// SpacetimeType equivalent to [bevy::input::gamepad::GamepadAxis](https://docs.rs/bevy/latest/bevy/input/gamepad/enum.GamepadAxis.html)
#[derive(SpacetimeType, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ,
    Other(u8),
}
