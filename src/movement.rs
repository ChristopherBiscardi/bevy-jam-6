use std::{
    f32::consts::{FRAC_PI_4, PI},
    ops::Neg,
};

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::playing::Player;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Grounded>()
            .add_observer(bind_actions)
            .add_observer(apply_movement);
    }
}

#[derive(InputContext)]
pub struct Grounded;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

// gravity += Jerk * time
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct FastFall;

fn bind_actions(
    trigger: Trigger<Binding<Grounded>>,
    // settings: Res<AppSettings>,
    mut actions: Query<&mut Actions<Grounded>>,
) {
    let mut actions =
        actions.get_mut(trigger.target()).unwrap();

    actions
        .bind::<Move>()
        .to((
            Cardinal::wasd_keys(),
            Axial::left_stick(),
        ))
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
        ));

    actions
        .bind::<FastFall>()
        .to((KeyCode::Space, GamepadButton::South));
}

/// Apply movement when `Move` action considered
/// fired.
fn apply_movement(
    trigger: Trigger<Fired<Move>>,
    mut players: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
) {
    let current_velocity =
        players.get_mut(trigger.target()).unwrap().0;

    // info!(?current_velocity);

    let angle = trigger.event().value.x.signum().neg() * PI
        / 10.
        * time.delta_secs();
    let rotation_matrix = Mat3::from_cols(
        Vec3::new(angle.cos(), 0., (-angle).sin()),
        Vec3::new(0., 1., 0.),
        Vec3::new(angle.sin(), 0., angle.cos()),
    );

    for mut linvel in &mut players {
        let rotated = rotation_matrix * linvel.0;
        linvel.0 = rotated;
    }
}
