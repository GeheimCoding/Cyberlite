#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let lantern = asset_server.load("models/Lantern.glb#Scene0");
    commands.spawn(SceneBundle {
        scene: lantern,
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(12.0, 14.0, 6.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(40.0, 20.0, 20.0)
            .looking_at(Vec3::new(0.0, 12.0, 0.0), Vec3::Y),
        ..default()
    });
}
