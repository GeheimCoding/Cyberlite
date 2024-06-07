#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

use bevy::core::Name;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::utils::{HashMap, HashSet};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::{thread_rng, Rng};
use rand_distr::StandardNormal;
use std::ffi::OsStr;
use std::mem::swap;
use std::{cmp, fs};

type Point = (usize, usize);
type Grid = Vec<Vec<f32>>;

enum Shape {
    Vertical,
    Horizontal,
    ShapeL,
    Shape7,
    ShapeF,
    ShapeJ,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    // run_app();
    let (start, end, grid) = generate_grid(16, 2, true);
    let path = calculate_shortest_path(start, end, &grid);

    render_grid(&grid);
    println!("=====================");
    render_path(&path, (grid.len(), grid[0].len()));
}

fn render_path(path: &Vec<Point>, (rows, columns): (usize, usize)) {
    calculate_closest_edge_direction(path[0], (rows, columns));
    calculate_closest_edge_direction(*path.last().unwrap(), (rows, columns));
    println!("{path:?} - {rows}x{columns}");
}

fn calculate_closest_edge_direction(point: Point, (rows, columns): (usize, usize)) -> Direction {
    let mut directions = vec![
        (point.0, Direction::Up),
        (rows - point.0, Direction::Down),
        (point.1, Direction::Left),
        (columns - point.1, Direction::Right),
    ];
    directions.sort_by(|a, b| a.0.cmp(&b.0));
    directions[0].1
}

fn calculate_shortest_path(start: Point, end: Point, grid: &Grid) -> Vec<Point> {
    let mut queue = vec![(start, 0.0, start)];
    let mut processed = HashSet::new();
    let mut path = vec![];
    while !queue.is_empty() {
        let (point, distance, previous) = queue.pop().expect("must exist");
        processed.insert(point);
        path.push((point, previous));
        if point == end {
            break;
        }
        let neighbors = get_neighbors(point, grid);
        for neighbor in neighbors {
            if processed.contains(&neighbor) {
                continue;
            }
            queue.push((neighbor, distance + grid[neighbor.0][neighbor.1], point));
        }
        queue.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("must compare"));
    }
    let mut lookup = HashMap::new();
    for (index, (point, _)) in path.iter().enumerate() {
        lookup.entry(*point).or_insert(index);
    }
    let mut shortest_path = vec![end];
    let mut point = end;
    while point != start {
        point = path[*lookup.get(&point).expect("must exist")].1;
        shortest_path.push(point);
    }
    shortest_path.reverse();
    shortest_path
}

fn get_neighbors(point: Point, grid: &Grid) -> Vec<Point> {
    let mut neighbors = vec![];
    if point.0 > 0 {
        neighbors.push((point.0 - 1, point.1));
    }
    if point.1 > 0 {
        neighbors.push((point.0, point.1 - 1));
    }
    if point.0 < grid.len() - 1 {
        neighbors.push((point.0 + 1, point.1));
    }
    if point.1 < grid[0].len() - 1 {
        neighbors.push((point.0, point.1 + 1));
    }
    neighbors
}

fn generate_random_point(distance: usize) -> Point {
    let width = thread_rng().gen_range(0..=distance);
    (width, distance - width)
}

fn generate_grid(distance: usize, border: usize, hug_edge: bool) -> (Point, Point, Grid) {
    let point = generate_random_point(distance);
    let mut start = (border, border);
    let mut end = (point.0 + border, point.1 + border);

    let mut rows = end.0 + border + 1;
    let mut columns = end.1 + border + 1;
    if thread_rng().gen_bool(0.5) {
        swap(&mut start.0, &mut end.0);
    }
    if hug_edge {
        if rows > columns {
            start.0 -= border;
            end.0 -= border;
            rows -= border * 2;
        } else {
            start.1 -= border;
            end.1 -= border;
            columns -= border * 2;
        }
    }
    let mut grid = vec![vec![0.0; columns]; rows];
    for row in 0..rows {
        for column in 0..columns {
            grid[row][column] = thread_rng().sample(StandardNormal);
        }
    }
    grid[start.0][start.1] = 0.0;
    grid[end.0][end.1] = 0.0;

    (start, end, grid)
}

fn render_grid(grid: &Grid) {
    for row in grid {
        for v in row {
            print!("{:5.0}", (v * 100.0).floor());
        }
        println!();
    }
}

fn run_app() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00018,
            speed: 180.0,
        })
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, process_models)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            transform: Transform::from_xyz(200.0, 80.0, 100.0)
                .looking_at(Vec3::new(100.0, 40.0, 0.0), Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

fn process_models(
    scenes: ResMut<Assets<Scene>>,
    mut events: EventReader<AssetEvent<Scene>>,
    mut query: Query<(&Handle<Scene>, &Name, &mut Transform)>,
    mut counter: Local<u32>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (handle, name, mut transform) in query.iter_mut() {
                if AssetId::from(handle) == *id {
                    let scene = scenes.get(*id).expect("should be loaded at this point");
                    let (aabb, base_translation) = calculate_aabb(scene);
                    let scale_factor = 42.0 / aabb.half_extents.max_element();
                    let offset = (Vec3::from(aabb.center) + base_translation) * -scale_factor;

                    println!(
                        "{name} - {} - {} -> {scale_factor}",
                        aabb.center,
                        aabb.half_extents * 2.0
                    );
                    transform.scale = Vec3::splat(scale_factor);
                    transform.translation = Vec3::new(0.0, 0.0, *counter as f32 * -100.0) + offset;
                    *counter += 1;
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
