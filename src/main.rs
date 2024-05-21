#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::core::Name;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;
use std::ffi::OsStr;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00018,
            speed: 42.0,
        })
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, process_models)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    let models = fs::read_dir("assets/models").expect("expected models folder inside assets");
    for model in models {
        if let Ok(model) = model {
            if let Some(Some("glb")) = model.path().extension().map(OsStr::to_str) {
                let name = model.file_name();
                let name =
                    String::from(&name.to_str().expect("expected valid unicode")[..name.len() - 4]);
                let model = asset_server.load(format!("models/{name}.glb#Scene0"));

                commands.spawn((
                    SceneBundle {
                        scene: model,
                        transform: Transform::from_xyz(
                            rng.gen_range(-50.0..50.0),
                            0.0,
                            rng.gen_range(-50.0..50.0),
                        ),
                        ..default()
                    },
                    Name::new(name),
                ));
            }
        }
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 20.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(40.0, 70.0, 80.0)
                .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

fn process_models(
    scenes: ResMut<Assets<Scene>>,
    mut events: EventReader<AssetEvent<Scene>>,
    mut query: Query<(&Handle<Scene>, &Name, &mut Transform)>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (handle, name, mut transform) in query.iter_mut() {
                if AssetId::from(handle) == *id {
                    let scene = scenes.get(*id).expect("should be loaded at this point");
                    let (aabb, base_translation) = calculate_aabb(scene);
                    let scale_factor = 42.0 / aabb.half_extents.max_element();
                    let center = (Vec3::from(aabb.center) + base_translation) * -scale_factor;

                    println!(
                        "{name} - {} - {} -> {scale_factor}",
                        aabb.center,
                        aabb.half_extents * 2.0
                    );

                    transform.scale = Vec3::splat(scale_factor);
                    transform.translation = center;
                    break;
                }
            }
        }
    }
}

fn calculate_aabb(scene: &Scene) -> (Aabb, Vec3) {
    let world = &scene.world;
    let mut min = Vec3::default();
    let mut max = Vec3::default();
    let mut base_translation = Vec3::default();

    for (index, entity) in world
        .iter_entities()
        .filter(has_component::<Aabb>)
        .enumerate()
    {
        let aabb = entity.get::<Aabb>().expect("must be valid");
        let translation = calculate_global_transform(world, entity.id())
            .expect("should be valid")
            .translation();
        if index == 0 {
            base_translation = translation;
        }
        let offset = translation - base_translation;
        min = min.min(Vec3::from(aabb.min()) + offset);
        max = max.max(Vec3::from(aabb.max()) + offset);
    }
    (Aabb::from_min_max(min, max), base_translation)
}

fn calculate_global_transform(world: &World, mut entity: Entity) -> Option<GlobalTransform> {
    let mut global_transform = GlobalTransform::from(*world.entity(entity).get::<Transform>()?);
    while let Some(parent) = world.entity(entity).get::<Parent>() {
        entity = parent.get();
        global_transform = *world.entity(entity).get::<Transform>()? * global_transform;
    }
    Some(global_transform)
}

fn has_component<C: Component>(entity: &EntityRef) -> bool {
    entity.get::<C>().is_some()
}
