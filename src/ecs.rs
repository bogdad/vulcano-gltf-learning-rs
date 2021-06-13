
use bevy_ecs::change_detection::Mut;
use bevy_ecs::event::Events;
use bevy_ecs::world::World;
use bevy_ecs::schedule::{Schedule, SystemStage, Stage};
use bevy_ecs::system::IntoSystem;
use bevy_ecs::component::Component;

use crate::components::CameraEnteredEvent;
use crate::input::{MyKeyboardInput, MyMouseInput};
use crate::systems::{camera_reacts_to_keyboard, camera_reacts_to_mouse_movement, movement};

pub struct Ecs {
  pub world: World,
  schedule: Schedule,
}

pub struct EcsEvents<'a> {
  pub events_keyboard: &'a Events<MyKeyboardInput>,
  pub events_mouse: &'a Events<MyMouseInput>,
  pub events_camera: &'a Events<CameraEnteredEvent>,
}

impl Ecs {
  pub fn new() -> Self {
    let mut world = World::default();

    world.insert_resource(Events::<MyMouseInput>::default());
    world.insert_resource(Events::<MyKeyboardInput>::default());
    world.insert_resource(Events::<CameraEnteredEvent>::default());

    let mut schedule = Schedule::default();
    schedule.add_stage("update", SystemStage::parallel()
        .with_system(Events::<MyMouseInput>::update_system.system())
        .with_system(Events::<MyKeyboardInput>::update_system.system())
        .with_system(Events::<CameraEnteredEvent>::update_system.system())
        .with_system(movement.system())
        .with_system(camera_reacts_to_keyboard.system())
        .with_system(camera_reacts_to_mouse_movement.system())
    );

    let ecs = Ecs {
      world,
      schedule,
    };
    ecs
  }

  /*pub fn get_events<'a>(&'a self) -> EcsEvents {
    let events_camera = ;
    let events_keyboard = self.world.get_resource::<Events<MyKeyboardInput>>().unwrap();
    let events_mouse = self.world.get_resource::<Events<MyMouseInput>>().unwrap();

    let ecs_events = EcsEvents {
      events_keyboard,
      events_mouse,
      events_camera,
    };
    ecs_events
  }*/

  pub fn get_events<T: Component>(&self) -> &Events::<T> {
    return self.world.get_resource::<Events<T>>().unwrap();
  }

  pub fn get_events_mut<T: Component>(&mut self) -> Mut<Events::<T>> {
    return self.world.get_resource_mut::<Events<T>>().unwrap();
  }

  pub fn gr<T: Component>(world: &World) -> &Events<T> {
    world.get_resource::<Events<T>>().unwrap()
  }

  pub fn tick(&mut self) {
    self.schedule.run(&mut self.world);
  }
}