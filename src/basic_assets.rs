use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum SimAssetId {
    Particle
}


#[derive(Resource, Default)]
pub struct MeshShapeDatabase {
    pub handles: HashMap<SimAssetId, Handle<Mesh>>,
}

#[derive(Resource, Default)]
pub struct MaterialColorDatabase {
    pub handles: HashMap<SimAssetId, Handle<ColorMaterial>>,
}





pub struct ParticleAssetPlugin;

impl Plugin for ParticleAssetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MeshShapeDatabase::default())
            .insert_resource(MaterialColorDatabase::default())
            .add_systems(PreStartup, load_particle_visuals);
    }
}

fn load_particle_visuals(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, mut mesh_db: ResMut<MeshShapeDatabase>, mut color_db: ResMut<MaterialColorDatabase>) {
    let circle = Circle::new(0.2);
    let mesh_handle = meshes.add(circle);
    mesh_db.handles.insert(SimAssetId::Particle, mesh_handle);

    let color = ColorMaterial::from_color(Color::Srgba(Srgba::rgba_u8(100, 100, 255, 20)));
    let color_handle = materials.add(color);
    color_db.handles.insert(SimAssetId::Particle, color_handle);
    println!("Loaded particle Assets");
}



