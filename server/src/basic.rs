use dashmap::DashMap;
use parking_lot::Mutex;
use rand::Rng;
use std::sync::Arc;
use tokio::{
    sync::{Barrier, oneshot},
    task::JoinSet,
};

pub type ShutdownFinishCallback = oneshot::Sender<()>;
pub type TaskId = u64;
pub type Tasks = Arc<DashMap<TaskId, Task>>;

#[derive(Debug)]
pub struct Task {
    name: String,
    description: String,
    shutdown_handle: Option<ShutdownSdrImpl>,
}

#[derive(Debug, Clone)]
pub struct ShutdownSdr {
    callback: Arc<Mutex<Option<ShutdownFinishCallback>>>,
    tasks: Tasks,
}

impl ShutdownSdr {
    pub fn new(callback: Option<ShutdownFinishCallback>) -> Self {
        Self {
            callback: Arc::new(Mutex::new(callback)),
            tasks: Arc::new(DashMap::new()),
        }
    }

    pub fn new_receiver(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> ShutdownRev {
        let mut task_id;
        loop {
            task_id = rand::thread_rng().r#gen();
            if !self.tasks.contains_key(&task_id) {
                break;
            }
        }
        tracing::trace!("task {} created", task_id);
        let (chann_sdr, chann_rev) = oneshot::channel();
        let task = Task {
            name: name.into(),
            description: description.into(),
            shutdown_handle: Some(chann_sdr),
        };
        self.tasks.insert(task_id, task);
        ShutdownRev::new(chann_rev, self.tasks.clone(), task_id)
    }

    pub async fn shutdown_all_tasks(&mut self) -> anyhow::Result<()> {
        let mut waiting_finished = JoinSet::new();
        let cnt = Arc::new(Barrier::new(self.tasks.len() + 1));
        let mut sent_opers = vec![];
        let wrong_check = Arc::new(Mutex::new(0));
        for mut task in self.tasks.iter_mut() {
            tracing::trace!("task {} shutdown", task.key());
            let (wait_stop_sdr, wait_stop_rev) = oneshot::channel::<()>();
            let tasks_ref = self.tasks.clone();
            let task_id = *task.key();
            let cnt_clone = cnt.clone();
            let wrong_check1 = wrong_check.clone();
            waiting_finished.spawn(async move {
                cnt_clone.wait().await;
                match wait_stop_rev.await {
                    Ok(()) => {}
                    Err(_) => {
                        *wrong_check1.lock() += 1;
                    }
                }
                tasks_ref.remove(&task_id);
            });
            let handle = task.value_mut().shutdown_handle.take().unwrap();
            let name = task.value().name.clone();
            let description = task.value().description.clone();
            let wrong_check2 = wrong_check.clone();
            sent_opers.push(move || {
                if let Err(e) = handle.send(wait_stop_sdr) {
                    tracing::error!(
                        "task {} send shutdown error: {:?} name: {}, description: {}",
                        task_id,
                        e,
                        name,
                        description
                    );
                    *wrong_check2.lock() -= 1;
                }
            });
        }
        if *wrong_check.lock() != 0 {
            tracing::error!("some tasks shutdown failed");
        }
        tracing::trace!("total tasks: {}", self.tasks.len());
        cnt.wait().await;
        tracing::trace!("tasks are ready to fetch finish signal");
        for oper in sent_opers {
            oper();
        }
        waiting_finished.join_all().await;
        if let Some(callback) = self.callback.lock().take() {
            callback.send(()).unwrap();
        }
        self.log_all_task();
        Ok(())
    }

    pub fn log_all_task(&self) {
        for task in self.tasks.iter() {
            tracing::info!(
                "task {} name: {}, description: {}",
                task.key(),
                task.value().name,
                task.value().description
            );
        }
    }

    pub fn gen_task_info(tasks: &Tasks, task_id: TaskId) -> anyhow::Result<String> {
        let task = tasks.get(&task_id).unwrap();
        Ok(format!(
            "task {} name: {}, description: {}",
            task_id, task.name, task.description
        ))
    }
}

pub struct ShutdownRev {
    receiver: Option<ShutdownRevImpl>,
    manager_handle: Arc<DashMap<TaskId, Task>>,
    task_id: TaskId,
}

impl ShutdownRev {
    pub async fn wait_shutdowning(&mut self) {
        let gen_err_info = || ShutdownSdr::gen_task_info(&self.manager_handle, self.task_id);
        match &mut self.receiver {
            Some(receiver) => {
                if let Err(e) = receiver.await.unwrap().send(()) {
                    tracing::error!(
                        "receive shutdown error: {:?};details:{:?}",
                        e,
                        gen_err_info()
                    );
                }
                self.remove_self();
            }
            None => {
                tracing::error!("without shutdown signal sender;task:{:?}", gen_err_info());
            }
        }
    }

    pub fn new(
        receiver: ShutdownRevImpl,
        manager_handle: Arc<DashMap<TaskId, Task>>,
        task_id: TaskId,
    ) -> Self {
        Self {
            receiver: Some(receiver),
            manager_handle,
            task_id,
        }
    }

    fn remove_self(&self) {
        tracing::trace!("task {} removed", self.task_id);
        self.manager_handle.remove(&self.task_id);
    }
}

impl Drop for ShutdownRev {
    fn drop(&mut self) {
        if self.receiver.is_some() {
            self.remove_self();
        }
    }
}

pub type ShutdownRevImpl = oneshot::Receiver<oneshot::Sender<()>>;
pub type ShutdownSdrImpl = oneshot::Sender<oneshot::Sender<()>>;
