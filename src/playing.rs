use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind::*, pbr::light_consts::lux,
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use noiz::prelude::*;

use crate::{
    AppState,
    movement::{FastFall, Grounded},
    terrain_chunking::{
        LandChunkNoise, Obstacle, TERRAIN_AMPLITUDE,
    },
};

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeCastGrounded>()
            .init_resource::<HitstopTimer>()
            .add_systems(
                OnEnter(AppState::Playing),
                start_playing,
            )
            .add_systems(
                OnExit(AppState::Playing),
                stop_playing,
            )
            .add_systems(
                FixedUpdate,
                (
                    // min_linear,
                    gravity, casting,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    camera_follow,
                    debug_side_camera_follow,
                    update_speed_text,
                    tick_hitstop,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                PhysicsSchedule,
                update_previous_velocity
                    .in_set(PhysicsStepSet::First),
            );
    }
}

fn update_previous_velocity(
    mut query: Query<(
        &LinearVelocity,
        &mut LastFrameVelocity,
    )>,
) {
    for (velocity, mut last_frame_velocity) in &mut query {
        last_frame_velocity.0 = velocity.0;
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
struct ShapeCastGrounded(bool);

#[derive(Component)]
struct SpeedText;

#[derive(Component)]
struct GroundedText;

#[derive(Component, Debug)]
pub struct LastFrameVelocity(Vec3);

#[derive(Resource)]
struct HitstopTimer(Timer);

impl Default for HitstopTimer {
    fn default() -> Self {
        // 0.02 is the slowdown virtual time
        // 0.020 is the "realtime" timer time
        Self(Timer::from_seconds(
            0.01,
            TimerMode::Once,
        ))
    }
}

fn start_playing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    noise: Res<LandChunkNoise>,
    // time: ResMut<Time<Virtual>>,
    asset_server: Res<AssetServer>,
) {
    // time.set_relative_speed(0.02);

    // Text with multiple sections
    commands.spawn((
        Node{
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        children![(
            Node {
                padding: UiRect { bottom: Val::Px(50.),..default() },
                ..default()
            },
            Text::new(""),
            TextColor(SLATE_50.into()),
            children![
                (
                    TextSpan::default(),
                    TextFont {
                        // This font is loaded and will be used instead of the default font.
                        font: asset_server
                            .load("fonts/Alfa_Slab_One/AlfaSlabOne-Regular.ttf"),
                        font_size: 42.0,
                        ..default()
                    },
                    SpeedText,
                ),
                (
                    TextSpan::new(" m/s"),
                    TextFont {
                        // This font is loaded and will be used instead of the default font.
                        font: asset_server
                            .load("fonts/Alfa_Slab_One/AlfaSlabOne-Regular.ttf"),
                        font_size: 20.0,
                        ..default()
                    }
                )
            ],
        )]
    ));

    // Text with multiple sections
    commands.spawn((
    Node{
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexEnd,
        ..default()
    },
    children![(
        Node {
            padding: UiRect { bottom: Val::Px(100.),..default() },
            ..default()
        },
        Text::new("Grounded: "),
        TextColor(SLATE_50.into()),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font: asset_server
                .load("fonts/Alfa_Slab_One/AlfaSlabOne-Regular.ttf"),
            font_size: 42.0,
            ..default()
        },
        children![
            (
                TextSpan::default(),
                TextFont {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server
                        .load("fonts/Alfa_Slab_One/AlfaSlabOne-Regular.ttf"),
                    font_size: 42.0,
                    ..default()
                },
                GroundedText,
            ),
        ],
    )]
));

    info!("start playing");
    commands.spawn(DirectionalLight {
        shadows_enabled: true,
        // lux::RAW_SUNLIGHT is recommended for use with
        // this feature, since other values
        // approximate sunlight *post-scattering* in various
        // conditions. RAW_SUNLIGHT in comparison is the
        // illuminance of the sun unfiltered by the
        // atmosphere, so it is the proper input for
        // sunlight to be filtered by the atmosphere.
        illuminance: lux::RAW_SUNLIGHT,
        ..default()
    });

    // for i in 0..40 {

    // }

    let player_position: f32 = noise.sample(Vec3::ZERO);
    // let player_position = 0.;

    commands
        .spawn((
            Player,
            Name::new("Player"),
            TransformInterpolation,
            Actions::<Grounded>::default(),
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.))),
            MeshMaterial3d(materials.add(
                StandardMaterial {
                    base_color: SLATE_400.into(),
                    ..default()
                },
            )),
            Transform::from_xyz(
                0.,
                player_position * TERRAIN_AMPLITUDE + 1.,
                0.,
            ),
            RigidBody::Kinematic,
            Collider::capsule(0.5, 1.),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity(Vec3 {
                x: 0.,
                y: 0.,
                z: -50.,
            }),
            LastFrameVelocity(Vec3 {
                x: 0.,
                y: 0.,
                z: -50.,
            }),
            CollisionEventsEnabled,
            ShapeCaster::new(
                Collider::capsule(0.5, 1.),
                Vec3::ZERO,
                Quat::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.2),
        ))
        .observe(
            |trigger: Trigger<OnCollisionStart>,
             obstacles: Query<&Obstacle>,
             mut time: ResMut<Time<Virtual>>,
             mut hitstop_timer: ResMut<HitstopTimer>,
             mut commands: Commands| {
                if obstacles.get(trigger.collider).is_ok() {
                    info!("colliding");
                    // info!(event=?trigger.event());
                    // start with double `Virtual` time
                    // resulting in one of the sprites
                    // moving at twice the speed
                    // of the other sprite which moves based
                    // on `Real` (unscaled) time
                    time.set_relative_speed(0.02);
                    *hitstop_timer = Default::default();
                    commands
                        .entity(trigger.collider)
                        .despawn();
                }
            },
        );
}

fn tick_hitstop(
    mut hitstop_timer: ResMut<HitstopTimer>,
    time: Res<Time>,
    mut virtual_time: ResMut<Time<Virtual>>,
    // real_time: Res<Time<Real>>
) {
    if hitstop_timer.0.tick(time.delta()).just_finished() {
        virtual_time.set_relative_speed(1.);
    }
}

fn stop_playing() {}

#[derive(Component)]
pub struct PlayerFollowCamera;

fn camera_follow(
    mut query: Query<
        &mut Transform,
        (
            With<PlayerFollowCamera>,
            Without<Player>,
        ),
    >,
    player: Single<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for mut transform in &mut query {
        transform.translation.smooth_nudge(
            &(player.translation + Vec3::new(0., 3., 4.)),
            4.,
            time.delta_secs(),
        );
        transform.look_at(player.translation, Vec3::Y);
    }
}

#[derive(Component)]
pub struct PlayerFollowCameraDebug;

fn debug_side_camera_follow(
    mut query: Query<
        &mut Transform,
        (
            With<PlayerFollowCameraDebug>,
            Without<Player>,
        ),
    >,
    player: Single<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for mut transform in &mut query {
        transform.translation.smooth_nudge(
            &(player.translation + Vec3::new(-10., 0., 0.)),
            10.,
            time.delta_secs(),
        );
        // .looking_at(Vec3::ZERO, Vec3::Y)
    }
}

fn gravity(
    mut query: Query<
        (
            &mut LinearVelocity,
            // &ShapeCaster,
            &ShapeHits,
            &Actions<Grounded>,
            &LastFrameVelocity,
            &Transform,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    mut grounded_text: Query<
        &mut TextSpan,
        With<GroundedText>,
    >,
    mut shape_cast_grounded: ResMut<ShapeCastGrounded>,
    mut gizmos: Gizmos,
) {
    for (
        mut velocity,
        // shape_caster,
        shape_hits,
        actions,
        last_frame_velocity,
        transform,
    ) in &mut query
    {
        gizmos.arrow(
            transform.translation,
            transform.translation + velocity.0,
            SKY_400,
        );
        match shape_hits.iter().next() {
            Some(shape_hit_data) => {
                // gizmos.arrow(
                //     shape_hit_data.point1,
                //     shape_hit_data.point1
                //         + shape_hit_data.normal1 * 5.,
                //     SKY_400
                // );

                // info!(?shape_hit_data);
                // don't apply gravity
                for mut text in &mut grounded_text {
                    text.0 = "True".to_string();
                }
                // TODO: dot(cross(normal,up?))
                // shape_hit_data.normal1;

                // .normalize()
                if !shape_cast_grounded.0 {
                    info!(?velocity, ?last_frame_velocity);
                    let tangent = shape_hit_data
                        .normal1
                        .cross(Vec3::X);
                    // let angle =
                    // shape_hit_data.normal1.dot(velocity.
                    // 0.normalize());
                    let angle_2 = tangent.dot(
                        last_frame_velocity.0.normalize(),
                    );

                    if angle_2 > 0.99 {
                        info!("perfect");
                    } else if angle_2 > 0.98 {
                        info!("good");
                    } else if angle_2 > 0.95 {
                        info!("ok");
                    } else {
                        info!("meh");
                    }

                    // gizmos.arrow(
                    //     shape_hit_data.point1,
                    //     shape_hit_data.point1
                    //         + shape_hit_data.
                    //           normal1 * 5.,
                    //     SKY_400
                    // );
                    // info!(angle=?angle.
                    // to_degrees());
                    // cos(90deg) == 0
                    // angle.to_degrees()
                }
                shape_cast_grounded.0 = true;
            }
            None => {
                for mut text in &mut grounded_text {
                    text.0 = "False".to_string();
                }
                if actions
                    .state::<FastFall>()
                    .is_ok_and(|v| v == ActionState::Fired)
                {
                    // apply gravity
                    velocity.y -=
                        9.8 * 7. * time.delta_secs();
                } else {
                    // apply gravity
                    velocity.y -=
                        9.8 * 2. * time.delta_secs();
                }

                shape_cast_grounded.0 = false;

                continue;
            }
        }
    }
}

// fn min_linear(
//     mut query: Query<&mut LinearVelocity>,
//     time: Res<Time>,
// ) {
//     // let delta_secs = time.delta_secs();
//     // for linear_velocity in &mut query {
//         // Accelerate the entity towards +X at
//         // `2.0` units per second squared.
//         // linear_velocity.z = -30.;
//     }
// }

fn casting(
    spatial_query: SpatialQuery,
    mut characters: Query<
        (
            Entity,
            &Collider,
            &Position,
            // &DesiredDirection,
            // &KccVelocity,
            &mut LinearVelocity,
            // &mut Transform,
        ),
        // With<KinematicCharacterController>,
        With<Player>,
    >,
    mut gizmos: Gizmos,
    time: Res<Time>,
    obstacles: Query<Entity, With<Obstacle>>,
) {
    for (
        entity,
        collider,
        position,
        // desired_direction,
        mut linear_velocity,
        // transform,
    ) in &mut characters
    {
        let max_hits = 5;
        let mut origin: Vec3 = position.0;
        let Ok(mut direction) =
            Dir3::new(linear_velocity.0)
        else {
            continue;
        };

        let colors = [
            RED_400, ORANGE_400, YELLOW_400, GREEN_400,
            BLUE_400, INDIGO_400,
        ];
        // print!("\ntry loop: ");

        let mut slide_accumulation = Vec3::ZERO;
        let mut new_direction = Vec3::ZERO;

        let mut current_velocity_magnitude =
            linear_velocity.0.length()
                * time.delta_secs()
                * 1.;

        // info!(?current_velocity_magnitude);

        'inner: for (i, color) in
            colors.iter().enumerate().take(max_hits)
        {
            // info!("{i}");
            let mut excluded_entities =
                obstacles.iter().collect::<Vec<Entity>>();
            excluded_entities.push(entity);
            // Cast shape and print first hit
            let Some(first_hit) = spatial_query.cast_shape(
                collider,
                origin,
                Quat::default(),
                direction,
                &ShapeCastConfig::from_max_distance(
                    current_velocity_magnitude,
                ),
                &SpatialQueryFilter::default()
                    .with_excluded_entities(
                        excluded_entities,
                    ),
            ) else {
                if i == 0 {
                    slide_accumulation = linear_velocity.0
                        * time.delta_secs();
                } else {
                    // no hit
                    slide_accumulation += new_direction;
                }
                continue 'inner;
            };

            // println!("First hit: {:?}", first_hit);
            let first_hit_gizmo_position =
                origin + first_hit.distance * direction;

            gizmos.arrow(
                first_hit_gizmo_position,
                first_hit_gizmo_position
                    + first_hit.normal1,
                RED_400,
            );
            gizmos.primitive_3d(
                &Capsule3d::new(0.5, 1.),
                Isometry3d::from_translation(
                    first_hit_gizmo_position,
                ),
                *color,
            );

            slide_accumulation +=
                first_hit.distance * direction;
            // info!(?slide_accumulation, "add");

            gizmos.arrow(
                origin,
                first_hit_gizmo_position,
                *color,
            );
            // let used_velocity_percent =
            // first_hit.distance
            //     - current_velocity_magnitude;

            let up_down = ((first_hit_gizmo_position
                - origin)
                .normalize()
                * current_velocity_magnitude)
                .cross(first_hit.normal1);
            // "up_down"
            gizmos.arrow(
                first_hit_gizmo_position,
                first_hit_gizmo_position + up_down * 10.,
                GREEN_400,
            );
            new_direction =
                first_hit.normal1.cross(up_down);

            // gizmos.arrow(
            //     origin,
            //     first_hit_gizmo_position,
            //     *color,
            // );

            gizmos.arrow(
                first_hit_gizmo_position,
                first_hit_gizmo_position + new_direction,
                Color::BLACK,
            );

            // this is the leftover velocity magnitude
            current_velocity_magnitude =
                new_direction.length();

            let Ok(new_dir) = Dir3::try_from(new_direction)
            else {
                continue 'inner;
            };
            slide_accumulation += first_hit.normal1 / 20.;
            origin = first_hit_gizmo_position
                + first_hit.normal1 / 60.;
            direction = new_dir;
        }

        gizmos.arrow(
            position.0,
            position.0 + slide_accumulation,
            Color::WHITE,
        );

        // info!(?slide_accumulation);
        // transform.translation +=
        //     slide_accumulation * time.delta_secs() *
        // 30.; **linear_velocity +=
        // slide_accumulation;

        let Ok(slide_dir) = Dir3::new(slide_accumulation)
        else {
            // invalid slide_dir means we're going to fall
            // through the map
            info!("invalid slide_dir");
            continue;
        };
        **linear_velocity =
            linear_velocity.length() * slide_dir;
    }
}

fn update_speed_text(
    player: Single<&LinearVelocity, With<Player>>,
    mut speed_texts: Query<&mut TextSpan, With<SpeedText>>,
) {
    for mut text in &mut speed_texts {
        text.0 = (player.length() as u32).to_string();
    }
}

// fn shape_cast_grounded(mut shape_cast_grounded:
// ResMut<ShapeCastGrounded>) {

// }
