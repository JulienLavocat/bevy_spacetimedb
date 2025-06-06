// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

use super::body_types_type::BodyTypes;

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub struct StarSystem {
    pub id: u32,
    pub name: String,
    pub star_type: BodyTypes,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub mass: f32,
    pub radius: f32,
    pub luminosity: f32,
    pub inner_limit: f32,
    pub outer_limit: f32,
    pub frost_line: f32,
    pub hz_inner: f32,
    pub hz_outer: f32,
}

impl __sdk::InModule for StarSystem {
    type Module = super::RemoteModule;
}
