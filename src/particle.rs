use bevy::color::palettes::tailwind::{BLUE_400, BLUE_950};
use bevy::prelude::*;
use crate::chunk::{ChunkPosition, EntityLookupChunk, CHUNK_SIZE};

#[derive(Component, Default, Clone, Debug)]
pub struct Position(Vec2);

#[derive(Component, Default, Clone, Debug)]
pub struct PredictedPos(Vec2);

#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(Vec2);

#[derive(Bundle, Default, Clone)]
pub struct ParticleBundle {
    pub position: Position,
    pub pred_pos: PredictedPos,
    pub velocity: Velocity,
    pub chunk_position: ChunkPosition,
}


impl ParticleBundle {
    pub fn at_pos(x: f32, y: f32) -> Self {
        let pos = Vec2::new(x, y);
        Self {
            position: Position(pos.clone()),
            pred_pos: PredictedPos(pos),
            ..Default::default()
        }
    }
}



pub struct ParticlePlugin;


impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityLookupChunk>()
            .add_systems(Startup, spawn_particles)
            .add_systems(Update, display_particles_sys);
    }
}


pub fn display_particles_sys(mut gizmos: Gizmos, particle_q: Query<(&Position, &PredictedPos)>) {
    for (position, pred) in particle_q.iter() {
        gizmos.circle_2d(pred.0, 0.1, BLUE_950);
        gizmos.circle_2d(position.0, 0.1, BLUE_400);

    }
}
pub fn spawn_particles(mut commands: Commands, mut chunk: ResMut<EntityLookupChunk>) {
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let particle_bundle = ParticleBundle::at_pos(x as f32, y as f32);
            let entity = commands.spawn(particle_bundle).id();
            chunk.push(entity, x, y).expect("Adding Entity to Chunk failed");
        }
    }
}

