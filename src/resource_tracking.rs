use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;

pub fn plugin(app: &mut App) {
    app.insert_resource(Callbacks::default())
        .add_systems(PreUpdate, track_resources);
}

pub trait TrackableResource: Resource {
    fn get_handles_to_track(&self) -> Vec<UntypedHandle>;

    fn on_tracked_handles_fully_loaded(&self) -> impl Command;
}

pub trait ResourceTracking<T: TrackableResource> {
    fn insert_trackable_resource(&mut self, trackable_resource: T);
}

#[derive(Default, Deref, DerefMut, Resource)]
struct Callbacks(HashMap<TypeId, SystemId>);

impl<T: TrackableResource> ResourceTracking<T> for Commands<'_, '_> {
    fn insert_trackable_resource(&mut self, trackable_resource: T) {
        self.insert_resource(trackable_resource);
        self.queue(|world: &mut World| {
            let system_id = world.register_system(track_resource::<T>);
            let mut callbacks = world
                .get_resource_mut::<Callbacks>()
                .expect("resource should exist at this point");
            callbacks.insert(TypeId::of::<T>(), system_id);
        });
    }
}

fn track_resources(mut commands: Commands, callbacks: Res<Callbacks>) {
    callbacks
        .values()
        .for_each(|system_id| commands.run_system(system_id.clone()));
}

fn track_resource<T: TrackableResource>(
    mut commands: Commands,
    trackable_resource: Res<T>,
    asset_server: Res<AssetServer>,
) {
    if trackable_resource
        .get_handles_to_track()
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle))
    {
        commands.queue(|world: &mut World| {
            let mut callbacks = world
                .get_resource_mut::<Callbacks>()
                .expect("resource should exist at this point");
            let system_id = callbacks.0.remove(&TypeId::of::<T>()).expect("system_id");
            world.unregister_system(system_id)
        });
        commands.queue(trackable_resource.on_tracked_handles_fully_loaded());
    }
}
