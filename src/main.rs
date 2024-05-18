#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::core::Name;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
    let lantern = asset_server.load("models/Lantern.glb#Scene0");
    commands.spawn((
        SceneBundle {
            scene: lantern,
            transform: Transform::from_xyz(-50.0, 0.0, 0.0),
            ..default()
        },
        Name::new("Lantern"),
    ));
    let building = asset_server.load("models/Building_Brutalist_TriCorner.glb#Scene0");
    commands.spawn((
        SceneBundle {
            scene: building,
            ..default()
        },
        Name::new("Building"),
    ));

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
