mod error;
mod task_bmc; // Task Backend Model Controller for task

use std::sync::Arc;

use crate::task::TaskExe;

pub use task_bmc::TaskBmc;

type Task = Arc<dyn TaskExe + Send + Sync>;

pub struct Model {
    pub tasks: Vec<Task>,
}

impl Model {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
}
