use crate::camera::MousePosition;
use crate::chunk::{Chunk, EntityLookupChunk, CHUNK_SIZE};
use crate::particle::{
    get_particle_density, get_particle_mass_density, get_particle_pressure_gradient,
    LocalMassDensity, Mass, Particle, PredictedPos, Velocity,
};
use bevy::app::{App, Plugin, Update};
use bevy::color::palettes::tailwind::{BLUE_200, GREEN_700, ORANGE_400, RED_500};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::prelude::{Gizmos, Query, Resource, Transform, With};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::cmp::max;

#[derive(Debug, Clone, Resource)]
pub struct DebugConfig {
    pub enable_vel_gizmo: bool,
    pub enable_pred_gizmo: bool,
    pub show_mouse_pos: bool,
    pub enable_local_density_gizmo: bool,
    pub highlight_neighborhood_entities: bool,
    pub show_density_grid: bool,
    pub show_derivative_gizmo: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_vel_gizmo: true,
            enable_pred_gizmo: false,
            show_mouse_pos: false,
            enable_local_density_gizmo: false,
            highlight_neighborhood_entities: false,
            show_density_grid: false,
            show_derivative_gizmo: false,
        }
    }
}

pub struct ParticleDebugPlugin;

impl Plugin for ParticleDebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugConfig::default())
            .add_plugins(EguiPlugin)
            .add_systems(Update, debug_config_ui)
            .add_systems(
                Update,
                (
                    particle_pred_gizmos.run_if(config_pred_gizmo_enabled),
                    particle_velocity_gizmos.run_if(config_vel_gizmo_enabled),
                    local_density_gizmos.run_if(config_local_density_gizmo_enabled),
                    highlight_neighborhood_entities.run_if(config_highlight_neighborhood_enabled),
                    density_grid.run_if(config_show_density_grid),
                    derivative_arrow.run_if(config_show_derivative_gizmo_enabled),
                ),
            );
    }
}

pub fn particle_pred_gizmos(mut gizmos: Gizmos, particle_q: Query<&PredictedPos, With<Particle>>) {
    for pred in particle_q.iter() {
        gizmos.circle_2d(pred.0, 0.2, BLUE_200);
    }
}

pub fn particle_velocity_gizmos(
    mut gizmos: Gizmos,
    particle_q: Query<(&Velocity, &Transform), With<Particle>>,
) {
    for (&Velocity(vel), transf) in particle_q.iter() {
        let start = Vec2::new(transf.translation.x, transf.translation.y);
        gizmos.arrow_2d(
            start,
            Vec2::new(vel.x + start.x, vel.y + start.y),
            ORANGE_400,
        );
    }
}

pub fn derivative_arrow(
    mut gizmos: Gizmos,

    at_pos: Res<MousePosition>,
    particles: Query<(&Transform, &LocalMassDensity, &Mass), With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
) {
    let derivative = get_particle_pressure_gradient(at_pos.0, &particles, &entity_lookup_chunk);
    if derivative.is_none() {
        return;
    }
    let derivative = derivative.unwrap();
    println!("Derivative {derivative:?}");
    gizmos.arrow_2d(
        at_pos.0,
        Vec2::new(at_pos.0.x + derivative.x, at_pos.0.y + derivative.y),
        GREEN_700,
    );
}

pub fn local_density_gizmos(
    mut gizmos: Gizmos,
    particles: Query<(&Transform, &Mass), With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
    mouse_pos: Res<MousePosition>,
) {
    let density = get_particle_mass_density(mouse_pos.0, &particles, &entity_lookup_chunk)
        .unwrap_or_default();
    println!("density: {:?}", density);
    let d = 1.0 - 1.0 / (density.max(0.01) * 10.0);
    let color = Color::Srgba(Srgba::rgb(d, d, d));
    gizmos.circle_2d(mouse_pos.0, 0.05, color);
    gizmos.circle_2d(mouse_pos.0, 0.01, color);
}

pub fn density_grid(
    mut gizmos: Gizmos,
    particles: Query<(&Transform, &Mass), With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
) {
    let mut x = 0.0;
    let mut y = 0.0;
    const STEP_SIZE: f32 = 1.0 / 3.0;
    while x <= CHUNK_SIZE as f32 {
        while y <= CHUNK_SIZE as f32 {
            let x_2 = x - 0.5;
            let y_2 = y - 0.5;
            let d = 1.0
                - 1.0
                    / (get_particle_mass_density(
                        Vec2::new(x_2, y_2),
                        &particles,
                        &entity_lookup_chunk,
                    )
                    .unwrap_or_default()
                        * 10.0)
                        .max(0.01);
            let color =
                Srgba::rgba_u8((255.0 * d) as u8, (200.0 * d) as u8, (255.0 * d) as u8, 100);
            //println!("XY, {x}|{y}");
            gizmos.rect_2d(Vec2::new(x_2, y_2), Vec2::new(STEP_SIZE, STEP_SIZE), color);

            y += STEP_SIZE;
        }
        y = 0.0;
        //println!("BreakLoop {}", x);
        x += STEP_SIZE;
    }
}

pub fn highlight_neighborhood_entities(
    mut gizmos: Gizmos,
    particles: Query<&Transform, With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
    mouse_pos: Res<MousePosition>,
) {
    let chunk_pos = Chunk::<Vec<Entity>>::get_chunk_pos(mouse_pos.0.x, mouse_pos.0.y);
    if chunk_pos.is_none() {
        return;
    }
    let chunk_pos = chunk_pos.unwrap();

    let entities =
        entity_lookup_chunk.get_neighborhood_entities(chunk_pos.x as usize, chunk_pos.y as usize);
    for entity in entities.iter() {
        if let Ok(transform) = particles.get(*entity) {
            gizmos.circle_2d(
                Vec2::new(transform.translation.x, transform.translation.y),
                0.05,
                RED_500,
            );
        }
    }
}

pub fn config_pred_gizmo_enabled(debug_config: Res<DebugConfig>) -> bool {
    debug_config.enable_pred_gizmo
}

pub fn config_vel_gizmo_enabled(debug_config: Res<DebugConfig>) -> bool {
    debug_config.enable_vel_gizmo
}

pub fn config_local_density_gizmo_enabled(debug_config: Res<DebugConfig>) -> bool {
    debug_config.enable_local_density_gizmo
}

pub fn config_highlight_neighborhood_enabled(debug_config: Res<DebugConfig>) -> bool {
    debug_config.highlight_neighborhood_entities
}

pub fn config_show_density_grid(debug_config: Res<DebugConfig>) -> bool {
    debug_config.show_density_grid
}

pub fn config_show_derivative_gizmo_enabled(debug_config: Res<DebugConfig>) -> bool {
    debug_config.show_derivative_gizmo
}

pub fn debug_config_ui(
    mut contexts: EguiContexts,
    mut config: ResMut<DebugConfig>,
    mouse_pos: Res<MousePosition>,
) {
    let show_mouse_pos = config.show_mouse_pos;
    egui::Window::new("Debug Config").show(contexts.ctx_mut(), |ui| {
        ui.checkbox(&mut config.enable_pred_gizmo, "Predicted Position Gizmo");
        ui.checkbox(&mut config.enable_vel_gizmo, "Velocity Gizmo");
        ui.checkbox(
            &mut config.show_mouse_pos,
            if show_mouse_pos {
                format!("Mouse Position: {:.1}|{:.1}", mouse_pos.0.x, mouse_pos.0.y)
            } else {
                "Show Mouse Position".to_string()
            },
        );
        ui.checkbox(
            &mut config.enable_local_density_gizmo,
            "Local Density Gizmo",
        );
        ui.checkbox(
            &mut config.highlight_neighborhood_entities,
            "Highlight Neighborhood Entities",
        );
        ui.checkbox(&mut config.show_density_grid, "Show Density Grid");
        ui.checkbox(&mut config.show_derivative_gizmo, "Show Derivative");
    });
}
