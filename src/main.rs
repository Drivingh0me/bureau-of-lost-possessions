use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Startup systems run once at initialization
        .add_systems(Startup, (setup_environment, initial_grab_cursor))
        // Update systems run every frame
        .add_systems(Update, (player_look, player_move, cursor_grab_system))
        .run();
}

// Marker component for our First Person Camera
#[derive(Component)]
struct FpcCamera {
    pitch: f32,
    yaw: f32,
    sensitivity: f32,
    speed: f32,
}

impl Default for FpcCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.0015,
            speed: 5.0,
        }
    }
}

/// Spawns the basic 3D entities in our world
fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::from(LinearRgba::WHITE))),
    ));

    // A floating puzzle block target
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::Srgba(Srgba::RED))),
        Transform::from_xyz(0.0, 1.0, -5.0),
    ));

    // Light source
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // First Person Camera placed at head-height (Y = 1.6 meters)
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.6, 0.0),
        FpcCamera::default(),
    ));
}

/// System handling mouse look rotation (Pitch and Yaw)
fn player_look(
    mut mouse_motion: EventReader<MouseMotion>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &mut FpcCamera)>,
) {
    let Ok(window) = window_query.get_single() else { return };
    let Ok((mut transform, mut camera)) = camera_query.get_single_mut() else { return };

    // Only look around if the cursor is grabbed inside the window
    if window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }

    for event in mouse_motion.read() {
        // Calculate new pitch and yaw based on mouse movements
        camera.yaw -= event.delta.x * camera.sensitivity;
        camera.pitch -= event.delta.y * camera.sensitivity;

        // Clamp looking up/down to prevent rolling or flipping over the head
        camera.pitch = camera.pitch.clamp(-1.54, 1.54);

        // Apply global yaw (around world Y axis) and local pitch
        transform.rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw)
            * Quat::from_axis_angle(Vec3::X, camera.pitch);
    }
}

/// System handling basic camera movement (WASD) relative to look direction
fn player_move(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &FpcCamera)>,
) {
    let Ok((mut transform, camera)) = camera_query.get_single_mut() else { return };

    let mut direction = Vec3::ZERO;

    // Get the forward and right vectors relative to current look orientation
    let forward = transform.forward();
    let right = transform.right();

    if keyboard_input.pressed(KeyCode::KeyW) { direction += *forward; }
    if keyboard_input.pressed(KeyCode::KeyS) { direction -= *forward; }
    if keyboard_input.pressed(KeyCode::KeyA) { direction -= *right; }
    if keyboard_input.pressed(KeyCode::KeyD) { direction += *right; }

    // Prevent moving faster diagonally by normalizing the directional vector
    if direction != Vec3::ZERO {
        direction = direction.normalize();
        transform.translation += direction * camera.speed * time.delta_secs();
    }
}

/// Requests the OS to hide and lock the cursor immediately upon launch
fn initial_grab_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

/// Toggles cursor visibility: press ESCAPE to free mouse, click the window to grab it again
fn cursor_grab_system(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = window_query.get_single_mut() else { return };

    if keys.just_pressed(KeyCode::Escape) {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

