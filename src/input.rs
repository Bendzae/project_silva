use bevy::input::InputSystem;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InputCommand {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    ACTION,
}

pub struct InputEvent(pub InputCommand);

pub fn input_system(input: Res<Input<KeyCode>>, mut event: EventWriter<InputEvent>) {
    if input.pressed(KeyCode::Left) {
        event.send(InputEvent(InputCommand::LEFT));
    }
    if input.pressed(KeyCode::Right) {
        event.send(InputEvent(InputCommand::RIGHT));
    }
    if input.pressed(KeyCode::Up) {
        event.send(InputEvent(InputCommand::UP));
    }
    if input.pressed(KeyCode::Down) {
        event.send(InputEvent(InputCommand::DOWN));
    }
}

pub fn debug_input_system(time: Res<Time>, mut events: EventReader<InputEvent>) {
    for event in events.iter() {
        println!("Event: received: {:?} at {}", event.0, time.seconds_since_startup());
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>()
            .add_system(input_system.after(InputSystem));
            // .add_system(debug_input_system.after(input_system));
    }

    fn name(&self) -> &str {
        "InputPlugin"
    }
}
