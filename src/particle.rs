use crate::basic_assets::{
    MaterialColorDatabase, MeshShapeDatabase, ParticleAssetPlugin, SimAssetId,
};
use crate::camera::MousePosition;
use crate::chunk::{Chunk, ChunkPosition, EntityLookupChunk, CHUNK_SIZE};
use crate::debug::ParticleDebugPlugin;
use bevy::ecs::entity;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Default, Clone, Debug)]
pub struct PredictedPos(pub Vec2);

#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default, Clone, Debug)]
pub struct Acceleration(pub Vec2);

#[derive(Component, Default, Clone, Debug)]
pub struct Mass(pub f32);

#[derive(Component, Default, Clone, Debug)]
pub struct LocalMassDensity(pub f32);

#[derive(Default, Clone, Debug, Resource)]
pub struct SimParameters {
    pub pressure_mult: f32,
    pub gravity: f32,
}

#[derive(Bundle, Clone, Default, Debug)]
pub struct ParticleBundle {
    pub particle: Particle,
    pub physics: ParticlePhysicsBundle,
    pub chunk_position: ChunkPosition,
    pub visual: ParticleVisualBundle,
}

#[derive(Component, Default, Clone, Debug)]
pub struct Particle;

#[derive(Bundle, Clone, Default, Debug)]
pub struct ParticleVisualBundle {
    pub mesh: Mesh2d,
    pub mesh_material: MeshMaterial2d<ColorMaterial>,
}

#[derive(Bundle, Clone, Default, Debug)]
pub struct ParticlePhysicsBundle {
    pub transform: Transform,
    pub predicted_pos: PredictedPos,
    pub velocity: Velocity,
    pub mass: Mass,
    pub local_mass_density: LocalMassDensity,
    pub acceleration: Acceleration,
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityLookupChunk>()
            .init_resource::<SimParameters>()
            .add_plugins(ParticleAssetPlugin)
            .add_systems(Startup, spawn_particles)
            .add_systems(
                Update,
                (
                    calc_pred_pos,
                    calc_local_mass_density,
                    calc_pressure_force,
                    // artificial_motion,
                    // mouse_interact,
                    // smooth_flow,
                    calc_velocity,
                    update_particle_pos,
                )
                    .chain(),
            )
            .add_plugins(ParticleDebugPlugin);
    }
}

pub fn spawn_particles(
    mut commands: Commands,
    mut chunk: ResMut<EntityLookupChunk>,
    colors: Res<MaterialColorDatabase>,
    meshes: Res<MeshShapeDatabase>,
) {
    for x in 0..CHUNK_SIZE {
        'outer: for y in 0..CHUNK_SIZE {
            let mesh_handle = meshes.handles.get(&SimAssetId::Particle).unwrap();

            for z in 0..1 {
                // let spawn_code = ((x % 3 + 3 * y + z) % 5);
                // if spawn_code >= 3 {
                // continue 'outer;
                // }
                // let mass = (spawn_code * 3 + 1) as f32 * 1.0;

                // let material = (spawn_code);

                let spawn_code = (x + y + z) % 2;
                if spawn_code == 2 {
                    continue;
                }
                let mass = spawn_code as f32 * 3.0 + 2.0;
                let material = spawn_code;

                let color_handle = colors.handles.get(&material).unwrap();
                let mut rng = rand::thread_rng();
                let x_f = x as f32 + (rng.gen::<f32>() - 0.5) * 0.8;
                let y_f = y as f32 + (rng.gen::<f32>() - 0.5) * 0.8;

                let vel_x: f32 = rng.gen();
                let vel_y: f32 = rng.gen();

                let particle_bundle = ParticleBundle {
                    particle: Particle,
                    physics: ParticlePhysicsBundle {
                        transform: Transform::from_xyz(x_f, y_f, 0.0),
                        predicted_pos: PredictedPos(Vec2::new(x_f, y_f)),
                        velocity: Default::default(),
                        mass: Mass(mass),
                        local_mass_density: LocalMassDensity(0.5),
                        ..Default::default()
                    },
                    visual: ParticleVisualBundle {
                        mesh: Mesh2d(mesh_handle.clone()),
                        mesh_material: MeshMaterial2d(color_handle.clone()),
                    },
                    ..Default::default()
                };
                let entity = commands.spawn(particle_bundle).id();
                chunk
                    .insert(entity, x, y)
                    .expect("Adding Entity to Chunk failed");
            }
        }
    }
}

pub fn calc_velocity(mut particles: Query<(&mut Velocity, &Acceleration), With<Particle>>) {
    for (mut vel, acc) in particles.iter_mut() {
        // WARN: TEMPORARY -> = instead of +=

        let len = vel.0.length();
        vel.0 = vel.0.clamp_length_max(len * 0.9);

        let mut rng = rand::thread_rng();

        vel.0.x += acc.0.x + (rng.gen::<f32>() - 0.5) * 0.0005;
        vel.0.y += acc.0.y + (rng.gen::<f32>() - 0.5) * 0.0005;

        // println!("Vel calc: {}|{}", vel.0.x, vel.0.y);
    }
}

pub fn artificial_motion(
    mut particles: Query<(&mut Acceleration, &Transform, &LocalMassDensity), With<Particle>>,
) {
    for (mut acc, transf, density) in particles.iter_mut() {
        let edge = 5.0;
        if transf.translation.x < edge {
            let strength = (edge - transf.translation.x) / edge;

            acc.0.x += 0.2 / density.0 * strength;
            acc.0.y += 0.5 / density.0 * strength;
        }
        let edge = 32.0;
        if transf.translation.y > CHUNK_SIZE as f32 - edge {
            let strength = (edge - (CHUNK_SIZE as f32 - transf.translation.y)) / edge;

            acc.0.x += 0.5 / density.0 * strength;
            acc.0.y -= 0.2 / density.0 * strength;
        }
    }
}

pub fn mouse_interact(
    mut particles: Query<(&mut Acceleration, &Transform, &LocalMassDensity), With<Particle>>,
    mouse_pos: Res<MousePosition>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
) {
    let chunk_pos = Chunk::<Vec<Entity>>::get_chunk_pos(mouse_pos.0.x, mouse_pos.0.y);
    if chunk_pos.is_none() {
        return;
    }
    let chunk_pos = chunk_pos.unwrap();

    let entities =
        entity_lookup_chunk.get_neighborhood_entities(chunk_pos.x as usize, chunk_pos.y as usize);

    for entity in entities.iter() {
        let (mut acc, transf, density) = particles.get_mut(*entity).expect("AAAAAHHHHH");
        let diff = Vec2::new(
            transf.translation.x - mouse_pos.0.x,
            transf.translation.y - mouse_pos.0.y,
        );
        let len = diff.length();
        if len >= 1.0 || len <= 0.0001 {
            continue;
        }
        let direction = diff.normalize_or_zero();
        let strength = (1.0 - len) * 0.5;
        let force = Vec2::new(
            direction.x * strength / density.0,
            direction.y * strength / density.0,
        );

        acc.0.x += force.x;
        acc.0.y += force.y;
    }
}

pub const RATIO: f32 = 0.1;

pub fn smooth_flow(
    mut particles: Query<(&mut Acceleration, &LocalMassDensity), With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
) {
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let entities = entity_lookup_chunk
                .0
                .get(x, y)
                .expect("Should be in bounds");
            let mut accumulated = Vec2::splat(0.0);
            let mut acc_mass = 0.0;
            for entity in entities.iter() {
                let (mut acc, mass) = particles.get_mut(*entity).expect("Particle should exist");

                acc_mass += mass.0;

                let transf_x = acc.0.x * mass.0 * RATIO;
                let transf_y = acc.0.y * mass.0 * RATIO;
                accumulated.x += transf_x;
                accumulated.y += transf_y;
                acc.0.x -= transf_x;
                acc.0.y -= transf_y;
            }
            for entity in entities.iter() {
                let (mut acc, mass) = particles.get_mut(*entity).expect("Particle should exist");
                acc.0.x += accumulated.x / acc_mass * mass.0;
                acc.0.y += accumulated.y / acc_mass * mass.0;
            }
        }
    }
}

pub fn calc_pressure_force(
    mut particles: Query<(&mut Acceleration, &LocalMassDensity, &Transform), With<Particle>>,
    read_particles: Query<(&Transform, &LocalMassDensity, &Mass), With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
    params: Res<SimParameters>,
) {
    for (mut acc, mass_density, pos) in particles.iter_mut() {
        let mut rng = rand::thread_rng();
        let pos = Vec2::new(
            pos.translation.x, // + (rng.gen::<f32>() - 0.5) * 0.1,
            pos.translation.y, // + (rng.gen::<f32>() - 0.5) * 0.1,
        );
        let pressure_force =
            get_particle_pressure_gradient(pos, &read_particles, &entity_lookup_chunk, &params)
                .unwrap_or_default();
        // NOTE: usize mass_density because here it is the "local" mass
        acc.0.x = pressure_force.x / mass_density.0;
        acc.0.y = pressure_force.y / mass_density.0 - params.gravity;
        //let len = acc.0.length();
        //acc.0.clamp_length_max(len * 0.9);
    }
}

pub fn calc_local_mass_density(
    mut write_particles: Query<(&mut LocalMassDensity, &Transform), With<Particle>>,
    read_particles: Query<(&Transform, &Mass), With<Particle>>,
    entity_lookup_chunk: Res<EntityLookupChunk>,
) {
    for (mut local_density, transform) in write_particles.iter_mut() {
        let mass_density = get_particle_mass_density(
            Vec2::new(transform.translation.x, transform.translation.y),
            &read_particles,
            &entity_lookup_chunk,
        )
        .unwrap_or_default();
        local_density.0 = mass_density;
        // println!("LMD {mass_density}");
    }
}

pub fn calc_pred_pos(
    mut particle_q: Query<(&mut PredictedPos, &Transform, &Velocity), With<Particle>>,
) {
    for (mut pred, transf, &Velocity(vel)) in particle_q.iter_mut() {
        let pos = &mut pred.0;
        pos.x = transf.translation.x + vel.x;
        pos.y = transf.translation.y + vel.y;
    }
}

pub fn get_particle_density(
    at_pos: Vec2,
    particles: &Query<&Transform, With<Particle>>,
    entity_lookup_chunk: &EntityLookupChunk,
) -> Option<f32> {
    let chunk_pos: UVec2 = Chunk::<Vec<Entity>>::get_chunk_pos(at_pos.x, at_pos.y)?;
    let chunk_x = chunk_pos.x as usize;
    let chunk_y = chunk_pos.y as usize;
    //println!("Chunk_pos: {}|{}", chunk_x, chunk_y);

    let entities = entity_lookup_chunk.get_neighborhood_entities(chunk_x, chunk_y);

    let mut density = 0.0;
    for entity in entities.iter() {
        //println!("Entity: {}", entity);
        let transf = particles
            .get(*entity)
            .expect("Particle Chunk registry failed; Expected particle to exist; didn't");

        let x_diff = transf.translation.x - at_pos.x;
        let y_diff = transf.translation.y - at_pos.y;
        let dist_sq = x_diff * x_diff + y_diff * y_diff;
        if dist_sq >= 1.0 {
            continue;
        }
        let distance = f32::sqrt(dist_sq);
        let influence = distance_density_influence(distance);
        density += influence;
        //println!("Influence: {}", influence);
    }
    Some(density)
}
pub fn get_particle_mass_density(
    at_pos: Vec2,
    particles: &Query<(&Transform, &Mass), With<Particle>>,
    entity_lookup_chunk: &EntityLookupChunk,
) -> Option<f32> {
    let chunk_pos: UVec2 = Chunk::<Vec<Entity>>::get_chunk_pos(at_pos.x, at_pos.y)?;
    let chunk_x = chunk_pos.x as usize;
    let chunk_y = chunk_pos.y as usize; //println!("Chunk_pos: {}|{}", chunk_x, chunk_y);
    let entities = entity_lookup_chunk.get_neighborhood_entities(chunk_x, chunk_y);

    let mut density = 0.0;
    for entity in entities.iter() {
        //println!("Entity: {}", entity);
        let (transf, mass) = particles
            .get(*entity)
            .expect("Particle Chunk registry failed; Expected particle to exist; didn't");

        let x_diff = transf.translation.x - at_pos.x;
        let y_diff = transf.translation.y - at_pos.y;
        let dist_sq = x_diff * x_diff + y_diff * y_diff;
        if dist_sq >= 1.0 {
            continue;
        }
        let distance = f32::sqrt(dist_sq);
        let influence = distance_density_influence(distance);
        density += influence * mass.0;
        //println!("Influence: {}", influence);
    }
    Some(density)
}

pub const DAMPENING: f32 = 0.7;
pub fn update_particle_pos(
    mut particles: Query<
        (Entity, &mut Transform, &mut Velocity, &mut ChunkPosition),
        With<Particle>,
    >,
    mut entity_lookup_chunk: ResMut<EntityLookupChunk>,
) {
    for (entity, mut pos, mut vel, mut chunk_pos) in particles.iter_mut() {
        let mut new_x = pos.translation.x + vel.0.x;
        // println!("Vel: {}|{}", vel.0.x, vel.0.y);
        if new_x < 0.5 {
            new_x = 0.5;
            vel.0.x *= -DAMPENING;
        } else if new_x > (CHUNK_SIZE - 1) as f32 {
            new_x = (CHUNK_SIZE - 1) as f32;
            vel.0.x *= -DAMPENING;
        }

        let mut new_y = pos.translation.y + vel.0.y;
        if new_y < 0.5 {
            new_y = 0.5;
            vel.0.y *= -DAMPENING;
        } else if new_y > (CHUNK_SIZE - 1) as f32 {
            new_y = (CHUNK_SIZE - 1) as f32;
            vel.0.y *= -DAMPENING;
        }

        pos.translation.x = new_x;
        pos.translation.y = new_y;

        // println!("{}|{}", new_x, new_y);
        let previous = chunk_pos.pos.clone();
        // println!("ChunkPos {}|{}", chunk_pos.pos.x, chunk_pos.pos.y);

        chunk_pos.pos = Chunk::<Vec<Entity>>::get_chunk_pos(new_x, new_y)
            .expect("Particle is not allowed outside bounds");

        if previous != chunk_pos.pos {
            let mut prev_cell = entity_lookup_chunk
                .0
                .get_mut(previous.x as usize, previous.y as usize)
                .expect("All Cell positions should be valid at all times");
            prev_cell.remove(&entity);

            entity_lookup_chunk.insert(entity, chunk_pos.pos.x as usize, chunk_pos.pos.y as usize);
        }
    }
}

pub fn get_particle_pressure_gradient(
    at_pos: Vec2,
    particles: &Query<(&Transform, &LocalMassDensity, &Mass), With<Particle>>,
    entity_lookup_chunk: &EntityLookupChunk,
    params: &SimParameters,
) -> Option<Vec2> {
    let chunk_pos: UVec2 = Chunk::<Vec<Entity>>::get_chunk_pos(at_pos.x, at_pos.y)?;
    let chunk_x = chunk_pos.x as usize;
    let chunk_y = chunk_pos.y as usize; //println!("Chunk_pos: {}|{}", chunk_x, chunk_y);
    let entities = entity_lookup_chunk.get_neighborhood_entities(chunk_x, chunk_y);

    let mut rng = rand::thread_rng();

    let mut result = Vec2::ZERO;
    for entity in entities.iter() {
        let (transf, mass_density, mass) =
            particles.get(*entity).expect("Entity not found in query");
        let particle_pos = Vec2::new(transf.translation.x, transf.translation.y);
        let diff = Vec2::new(particle_pos.x - at_pos.x, particle_pos.y - at_pos.y);
        let dist = diff.length();
        if dist >= INFLUENCE_RADIUS {
            continue;
        }
        if dist <= 0.000001 {
            result = result.mul_add(
                Vec2::ONE,
                Vec2::new(
                    rng.gen::<f32>() * 0.01 - 0.005,
                    rng.gen::<f32>() * 0.01 - 0.005,
                ),
            );
            continue;
        }
        // println!("Diff: ", )
        let direction = diff.normalize_or_zero();
        // NOTE: It might be easier to just leave density out entirely and instead rely on
        // particle_density???;
        // let influence = mass.0 / pressure_from_density(mass_density.0, &pressure_mult);
        let influence = pressure_from_density(mass_density.0, &params) / mass_density.0 * mass.0;
        // let influence = mass.0 / mass_density.0 * 0.1;

        let derivative = distance_density_derivative(dist);
        let derivative_vector = Vec2::new(
            -direction.x * derivative * influence,
            -direction.y * derivative * influence,
        );
        // println!("Single deriv: {derivative_vector:?}");
        result = result.mul_add(Vec2::ONE, derivative_vector);
    }
    Some(result)
}

pub const INFLUENCE_RADIUS: f32 = 1.0; // exactly 1 tile in each direction
pub const INFLUENCE_VOLUME: f32 = 1.0;
pub fn distance_density_influence(distance: f32) -> f32 {
    if !(0.0..=INFLUENCE_RADIUS).contains(&distance) {
        return 0.0;
    }
    let x = INFLUENCE_RADIUS - distance;

    x * x * x / INFLUENCE_VOLUME
}

// The derivative will be needed to determine the speed with which the particles desire to leave
// the dense areas
pub fn distance_density_derivative(distance: f32) -> f32 {
    if !(0.0..=INFLUENCE_RADIUS).contains(&distance) {
        return 0.0;
    }

    if distance == 0.0 {
        return 0.0;
    }

    // NOTE: f(x) = (r - |x|)^3
    // --> f(x) = u(x)^3
    // --> u(x) = r - |x|
    // f'(x) = 3u(x)^2 * u'(x)
    // u'(x) = r' - |x|'
    //       = 0 - sign(x)
    //       = -sign(x)
    // f'(x) = 3(r - |x|)^2 * -sign(x)
    //
    // for this function though, we can ignore the sign since the sign is reintroduced with the
    // vector-difference

    3.0 * (INFLUENCE_RADIUS - distance).powi(2)
}

pub const TARGET_DENSITY: f32 = 0.0;

pub fn pressure_from_density(density: f32, params: &SimParameters) -> f32 {
    let error = density - TARGET_DENSITY;
    let mut val = error * params.pressure_mult;
    if val.is_nan() {
        val = 0.0;
    }
    if val.abs() <= 0.01 {
        val.signum() * 0.01
    } else {
        val
    }
}
