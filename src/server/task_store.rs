use bevy_defer::{AccessResult, Task};
use bevy_derive::{Deref, DerefMut};
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TaskType {
    Server,
    Connection(usize),
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub(crate) struct TaskStore(Arc<DashMap<TaskType, Task<AccessResult>>>);

impl TaskStore {
    pub(crate) fn insert(&self, task_type: TaskType, task: Task<AccessResult>) {
        self.cleanup_finished_tasks();
        self.0.insert(task_type, task);
    }

    pub(crate) fn finished_task_count(&self) -> usize {
        self.0
            .iter()
            .filter(|entry| entry.value().is_finished())
            .count()
    }

    pub(crate) fn cleanup_finished_tasks(&self) {
        self.0.retain(|_, task| !task.is_finished());
    }
}
