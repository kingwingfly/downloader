mod error;
mod task_bmc; // Task Backend Model Controller for task

use crate::task::Task;
use std::rc::Rc;

pub struct Model {
    pub tasks: Vec<Rc<Task>>,
}

impl Model {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
}
