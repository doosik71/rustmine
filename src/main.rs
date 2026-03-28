use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::app::AppExit;
use std::collections::HashMap;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.75, 0.95)))
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin::default()))
        .init_resource::<LoadingState>()
        .init_resource::<ChunkMap>()
        .init_resource::<MenuState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement_system,
                player_animation_system,
                shadow_system,
                camera_orbit_system,
                terrain_stream_system,
                menu_toggle_system,
                menu_button_system,
                hud_system,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct ThirdPersonCamera;

#[derive(Component)]
struct PlayerPhysics {
    velocity: Vec3,
    on_ground: bool,
}

#[derive(Component, Clone)]
struct Limb {
    kind: LimbKind,
    base: Transform,
}

#[derive(Component, Default)]
struct PlayerAnim {
    phase: f32,
}

#[derive(Clone, Copy)]
enum LimbKind {
    Head,
    Torso,
    ArmL,
    ArmR,
    LegL,
    LegR,
}

#[derive(Component)]
struct Shadow;

#[derive(Component)]
struct Terrain;

#[derive(Resource, Default)]
struct LoadingState {
    loading: bool,
    progress: f32,
}

#[derive(Resource, Default)]
struct MenuState {
    open: bool,
}

#[derive(Resource)]
struct CameraRig {
    yaw: f32,
    pitch: f32,
    distance: f32,
    height: f32,
    sensitivity: f32,
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct LoadingText;

#[derive(Component)]
struct LoadingOverlay;

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct ExitButton;

#[derive(Resource, Default)]
struct ChunkMap {
    entities: HashMap<IVec2, Entity>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loading: ResMut<LoadingState>,
) {
    // Player root
    let player_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.75, 0.65),
        perceptual_roughness: 0.9,
        ..default()
    });
    let player = commands.spawn((
        Player,
        PlayerPhysics {
            velocity: Vec3::ZERO,
            on_ground: false,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        PlayerAnim::default(),
    ));
    let player_entity = player.id();

    // Humanoid parts
    let torso_mesh = meshes.add(Cuboid::new(0.8, 1.0, 0.4));
    let head_mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    let arm_mesh = meshes.add(Cuboid::new(0.25, 0.7, 0.25));
    let leg_mesh = meshes.add(Cuboid::new(0.28, 1.0, 0.28));
    let eye_mesh = meshes.add(Cuboid::new(0.08, 0.08, 0.02));
    let nose_mesh = meshes.add(Cuboid::new(0.05, 0.08, 0.02));
    let mouth_mesh = meshes.add(Cuboid::new(0.2, 0.05, 0.02));
    let eye_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.05, 0.05),
        unlit: true,
        ..default()
    });
    let nose_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.55, 0.45),
        unlit: true,
        ..default()
    });
    let mouth_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.2, 0.2),
        unlit: true,
        ..default()
    });

    commands.entity(player_entity).with_children(|parent| {
        let torso_base = Transform::from_xyz(0.0, 0.8, 0.0);
        parent.spawn((
            Mesh3d(torso_mesh),
            MeshMaterial3d(player_mat.clone()),
            torso_base,
            Limb {
                kind: LimbKind::Torso,
                base: torso_base,
            },
        ));

        let head_base = Transform::from_xyz(0.0, 1.65, 0.0);
        parent
            .spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(player_mat.clone()),
                head_base,
                Limb {
                    kind: LimbKind::Head,
                    base: head_base,
                },
            ))
            .with_children(|head_parent| {
                let face_z = -0.26;
                head_parent.spawn((
                    Mesh3d(eye_mesh.clone()),
                    MeshMaterial3d(eye_mat.clone()),
                    Transform::from_xyz(-0.12, 0.08, face_z),
                ));
                head_parent.spawn((
                    Mesh3d(eye_mesh),
                    MeshMaterial3d(eye_mat.clone()),
                    Transform::from_xyz(0.12, 0.08, face_z),
                ));
                head_parent.spawn((
                    Mesh3d(nose_mesh),
                    MeshMaterial3d(nose_mat),
                    Transform::from_xyz(0.0, 0.0, face_z),
                ));
                head_parent.spawn((
                    Mesh3d(mouth_mesh),
                    MeshMaterial3d(mouth_mat),
                    Transform::from_xyz(0.0, -0.12, face_z),
                ));
            });

        let arm_l_base = Transform::from_xyz(-0.55, 0.8, 0.0);
        parent.spawn((
            Mesh3d(arm_mesh.clone()),
            MeshMaterial3d(player_mat.clone()),
            arm_l_base,
            Limb {
                kind: LimbKind::ArmL,
                base: arm_l_base,
            },
        ));

        let arm_r_base = Transform::from_xyz(0.55, 0.8, 0.0);
        parent.spawn((
            Mesh3d(arm_mesh),
            MeshMaterial3d(player_mat.clone()),
            arm_r_base,
            Limb {
                kind: LimbKind::ArmR,
                base: arm_r_base,
            },
        ));

        let leg_l_base = Transform::from_xyz(-0.25, -0.2, 0.0);
        parent.spawn((
            Mesh3d(leg_mesh.clone()),
            MeshMaterial3d(player_mat.clone()),
            leg_l_base,
            Limb {
                kind: LimbKind::LegL,
                base: leg_l_base,
            },
        ));

        let leg_r_base = Transform::from_xyz(0.25, -0.2, 0.0);
        parent.spawn((
            Mesh3d(leg_mesh),
            MeshMaterial3d(player_mat),
            leg_r_base,
            Limb {
                kind: LimbKind::LegR,
                base: leg_r_base,
            },
        ));
    });

    // Ground shadow (blob)
    let shadow_mesh = meshes.add(Plane3d::default().mesh().size(1.4, 1.4));
    let shadow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.35),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Shadow,
        Mesh3d(shadow_mesh),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.02, 0.0),
    ));

    commands.insert_resource(CameraRig {
        yaw: -135.0_f32.to_radians(),
        pitch: -20.0_f32.to_radians(),
        distance: 8.0,
        height: 2.5,
        sensitivity: 0.003,
    });

    // 3D camera
    commands.spawn((
        Camera3d::default(),
        Camera { order: 0, ..default() },
        Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCamera,
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(20.0, 40.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // UI camera (render after 3D)
    commands.spawn((Camera2d, Camera { order: 1, ..default() }));

    // FPS text (top-right)
    commands.spawn((
        Text::new("FPS: 0"),
        TextFont::from_font_size(18.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(10.0),
            ..default()
        },
        ZIndex(10),
        FpsText,
    ));

    // Loading overlay with centered text
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.45)),
            ZIndex(5),
            LoadingOverlay,
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LOADING 0%"),
                TextFont::from_font_size(28.0),
                TextColor(Color::WHITE),
                LoadingText,
            ));
        });

    // Crosshair (two lines)
    let crosshair_size = 12.0;
    let crosshair_thickness = 2.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                ..default()
            },
            ZIndex(20),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-crosshair_size / 2.0),
                    top: Val::Px(-crosshair_thickness / 2.0),
                    width: Val::Px(crosshair_size),
                    height: Val::Px(crosshair_thickness),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-crosshair_thickness / 2.0),
                    top: Val::Px(-crosshair_size / 2.0),
                    width: Val::Px(crosshair_thickness),
                    height: Val::Px(crosshair_size),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
        });

    // Pause menu (hidden by default)
    commands
        .spawn((
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.6)),
            ZIndex(30),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(320.0),
                        height: Val::Px(220.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.1, 0.14, 0.9)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Paused"),
                        TextFont::from_font_size(28.0),
                        TextColor(Color::WHITE),
                    ));
                    panel.spawn((
                        Button,
                        ContinueButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.5, 0.3, 0.9)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Continue"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::WHITE),
                        ));
                    });
                    panel.spawn((
                        Button,
                        ExitButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.6, 0.2, 0.2, 0.9)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Exit"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });

    loading.loading = true;
    loading.progress = 0.0;
}

fn player_movement_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&mut Transform, &mut PlayerPhysics), With<Player>>,
    rig: Res<CameraRig>,
    menu: Res<MenuState>,
) {
    if menu.open {
        return;
    }
    let Ok((mut transform, mut phys)) = player_q.single_mut() else {
        return;
    };

    let forward = Vec3::new(rig.yaw.cos(), 0.0, rig.yaw.sin()).normalize();
    let right = Vec3::new(-rig.yaw.sin(), 0.0, rig.yaw.cos()).normalize();

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if dir.length_squared() > 0.0 {
        dir = dir.normalize();
    }

    let is_running = keys.pressed(KeyCode::ShiftLeft);
    let speed = if is_running { 18.0 } else { 6.0 };
    let gravity = -20.0;
    let jump_speed = 7.5;
    let dt = time.delta_secs();

    phys.velocity.x = dir.x * speed;
    phys.velocity.z = dir.z * speed;
    phys.velocity.y += gravity * dt;

    if keys.just_pressed(KeyCode::Space) && phys.on_ground {
        phys.velocity.y = jump_speed;
        phys.on_ground = false;
    }

    transform.translation += phys.velocity * dt;

    let ground_y = height_at(transform.translation.x, transform.translation.z);
    let min_y = ground_y + 0.7;
    if transform.translation.y < min_y {
        transform.translation.y = min_y;
        phys.velocity.y = 0.0;
        phys.on_ground = true;
    }

    if dir.length_squared() > 0.0 {
        transform.look_to(Vec3::new(dir.x, 0.0, dir.z), Vec3::Y);
    }
}

fn player_animation_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&PlayerPhysics, &mut PlayerAnim), With<Player>>,
    mut limbs: Query<(&mut Transform, &Limb)>,
    menu: Res<MenuState>,
) {
    if menu.open {
        return;
    }
    let Ok((phys, mut anim)) = player_q.single_mut() else {
        return;
    };

    let moving = Vec3::new(phys.velocity.x, 0.0, phys.velocity.z).length() > 0.1;
    let run = keys.pressed(KeyCode::ShiftLeft);
    let speed = if run { 7.5 } else { 6.75 };

    if moving {
        anim.phase += time.delta_secs() * speed;
    } else {
        anim.phase = 0.0;
    }

    let swing = anim.phase.sin() * if run { 0.9 } else { 0.6 };
    let knee = (anim.phase + std::f32::consts::FRAC_PI_2).sin() * 0.3;

    for (mut transform, limb) in limbs.iter_mut() {
        let mut t = limb.base;
        match limb.kind {
            LimbKind::ArmL => {
                t.rotate_x(swing);
            }
            LimbKind::ArmR => {
                t.rotate_x(-swing);
            }
            LimbKind::LegL => {
                t.rotate_x(-swing);
            }
            LimbKind::LegR => {
                t.rotate_x(swing);
            }
            LimbKind::Head => {
                t.rotate_y(swing * 0.1);
                t.rotate_x(knee * 0.05);
            }
            LimbKind::Torso => {
                t.rotate_y(swing * 0.05);
            }
        }
        transform.translation = t.translation;
        transform.rotation = t.rotation;
    }
}

fn camera_orbit_system(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut rig: ResMut<CameraRig>,
    mut cursor_opts: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut queries: ParamSet<(
        Query<&Transform, (With<Player>, Without<ThirdPersonCamera>)>,
        Query<&mut Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    )>,
    menu: Res<MenuState>,
) {
    if menu.open {
        if let Ok(mut opts) = cursor_opts.single_mut() {
            opts.grab_mode = CursorGrabMode::None;
            opts.visible = true;
        }
        return;
    }
    if let Ok(mut opts) = cursor_opts.single_mut() {
        if opts.grab_mode == CursorGrabMode::None {
            opts.grab_mode = CursorGrabMode::Locked;
            opts.visible = false;
        }
    }

    let delta = mouse_motion.delta;
    rig.yaw += delta.x * rig.sensitivity;
    rig.pitch -= delta.y * rig.sensitivity;
    rig.pitch = rig.pitch.clamp(-0.9, 0.5);

    let target = {
        let p0 = queries.p0();
        let Ok(player_tf) = p0.single() else {
            return;
        };
        player_tf.translation + Vec3::Y * 1.0
    };

    let mut p1 = queries.p1();
    let Ok(mut cam_tf) = p1.single_mut() else {
        return;
    };

    let forward = Vec3::new(
        rig.yaw.cos() * rig.pitch.cos(),
        rig.pitch.sin(),
        rig.yaw.sin() * rig.pitch.cos(),
    )
    .normalize();
    let offset = -forward * rig.distance + Vec3::Y * rig.height;
    let mut cam_pos = target + offset;
    let ground_y = height_at(cam_pos.x, cam_pos.z);
    let min_cam_y = ground_y + 0.8;
    if cam_pos.y < min_cam_y {
        cam_pos.y = min_cam_y;
    }
    cam_tf.translation = cam_pos;
    cam_tf.look_at(target, Vec3::Y);
}

fn shadow_system(
    player_q: Query<&Transform, (With<Player>, Without<Shadow>)>,
    mut shadow_q: Query<&mut Transform, (With<Shadow>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(mut shadow_tf) = shadow_q.single_mut() else {
        return;
    };
    let ground_y = height_at(player_tf.translation.x, player_tf.translation.z);
    shadow_tf.translation = Vec3::new(player_tf.translation.x, ground_y + 0.02, player_tf.translation.z);
}

fn menu_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    mut menu_vis: Query<&mut Visibility, With<MenuRoot>>,
    mut cursor_opts: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.open = !menu.open;
    }
    if let Ok(mut vis) = menu_vis.single_mut() {
        *vis = if menu.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut opts) = cursor_opts.single_mut() {
        if menu.open {
            opts.grab_mode = CursorGrabMode::None;
            opts.visible = true;
        }
    }
}

fn menu_button_system(
    mut menu: ResMut<MenuState>,
    mut menu_vis: Query<&mut Visibility, With<MenuRoot>>,
    mut cursor_opts: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut commands: Commands,
    mut buttons: Query<(&Interaction, &mut BackgroundColor, Option<&ContinueButton>, Option<&ExitButton>), Changed<Interaction>>,
) {
    for (interaction, mut color, is_continue, is_exit) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                if is_continue.is_some() {
                    menu.open = false;
                    if let Ok(mut vis) = menu_vis.single_mut() {
                        *vis = Visibility::Hidden;
                    }
                    if let Ok(mut opts) = cursor_opts.single_mut() {
                        opts.grab_mode = CursorGrabMode::Locked;
                        opts.visible = false;
                    }
                } else if is_exit.is_some() {
                    commands.write_message(AppExit::Success);
                }
            }
            Interaction::Hovered => {
                color.0.set_alpha(1.0);
            }
            Interaction::None => {
                color.0.set_alpha(0.9);
            }
        }
    }
}

fn hud_system(
    diagnostics: Res<DiagnosticsStore>,
    loading: Res<LoadingState>,
    mut texts: ParamSet<(
        Query<&mut Text, With<FpsText>>,
        Query<&mut Text, With<LoadingText>>,
    )>,
    mut overlay_query: Query<&mut Visibility, With<LoadingOverlay>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    if let Ok(mut text) = texts.p0().single_mut() {
        text.0 = format!("FPS: {}", fps.round() as u32);
    }

    if let Ok(mut text) = texts.p1().single_mut() {
        if loading.loading {
            text.0 = format!("LOADING {}%", (loading.progress * 100.0) as u32);
        } else {
            text.0.clear();
        }
    }

    if let Ok(mut vis) = overlay_query.single_mut() {
        *vis = if loading.loading {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

const CHUNK_SIZE: i32 = 32;
const CHUNK_RES: usize = 32;
const CHUNK_RADIUS: i32 = 2;
const HEIGHT_SCALE: f32 = 6.0;
const NOISE_FREQ: f32 = 0.04;

fn terrain_stream_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_map: ResMut<ChunkMap>,
    mut loading: ResMut<LoadingState>,
    cam_query: Query<&Transform, With<Player>>,
) {
    let Ok(cam_transform) = cam_query.single() else {
        return;
    };

    let cam_pos = cam_transform.translation;
    let cam_chunk = IVec2::new(
        (cam_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (cam_pos.z / CHUNK_SIZE as f32).floor() as i32,
    );

    let mut needed = Vec::new();
    for dz in -CHUNK_RADIUS..=CHUNK_RADIUS {
        for dx in -CHUNK_RADIUS..=CHUNK_RADIUS {
            let c = cam_chunk + IVec2::new(dx, dz);
            if !chunk_map.entities.contains_key(&c) {
                needed.push(c);
            }
        }
    }

    // Despawn far chunks
    let keep_min = cam_chunk - IVec2::splat(CHUNK_RADIUS + 1);
    let keep_max = cam_chunk + IVec2::splat(CHUNK_RADIUS + 1);
    let mut to_remove = Vec::new();
    for (pos, entity) in chunk_map.entities.iter() {
        if pos.x < keep_min.x
            || pos.x > keep_max.x
            || pos.y < keep_min.y
            || pos.y > keep_max.y
        {
            to_remove.push(*pos);
            commands.entity(*entity).despawn();
        }
    }
    for pos in to_remove {
        chunk_map.entities.remove(&pos);
    }

    // Spawn a limited number of chunks per frame
    let mut spawned = 0;
    for chunk_pos in needed {
        if spawned >= 2 {
            break;
        }
        let mesh = build_chunk_mesh(chunk_pos);
        let mesh_handle = meshes.add(mesh);
        let mat_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.7, 0.3),
            perceptual_roughness: 1.0,
            ..default()
        });
        let world_x = chunk_pos.x as f32 * CHUNK_SIZE as f32;
        let world_z = chunk_pos.y as f32 * CHUNK_SIZE as f32;
        let entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(mat_handle),
                Transform::from_xyz(world_x, 0.0, world_z),
                Terrain,
            ))
            .id();
        chunk_map.entities.insert(chunk_pos, entity);
        spawned += 1;
    }

    let total_target = (CHUNK_RADIUS * 2 + 1).pow(2) as f32;
    let have = chunk_map.entities.len() as f32;
    let progress = (have / total_target).clamp(0.0, 1.0);
    loading.progress = progress;
    loading.loading = progress < 1.0;
}

fn build_chunk_mesh(chunk_pos: IVec2) -> Mesh {
    let step = CHUNK_SIZE as f32 / CHUNK_RES as f32;
    let vert_count = (CHUNK_RES + 1) * (CHUNK_RES + 1);

    let mut heights = vec![0.0f32; vert_count];
    for z in 0..=CHUNK_RES {
        for x in 0..=CHUNK_RES {
            let idx = z * (CHUNK_RES + 1) + x;
            let world_x =
                chunk_pos.x as f32 * CHUNK_SIZE as f32 + x as f32 * step;
            let world_z =
                chunk_pos.y as f32 * CHUNK_SIZE as f32 + z as f32 * step;
            heights[idx] = height_at(world_x, world_z);
        }
    }

    let mut positions = Vec::with_capacity(vert_count);
    let mut normals = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    for z in 0..=CHUNK_RES {
        for x in 0..=CHUNK_RES {
            let idx = z * (CHUNK_RES + 1) + x;
            let px = x as f32 * step;
            let pz = z as f32 * step;
            let h = heights[idx];

            positions.push([px, h, pz]);
            uvs.push([x as f32 / CHUNK_RES as f32, z as f32 / CHUNK_RES as f32]);

            let world_x = chunk_pos.x as f32 * CHUNK_SIZE as f32 + px;
            let world_z = chunk_pos.y as f32 * CHUNK_SIZE as f32 + pz;
            let sample = step.max(0.5);
            let h_l = height_at(world_x - sample, world_z);
            let h_r = height_at(world_x + sample, world_z);
            let h_d = height_at(world_x, world_z - sample);
            let h_u = height_at(world_x, world_z + sample);
            let normal = Vec3::new(h_l - h_r, 2.5, h_d - h_u).normalize();
            normals.push(normal.to_array());
        }
    }

    let mut indices = Vec::with_capacity(CHUNK_RES * CHUNK_RES * 6);
    for z in 0..CHUNK_RES {
        for x in 0..CHUNK_RES {
            let i0 = (z * (CHUNK_RES + 1) + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + (CHUNK_RES + 1) as u32;
            let i3 = i2 + 1;
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn height_at(x: f32, z: f32) -> f32 {
    let mut amp = 1.0;
    let mut freq = NOISE_FREQ;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for _ in 0..3 {
        sum += value_noise(x * freq, z * freq) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    let n = (sum / norm) * 2.0 - 1.0;
    n * HEIGHT_SCALE
}

fn value_noise(x: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let sx = smoothstep(x - x0 as f32);
    let sz = smoothstep(z - z0 as f32);

    let n00 = hash01(x0, z0);
    let n10 = hash01(x1, z0);
    let n01 = hash01(x0, z1);
    let n11 = hash01(x1, z1);

    let ix0 = lerp(n00, n10, sx);
    let ix1 = lerp(n01, n11, sx);
    lerp(ix0, ix1, sz)
}

fn hash01(x: i32, z: i32) -> f32 {
    let mut h = x as u64;
    h = h
        .wrapping_mul(374761393)
        .wrapping_add((z as u64).wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
