use bevy::{
    prelude::*,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::mouse::{
        MouseMotion,
        MouseWheel
    },
};

fn main() {
    App::build()
    .add_resource(Msaa { samples: 4 })
    .init_resource::<State>()
    .add_plugins(DefaultPlugins)
    .add_plugin(FrameTimeDiagnosticsPlugin)
    .add_startup_system(setup.system())
    .add_system(process_mouse_events.system())
    .add_system(update_camera.system())
    .add_system(update_play.system())
    .run();
}

struct Position {
    yaw: f32,

    camera_distance: f32,
    camera_pitch: f32,
    camera_entity: Option<Entity>,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            yaw: 0.,

            camera_distance: 20.,
            camera_pitch: 30.0f32.to_radians(),
            camera_entity: None,
        }
    }
}

#[derive(Default)]
struct Player {
    pos_translation: Vec3,
    pos_rotation: Quat
}

#[derive(Default)]
struct State {
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let cube_mat_handle = materials.add({
        let mut cube_material: StandardMaterial = Color::rgb(1.0, 1.0, 1.0).into();
        cube_material.shaded = true;
        cube_material
    });

    let camera_entity = commands
        .spawn(Camera3dBundle::default())
        .current_entity();

    let pos_entity = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {size:0.1f32})),
            material: cube_mat_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0.5, 0.)),
            ..Default::default()
        })
        .with(Position {
            camera_entity,
            ..Default::default()
        })
        .current_entity();

    commands
        .push_children(pos_entity.unwrap(), &[camera_entity.unwrap()])
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.7, 0.3, 0.0).into()),
            ..Default::default()
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
            ..Default::default()
        });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {size: 1.})),
            material: cube_mat_handle,
            transform: Transform::from_translation(Vec3::new(0., 0.5, 0.)),
            ..Default::default()
        })
        .with(Player::default());
}

fn process_mouse_events(
    time: Res<Time>,
    mut state: ResMut<State>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    mut query: Query<&mut Position>,
) {
    let mut look = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        look = event.delta;
    }

    let mut zoom_delta = 0.;
    for event in state.mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        zoom_delta = event.y;
    }

    let zoom_sense = 10.0;
    let look_sense = 1.0;
    let delta_seconds = time.delta_seconds();

    for mut pos in &mut query.iter_mut() {
        pos.yaw += look.x * delta_seconds;
        pos.camera_pitch -= look.y * delta_seconds * look_sense;
        pos.camera_distance -= zoom_delta * delta_seconds * zoom_sense;
    }
}

fn update_camera (
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(Query<(&mut Position, &mut Transform)>, Query<&mut Transform>)>,
    mut query: Query<&mut Player>
) {
    let mut movement = Vec2::zero();
    if keyboard_input.pressed(KeyCode::W) { movement.y += 1.; }
    if keyboard_input.pressed(KeyCode::S) { movement.y -= 1.; }
    if keyboard_input.pressed(KeyCode::D) { movement.x += 1.; }
    if keyboard_input.pressed(KeyCode::A) { movement.x -= 1.; }

    if movement != Vec2::zero() { movement.normalize(); }

    let move_speed = 10.0;
    movement *= time.delta_seconds() * move_speed;

    let mut cam_positions = Vec::new();

    let mut pos_translation = Vec3::zero();
    let mut pos_rotation = Quat::identity();

    for (mut pos, mut transform) in &mut queries.q0_mut().iter_mut() {
        pos.camera_pitch = pos.camera_pitch.max(1f32.to_radians()).min(179f32.to_radians());
        pos.camera_distance = pos.camera_distance.max(5.).min(30.);

        let fwd = transform.forward();
        let right = Vec3::cross(fwd, Vec3::unit_y());
        let fwd = fwd * movement.y;
        let right = right * movement.x;

        transform.translation += Vec3::from(fwd + right);
        transform.rotation = Quat::from_rotation_y(-pos.yaw);

        pos_translation = transform.translation;
        pos_rotation = transform.rotation;

        if let Some(camera_entity) = pos.camera_entity {
            let cam_pos = Vec3::new(0., pos.camera_pitch.cos(), -pos.camera_pitch.sin()).normalize() * pos.camera_distance;
            cam_positions.push((camera_entity, cam_pos));
        }
    }

    for (camera_entity, cam_pos) in cam_positions.iter() {
        if let Ok(mut cam_trans) = queries.q1_mut().get_component_mut::<Transform>(*camera_entity) {
            cam_trans.translation = *cam_pos;

            let look = Mat4::face_toward(cam_trans.translation, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
            cam_trans.rotation = look.to_scale_rotation_translation().1;
            pos_rotation *= cam_trans.rotation;
        }
    }

    for mut player in &mut query.iter_mut() {
        player.pos_translation = pos_translation;
        player.pos_rotation = pos_rotation;
    }
}

fn update_play (
    mut query: Query<(&mut Player, &mut Transform)>
) {
    for (player, mut transform) in query.iter_mut() {
        transform.translation = player.pos_translation;
        let lerp_rotation = transform.rotation.lerp(player.pos_rotation, 0.05);
        transform.rotation = lerp_rotation;
    }
}
