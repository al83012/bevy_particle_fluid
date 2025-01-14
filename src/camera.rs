use bevy::color::palettes::tailwind::BLUE_950;
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PanCamPlugin)
            .add_systems(Startup, setup_camera)
            .insert_resource(MousePosition(Vec2::ZERO))
            .add_systems(Update, camera_cursor_projection);
    }
}

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCam::default(),
        MainCamera
    ));
}

fn camera_cursor_projection(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_pos: ResMut<MousePosition>
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        //eprintln!("World coords: {}/{}", world_position.x, world_position.y);
        mouse_pos.0.x = world_position.x;
        mouse_pos.0.y = world_position.y;
    }
}

#[derive(Resource, Clone, Debug)]
pub struct MousePosition(pub Vec2);
