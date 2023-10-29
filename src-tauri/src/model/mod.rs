mod error;
mod task_bmc; // Task Backend Model Controller for task

use std::sync::Arc;

use crate::task::Task;

pub use task_bmc::TaskBmc;

pub struct Model {
    pub tasks: Vec<Arc<Task>>,
}

impl Model {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
}
