use avian3d::prelude::*;
use bevy::{
    asset::{AssetMetaCheck, UntypedAssetId},
    core_pipeline::bloom::{
        Bloom, BloomCompositeMode, BloomPrefilter,
    },
    diagnostic::{
        DiagnosticsStore, FrameTimeDiagnosticsPlugin,
    },
    ecs::spawn::SpawnWith,
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy::{
    color::palettes::tailwind::*,
    ecs::error::{GLOBAL_ERROR_HANDLER, warn},
};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_seedling::prelude::*;
use bevy_skein::SkeinPlugin;
use scaffolding::*;
use vleue_kinetoscope::{
    AnimatedImageController, AnimatedImagePlugin,
};

fn main() {
    GLOBAL_ERROR_HANDLER.set(warn).expect(
        "The error handler can only be set once, globally.",
    );

    let mut app = App::new();

    app.insert_resource(ClearColor(SLATE_950.into()))
        // .insert_resource(DefaultFriction(Friction::new(0.)))
        .add_plugins((
            DefaultPlugins
                // .set(AssetPlugin {
                //     // Wasm builds will check for meta files (that don't exist) if this isn't set.
                //     // This causes errors and even panics on web build on itch.
                //     // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                //     meta_check: AssetMetaCheck::Never,
                //     ..default()
                // })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Scaffolding".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            SkeinPlugin::default(),
            AnimatedImagePlugin,
            SeedlingPlugin::default(),
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
            EnhancedInputPlugin,
        ))
        .add_plugins((
            dev::DevToolsPlugin,
            assets::AppAssetsPlugin,
            playing::PlayingPlugin,
            movement::MovementPlugin,
            terrain_chunking::LandChunkPlugin,
        ))
        .init_state::<AppState>()
        .add_systems(
            OnEnter(AppState::AssetLoading),
            render_description,
        )
        .add_systems(OnEnter(AppState::Next), expect)
        .add_systems(
            OnEnter(AppState::MainMenu),
            spawn_main_menu,
        )
        .add_observer(update_button_color_on::<Over>(
            SLATE_400.into(),
        ))
        .add_observer(update_button_color_on::<Out>(
            SLATE_50.into(),
        ))
        .add_observer(update_button_color_on::<Pressed>(
            GREEN_400.into(),
        ))
        .add_observer(update_button_color_on::<Released>(
            SLATE_50.into(),
        ))
        .run();
}

fn update_button_color_on<E>(
    color: Color,
) -> impl Fn(
    Trigger<Pointer<E>>,
    Query<(), (With<Button>, With<MainMenuButton>)>,
    Query<&mut TextColor>,
    Query<&Children>,
)
where
    E: std::fmt::Debug + Reflect + Clone,
{
    // An observer closure that accepts a Pointer Event and a Color. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // Event/Color. Instead, the event type is a generic, and the Event/Color is passed in.
    move |trigger: Trigger<Pointer<E>>,
          buttons: Query<
        (),
        (With<Button>, With<MainMenuButton>),
    >,
          mut text_colors: Query<&mut TextColor>,
          children: Query<&Children>| {
        let Ok(_) = buttons.get(trigger.target()) else {
            return;
        };

        for entity in
            children.iter_descendants(trigger.target())
        {
            let Ok(mut text_color) =
                text_colors.get_mut(entity)
            else {
                continue;
            };
            text_color.0 = color;
        }
    }
}
fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // commands.spawn(AnimatedImageController::play(
    //     asset_server.load("video/bevy.webp"),
    // ));
    commands.spawn((
        playing::PlayerFollowCamera,
        TransformInterpolation,
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Bloom {
            intensity: 0.05,
            low_frequency_boost: 0.7,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 0.0,
                threshold_softness: 0.0,
            },
            composite_mode:
                BloomCompositeMode::EnergyConserving,
            max_mip_dimension: 512,
            scale: Vec2::ONE,
        },
        Transform::from_xyz(0., 3., 4.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        DistanceFog {
            // color: Color::srgb(0.25, 0.0, 0.25),
            color: SLATE_950.into(),
            falloff: FogFalloff::Linear {
                start: 100.0,
                end: 300.0,
            },
            ..default()
        },
        // Projection::Orthographic(OrthographicProjection {
        //     scaling_mode:
        //         bevy::render::camera::ScalingMode::AutoMin {
        //             min_width: 1920.,
        //             min_height: 1080.,
        //         },
        //     ..OrthographicProjection::default_2d()
        // }),
    ));
    // commands.spawn((
    //     playing::PlayerFollowCameraDebug,
    //     Camera3d::default(),
    //     Transform::from_xyz(-10., 0., 0.)
    //         .looking_at(Vec3::ZERO, Vec3::Y),
    //     Projection::Orthographic(OrthographicProjection {
    //         scale: 0.01,
    //         ..OrthographicProjection::default_3d()
    //     }),
    // ));
    let font = asset_server.load(
        "fonts/Alfa_Slab_One/AlfaSlabOne-Regular.ttf",
    );
    let font_one = font.clone();
    let font_two = font.clone();
    let font_three = font.clone();
    commands.spawn((
        StateScoped(AppState::MainMenu),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        children![(
            Node {
                width: Val::Px(250.),
                height: Val::Auto,
                margin: UiRect::all(Val::Px(125.)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                ..default()
            },
            Children::spawn((
                SpawnWith(|parent: &mut ChildSpawner| {
                    parent
                        .spawn(main_menu_text_button(
                            "Start", font_one,
                        ))
                        .observe(
                            |_trigger: Trigger<
                                Pointer<Click>,
                            >,
                             mut next_state: ResMut<
                                NextState<AppState>,
                            >| {
                                next_state
                                    .set(AppState::Playing);
                            },
                        );
                }),
                SpawnWith(|parent: &mut ChildSpawner| {
                    parent.spawn(main_menu_text_button(
                        "Options", font_two,
                    ));
                }),
                SpawnWith(|parent: &mut ChildSpawner| {
                    parent
                        .spawn(main_menu_text_button(
                            "Quit", font_three,
                        ))
                        .observe(
                            |_trigger: Trigger<
                                Pointer<Click>,
                            >,
                             mut app_exit: EventWriter<
                                AppExit,
                            >| {
                                app_exit.write(
                                    AppExit::Success,
                                );
                            },
                        );
                })
            ))
        )],
    ));
}

#[derive(Component)]
struct MainMenuButton;

fn main_menu_text_button(
    text: &str,
    font: Handle<Font>,
) -> impl Bundle {
    (
        Name::new(format!("{text} Button")),
        Button,
        MainMenuButton,
        Node {
            padding: UiRect::axes(
                Val::Px(15.),
                Val::Px(5.),
            ),
            ..default()
        },
        children![(
            Text::new(text),
            TextFont {
                font: font,
                ..default()
            }
        )],
    )
}
// fn startup(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
// ) {
//     commands.spawn(SceneRoot(
//         asset_server.load(
//             // Change this to your exported gltf file
//             GltfAssetLabel::Scene(0)
//                 .from_asset("gltf/levels.glb"),
//         ),
//     ));
// }

fn render_description(mut commands: Commands) {
    // commands.spawn(Camera2d);
}

fn expect() {}
