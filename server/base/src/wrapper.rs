use core::fmt;
use std::ops::{Deref, DerefMut};

pub struct JobSchedulerWrapper(tokio_cron_scheduler::JobScheduler);

impl JobSchedulerWrapper {
    pub fn new(internal: tokio_cron_scheduler::JobScheduler) -> Self {
        Self(internal)
    }
}

impl fmt::Debug for JobSchedulerWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JobScheduler's data cannot be printed.")
            .finish()
    }
}

impl Deref for JobSchedulerWrapper {
    type Target = tokio_cron_scheduler::JobScheduler;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JobSchedulerWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
