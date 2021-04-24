// use std::sync::Arc;
// use std::sync::atomic::AtomicBool;
// pub struct TaskGraph {
//     tasks: Vec<Task>,
// }

// pub struct TaskHandle {
//     completed: AtomicBool,
// }
// pub struct Task {
//     depends: Vec<TaskHandle>,
//     func: Arc<dyn FnOnce()->()>,
// }

// impl TaskGraph {
//     pub fn submit(&mut self, task: Task)->TaskHandle{
//         self.tasks.push(task);
//     }
//     pub fn execute(&mut self){

//     }
// }

// impl Task {
//     pub fn new<F: FnOnce()->() + 'static>(f: F)->Self{
//         Task {
//             depends:vec![],
//             func:Arc::new(f) as Arc<dyn FnOnce()->()>,
//         }
//     }
//     pub fn depends(&mut self, dependency: TaskHandle){
//         self.depends.push(dependency);
//     }
// }