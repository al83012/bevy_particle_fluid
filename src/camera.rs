use bevy::color::palettes::tailwind::BLUE_950;
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PanCamPlugin)
            .add_systems(Startup, setup_camera);
    }
}

#[derive(Component)]
struct MyCameraMarker;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCam::default()
    ));
}