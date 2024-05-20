#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::core::Name;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
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
