mod chunk;
mod particle;
mod camera;

use bevy::diagnostic::{DiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use crate::camera::CameraPlugin;
use crate::particle::ParticlePlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LogDiagnosticsPlugin::default()))
        .add_plugins(CameraPlugin)
        .add_plugins(ParticlePlugin)
        .run();
}
