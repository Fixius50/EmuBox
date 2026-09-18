use crate::{
    errors::EmuBoxError,
    models::{Emulator, Game, HardwareInfo, Platform, SystemSettings},
    services::GameService,
};
mod report;
mod tasks;
use report::Task;
pub use report::{Phase, Report, Resources};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};
use tasks::{execute, Execute, Output};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupData {
    pub settings: SystemSettings,
    pub platforms: Vec<Platform>,
    pub games: Vec<Game>,
    pub hardware: HardwareInfo,
    pub emulators: Vec<Emulator>,
    pub report: Report,
}

#[derive(Default)]
struct Data {
    settings: Option<SystemSettings>,
    platforms: Vec<Platform>,
    games: Option<Vec<Game>>,
    hardware: Option<HardwareInfo>,
    emulators: Vec<Emulator>,
}

struct Inner {
    report: Report,
    data: Data,
    frontend_claimed: bool,
}

#[derive(Clone)]
pub struct Startup {
    inner: Arc<(Mutex<Inner>, Condvar)>,
    started: Instant,
    launched: Arc<AtomicBool>,
}

type Notify = Arc<dyn Fn(Report) + Send + Sync>;

impl Startup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(Inner {
                    report: Report::new(Resources::detect()),
                    data: Data::default(),
                    frontend_claimed: false,
                }),
                Condvar::new(),
            )),
            started: Instant::now(),
            launched: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn report(&self) -> Report {
        let mut report = self
            .inner
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .report
            .clone();
        if matches!(report.phase, Phase::Preparing | Phase::Prepared) {
            report.elapsed_ms = self.started.elapsed().as_millis() as u64;
        }
        report
    }
    fn publish(&self, notify: &Notify) {
        notify(self.report());
    }
    fn fail(&self, message: String, notify: &Notify) {
        let mut inner = self
            .inner
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        inner.report.phase = Phase::Error;
        let running = inner.report.running_task_names();
        inner.report.error = Some(if running.is_empty() {
            message
        } else {
            format!("{message}. Tareas pendientes: {}", running.join(", "))
        });
        inner.report.cancel_pending_and_error_running();
        inner.report.elapsed_ms = self.started.elapsed().as_millis() as u64;
        drop(inner);
        self.inner.1.notify_all();
        self.publish(notify);
    }
    pub fn start(&self, notify: Notify, post_ready: impl FnOnce() + Send + 'static) {
        if self.launched.swap(true, Ordering::AcqRel) {
            return;
        }
        let startup = self.clone();
        std::thread::spawn(move || {
            startup.prepare(&notify);
            let inner = startup
                .inner
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let (inner, _) = startup
                .inner
                .1
                .wait_timeout_while(inner, Duration::from_secs(120), |inner| {
                    inner.report.phase == Phase::Prepared
                })
                .unwrap_or_else(|error| error.into_inner());
            let ready = matches!(inner.report.phase, Phase::Ready | Phase::Degraded);
            let abandoned = inner.report.phase == Phase::Prepared;
            drop(inner);
            if ready {
                post_ready();
            } else if abandoned {
                startup.fail(
                    "La interfaz no confirmo su preparacion en 120 segundos".into(),
                    &notify,
                );
            }
        });
    }
    fn prepare(&self, notify: &Notify) {
        self.prepare_with(notify, Duration::from_secs(60), Arc::new(execute));
    }
    fn prepare_with(&self, notify: &Notify, timeout: Duration, execute_task: Execute) {
        let (sender, receiver) = mpsc::channel();
        loop {
            if self.started.elapsed() >= timeout {
                self.fail(
                    "Preparacion nativa agotada; no se iniciaran tareas adicionales".into(),
                    notify,
                );
                return;
            }
            loop {
                let mut inner = self
                    .inner
                    .0
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                let Some(task) = inner.report.start_next_task() else {
                    break;
                };
                let hardware = inner.data.hardware.clone();
                drop(inner);
                self.publish(notify);
                let sender = sender.clone();
                let execute_task = execute_task.clone();
                std::thread::spawn(move || {
                    let started = Instant::now();
                    let _span = crate::services::infrastructure::telemetry::span(
                        "startup.task",
                        task.telemetry_name(),
                    );
                    eprintln!("[Startup] task={task:?} running");
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_task(task, hardware)
                    }))
                    .unwrap_or_else(|_| {
                        Err(EmuBoxError::ProcessFailed(
                            "Tarea de arranque interrumpida".into(),
                        ))
                    });
                    let elapsed = started.elapsed().as_millis() as u64;
                    eprintln!(
                        "[Startup] task={task:?} finished elapsedMs={elapsed} success={}",
                        result.is_ok()
                    );
                    let _ = sender.send((task, elapsed, result));
                });
            }
            let remaining = timeout.saturating_sub(self.started.elapsed());
            let (task, elapsed, result) = match receiver.recv_timeout(remaining) {
                Ok(result) => result,
                Err(_) => {
                    self.fail("Preparacion nativa agotada (60 segundos); no se iniciaran tareas adicionales".into(), notify);
                    return;
                }
            };
            if self.started.elapsed() >= timeout {
                self.fail(
                    "La tarea termino fuera del plazo de preparacion".into(),
                    notify,
                );
                return;
            }
            let (output, warnings) = match result {
                Ok(result) => result,
                Err(error) => {
                    self.fail(format!("{task:?}: {error}"), notify);
                    return;
                }
            };
            let mut inner = self
                .inner
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let degraded = !warnings.is_empty();
            inner.report.finish_task(task, elapsed, degraded);
            inner.report.warnings.extend(warnings);
            match output {
                Output::Library(settings, platforms, games) => {
                    inner.data.settings = Some(settings);
                    inner.data.platforms = platforms;
                    inner.data.games = Some(games);
                    inner.report.resources.storage_available = Some(true);
                }
                Output::Hardware(hardware) => {
                    inner.report.resources.graphics_state =
                        Some(format!("{:?}", hardware.graphics.detection_state).to_lowercase());
                    inner.report.resources.graphics_backend =
                        Some(hardware.graphics.operational_backend.clone());
                    inner.report.resources.virtual_machine = Some(hardware.is_virtual_machine);
                    inner.data.hardware = Some(hardware);
                }
                Output::Emulators(emulators) => inner.data.emulators = emulators,
                Output::Services => (),
            }
            let finished = inner.report.all_tasks_finished();
            if finished {
                inner.report.phase = Phase::Prepared;
            }
            inner.report.elapsed_ms = self.started.elapsed().as_millis() as u64;
            drop(inner);
            self.publish(notify);
            if finished {
                self.inner.1.notify_all();
                return;
            }
        }
    }
    pub fn data(&self) -> Result<StartupData, EmuBoxError> {
        let mut inner = self
            .inner
            .0
            .lock()
            .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
        if !matches!(
            inner.report.phase,
            Phase::Prepared | Phase::Ready | Phase::Degraded
        ) {
            return Err(EmuBoxError::ProcessFailed(
                inner
                    .report
                    .error
                    .clone()
                    .unwrap_or_else(|| "Preparacion en curso".into()),
            ));
        }
        let settings = inner.data.settings.clone().ok_or_else(|| {
            EmuBoxError::InvalidConfiguration("Configuracion no preparada".into())
        })?;
        let hardware = inner
            .data
            .hardware
            .clone()
            .ok_or_else(|| EmuBoxError::HardwareUnavailable("Hardware no preparado".into()))?;
        let games = inner.data.games.take();
        let platforms = inner.data.platforms.clone();
        let emulators = inner.data.emulators.clone();
        let report = inner.report.clone();
        inner.frontend_claimed = true;
        drop(inner);
        Ok(StartupData {
            settings,
            hardware,
            platforms,
            emulators,
            report,
            games: match games {
                Some(games) => games,
                None => GameService::get_games(None)?,
            },
        })
    }
    pub fn frontend_ready(&self) -> Result<Report, EmuBoxError> {
        let mut inner = self
            .inner
            .0
            .lock()
            .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
        if matches!(inner.report.phase, Phase::Ready | Phase::Degraded) {
            return Ok(inner.report.clone());
        }
        if inner.report.phase != Phase::Prepared || !inner.frontend_claimed {
            return Err(EmuBoxError::ProcessFailed(
                "La preparacion de la interfaz no ha terminado".into(),
            ));
        }
        inner.report.phase = if inner.report.warnings.is_empty() {
            Phase::Ready
        } else {
            Phase::Degraded
        };
        inner.report.elapsed_ms = self.started.elapsed().as_millis() as u64;
        let report = inner.report.clone();
        drop(inner);
        self.inner.1.notify_all();
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_cannot_acknowledge_unprepared_or_failed_startup() {
        let startup = Startup::new();
        assert!(startup.frontend_ready().is_err());
        let notify: Notify = Arc::new(|_| {});
        startup.fail("test failure".into(), &notify);
        assert_eq!(startup.report().phase, Phase::Error);
        assert!(startup.data().is_err());
        assert!(startup.frontend_ready().is_err());
    }
    #[test]
    fn readiness_requires_payload_and_is_idempotent() {
        for degraded in [false, true] {
            let startup = Startup::new();
            {
                let mut inner = startup.inner.0.lock().unwrap();
                inner.report.phase = Phase::Prepared;
                if degraded {
                    inner
                        .report
                        .warnings
                        .push("optional service unavailable".into());
                }
            }
            assert!(startup.frontend_ready().is_err());
            startup.inner.0.lock().unwrap().frontend_claimed = true;
            let expected = if degraded {
                Phase::Degraded
            } else {
                Phase::Ready
            };
            assert_eq!(startup.frontend_ready().unwrap().phase, expected);
            assert_eq!(startup.frontend_ready().unwrap().phase, expected);
        }
    }
    #[test]
    fn coordinator_stops_on_critical_failure_and_publishes_degraded_results() {
        let notify: Notify = Arc::new(|_| {});
        let startup = Startup::new();
        startup
            .inner
            .0
            .lock()
            .unwrap()
            .report
            .resources
            .max_concurrent_tasks = 1;
        startup.prepare_with(
            &notify,
            Duration::from_secs(2),
            Arc::new(|task, _| {
                assert_eq!(task, Task::Library);
                Err(EmuBoxError::StorageUnavailable("fixture".into()))
            }),
        );
        assert_eq!(startup.report().phase, Phase::Error);
        assert!(startup
            .report()
            .states_after(1)
            .iter()
            .all(|state| *state == "cancelled"));

        let startup = Startup::new();
        startup.prepare_with(
            &notify,
            Duration::from_secs(2),
            Arc::new(|task, _| {
                Ok((
                    Output::Services,
                    if task == Task::Services {
                        vec!["missing optional service".into()]
                    } else {
                        vec![]
                    },
                ))
            }),
        );
        assert_eq!(startup.report().phase, Phase::Prepared);
        assert_eq!(startup.report().warnings.len(), 1);
    }
    #[test]
    fn timeout_does_not_accept_late_results_or_schedule_replacements() {
        let startup = Startup::new();
        startup
            .inner
            .0
            .lock()
            .unwrap()
            .report
            .resources
            .max_concurrent_tasks = 1;
        let notify: Notify = Arc::new(|_| {});
        let (release, pending) = mpsc::channel::<()>();
        let pending = Mutex::new(pending);
        let (finished, completion) = mpsc::channel();
        startup.prepare_with(
            &notify,
            Duration::from_millis(20),
            Arc::new(move |task, _| {
                assert_eq!(task, Task::Library);
                pending.lock().unwrap().recv().unwrap();
                finished.send(()).unwrap();
                Ok((Output::Services, vec![]))
            }),
        );
        assert_eq!(startup.report().phase, Phase::Error);
        assert!(startup
            .report()
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Library"));
        release.send(()).unwrap();
        completion.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(startup.report().phase, Phase::Error);
        assert!(startup
            .report()
            .states_after(1)
            .iter()
            .all(|state| *state == "cancelled"));
    }
    #[test]
    fn independent_tasks_overlap_without_exceeding_two_slots() {
        use std::sync::atomic::AtomicUsize;
        let startup = Startup::new();
        startup
            .inner
            .0
            .lock()
            .unwrap()
            .report
            .resources
            .max_concurrent_tasks = 2;
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let worker_active = active.clone();
        let worker_peak = peak.clone();
        let (hardware_started, hardware_wait) = mpsc::sync_channel(0);
        let hardware_wait = Mutex::new(hardware_wait);
        let notify: Notify = Arc::new(|_| {});
        startup.prepare_with(
            &notify,
            Duration::from_secs(5),
            Arc::new(move |task, _| {
                let count = worker_active.fetch_add(1, Ordering::SeqCst) + 1;
                worker_peak.fetch_max(count, Ordering::SeqCst);
                if task == Task::Hardware {
                    hardware_started.send(()).unwrap();
                }
                if task == Task::Library {
                    hardware_wait
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(2))
                        .unwrap();
                }
                worker_active.fetch_sub(1, Ordering::SeqCst);
                Ok((Output::Services, vec![]))
            }),
        );
        assert_eq!(startup.report().phase, Phase::Prepared);
        assert_eq!(peak.load(Ordering::SeqCst), 2);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }
}
