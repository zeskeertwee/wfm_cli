use std::any::Any;
use std::future::Future;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;
use tokio::runtime::Runtime;

use crate::app::{App, AppEvent};

pub trait Job: Any + Send + Sync + Unpin + 'static {
    fn run(&mut self, rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()>;

    /// use this as a guard against having multiple of the same job pending
    /// if this returns Err(), the job will not be submitted and run() will never be called
    /// returning an error here does not return an error from the submit_job() function
    fn on_submit(&mut self, _app: &App) -> anyhow::Result<()> {
        Ok(())
    }
    fn job_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T> Job for T
where
    T: Future<Output = anyhow::Result<()>> + Send + Sync + Unpin + 'static,
{
    fn run(&mut self, rt: &Runtime, _tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        rt.block_on(self)
    }
}

pub struct WorkerPool {
    handles: Vec<JoinHandle<()>>,
    tx: Sender<Box<dyn Job>>,
    rx: Receiver<Box<dyn Job>>,
    app_tx: Sender<AppEvent>,
}

impl WorkerPool {
    pub fn new(app_tx: Sender<AppEvent>) -> Self {
        let (tx, rx) = unbounded();
        let handles = Vec::new();
        Self {
            handles,
            tx,
            rx,
            app_tx,
        }
    }

    pub fn spawn_worker(&mut self) -> anyhow::Result<()> {
        let rx_clone = self.rx.clone();
        let app_tx_clone = self.app_tx.clone();

        let handle = thread::Builder::new()
            .name(format!("worker-{}", self.handles.len() + 1))
            .spawn(move || {
                worker_main(rx_clone, app_tx_clone);
            })?;

        self.handles.push(handle);
        Ok(())
    }

    pub fn sumbit_job(&self, job: Box<dyn Job>) -> anyhow::Result<()> {
        self.tx.send(job)?;
        Ok(())
    }
}

fn worker_main(receiver: Receiver<Box<dyn Job>>, tx: Sender<AppEvent>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();

    loop {
        match receiver.recv() {
            Ok(mut job) => {
                let name = job.job_name();
                match job.run(&rt, &tx) {
                    Ok(_) => info!("Job {} completed", name),
                    Err(e) => info!("Job {} failed: {}", name, e),
                }
            }
            Err(e) => {
                info!("Channel closed: {}", e);
                return;
            }
        }
    }
}

pub enum JobState<T> {
    Idle,
    Pending,
    Done(anyhow::Result<T>),
}

impl<T> PartialEq for JobState<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pending, Self::Pending) => true,
            (Self::Idle, Self::Idle) => true,
            (Self::Done(_), Self::Done(_)) => true,
            _ => false,
        }
    }
}

impl<T> Default for JobState<T> {
    fn default() -> Self {
        Self::Idle
    }
}

pub fn send_over_tx(tx: &Sender<AppEvent>, event: AppEvent) -> anyhow::Result<()> {
    if let Err(e) = tx.send(event) {
        anyhow::bail!("Error sending event: {}", e);
    }

    Ok(())
}
