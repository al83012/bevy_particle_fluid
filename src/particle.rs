
use bevy::color::palettes::tailwind::{BLUE_200, BLUE_400, BLUE_50, BLUE_950, ORANGE_400};
use bevy::prelude::*;
use rand::Rng;
use crate::basic_assets::{MaterialColorDatabase, MeshShapeDatabase, ParticleAssetPlugin, SimAssetId};
use crate::chunk::{Chunk, ChunkPosition, EntityLookupChunk, CHUNK_SIZE};
use crate::debug::ParticleDebugPlugin;

#[derive(Component, Default, Clone, Debug)]
pub struct PredictedPos(pub Vec2);

#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Bundle, Clone, Default, Debug)]
pub struct ParticleBundle {
    pub particle: Particle,
    pub transform: Transform,
    pub pred_pos: PredictedPos,
    pub velocity: Velocity,
    pub chunk_position: ChunkPosition,
    pub visual: ParticleVisualBundle,
}

#[derive(Component, Default, Clone, Debug)]
pub struct Particle;


#[derive(Bundle, Clone, Default, Debug)]
pub struct ParticleVisualBundle{
    pub mesh: Mesh2d,
    pub mesh_material: MeshMaterial2d<ColorMaterial>,
}







pub struct ParticlePlugin;


impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityLookupChunk>()
            .add_plugins(ParticleAssetPlugin)
            .add_systems(Startup, spawn_particles)
            .add_systems(Update, (calc_pred_pos))
            .add_plugins(ParticleDebugPlugin);
    }
}





pub fn spawn_particles(mut commands: Commands, mut chunk: ResMut<EntityLookupChunk>, colors: Res<MaterialColorDatabase>, meshes: Res<MeshShapeDatabase>) {
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let mesh_handle = meshes.handles.get(&SimAssetId::Particle).unwrap();
            let color_handle = colors.handles.get(&SimAssetId::Particle).unwrap();

            let mut rng = rand::thread_rng();

            let vel_x : f32 = rng.gen();
            let vel_y : f32 = rng.gen();

            let particle_bundle = ParticleBundle {
                particle: Particle,
                transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                visual: ParticleVisualBundle {
                    mesh: Mesh2d(mesh_handle.clone()),
                    mesh_material: MeshMaterial2d(color_handle.clone())
                },
                pred_pos: PredictedPos(Vec2::new(x as f32 + 0.1, y as f32 + 0.1)),
                velocity: Velocity(Vec2::new(vel_x, vel_y)),
                ..Default::default()
            };
            let entity = commands.spawn(particle_bundle).id();
            chunk.push(entity, x, y).expect("Adding Entity to Chunk failed");
        }
    }
}

pub fn calc_pred_pos(mut particle_q: Query<(&mut PredictedPos, &Transform, &Velocity), With<Particle>>) {
    for (mut pred, transf, &Velocity(vel)) in particle_q.iter_mut() {
        let pos = &mut pred.0;
        pos.x = transf.translation.x + vel.x;
        pos.y = transf.translation.y + vel.y;
    }
}


pub fn get_particle_density(at_pos: Vec2, particles: &Query<&Transform, With<Particle>>, entity_lookup_chunk: &EntityLookupChunk) -> Option<f32> {
    let chunk_pos : UVec2 = Chunk::<Vec<Entity>>::get_chunk_pos(at_pos.x, at_pos.y)?;
    let chunk_x = chunk_pos.x as usize;
    let chunk_y = chunk_pos.y as usize;
    //println!("Chunk_pos: {}|{}", chunk_x, chunk_y);

    let entities = entity_lookup_chunk.get_neighborhood_entities(chunk_x, chunk_y);


    let mut density = 0.0;
    for entity in entities.iter() {
        //println!("Entity: {}", entity);
        let transf = particles.get(*entity).expect("Particle Chunk registry failed; Expected particle to exist; didn't");


        let x_diff = transf.translation.x - at_pos.x;
        let y_diff = transf.translation.y - at_pos.y;
        let distance = f32::sqrt(x_diff * x_diff + y_diff * y_diff);
        let influence = distance_density_influence(distance);
        density += influence;
        //println!("Influence: {}", influence);
    }
    Some(density)
}

// The output is a positive value as long as the distance is between 0 and 1

pub const INFLUENCE_RADIUS: f32 = 1.0; // exactly 1 tile in each direction
pub fn distance_density_influence(distance: f32) -> f32 {
    if distance < 0.0 || distance > INFLUENCE_RADIUS {
        return 0.0;
    }
    let x = INFLUENCE_RADIUS - distance;

    x * x * x

}

