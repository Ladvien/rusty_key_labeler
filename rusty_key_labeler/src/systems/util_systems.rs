use crate::DebounceTimer;
use bevy::prelude::*;

pub fn debounce_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DebounceTimer), With<DebounceTimer>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
        commands.entity(entity).remove::<DebounceTimer>();
    }
}
