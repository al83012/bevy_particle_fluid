use bevy::prelude::*;
use crate::map::{ChunkPosition, EntityLookupChunk, CHUNK_SIZE};

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
            .add_systems(Startup, spawn_particles);
    }
}

pub fn spawn_particles(mut commands: Commands, mut chunk: ResMut<EntityLookupChunk>) {
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let particle_bundle = ParticleBundle::at_pos(x as f32, y as f32);
            let entity = commands.spawn(particle_bundle).id();
            chunk.0.add(entity, x, y).expect("Adding Entity to Chunk failed");
        }
    }
}

