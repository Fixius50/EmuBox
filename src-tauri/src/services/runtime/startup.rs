use crate::{
    errors::EmuBoxError,
    models::{Emulator, Game, HardwareInfo, Platform, SystemSettings},
    services::{download_manager, EmulatorService, GameService, SystemService},
};
use serde::Serialize;
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Preparing,
    Prepared,
    Ready,
    Degraded,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Task {
    Library,
    Hardware,
    Services,
    Emulators,
}

impl Task {
    fn dependencies(self) -> &'static [Task] {
        match self {
            Self::Emulators => &[Self::Library, Self::Hardware],
            _ => &[],
        }
    }
    fn uses_io(self) -> bool {
        matches!(self, Self::Library | Self::Emulators)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    pub cpu_available: usize,
    pub memory_available_mb: Option<u64>,
    pub max_concurrent_tasks: usize,
    pub io_slots: usize,
    pub graphics_slots: usize,
    pub graphics_state: Option<String>,
    pub graphics_backend: Option<String>,
    pub virtual_machine: Option<bool>,
    pub storage_available: Option<bool>,
}

fn concurrency(cpu: usize, memory: Option<u64>) -> usize {
    if cpu >= 4 && memory.is_some_and(|available| available >= 2048) {
        2
    } else {
        1
    }
}

fn cpu_quota(text: &str) -> Option<usize> {
    let mut fields = text.split_whitespace();
    let quota: u64 = fields.next()?.parse().ok()?;
    let period: u64 = fields.next()?.parse().ok()?;
    (period > 0).then(|| (quota / period).max(1) as usize)
}

impl Resources {
    fn detect() -> Self {
        let mut cpu = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1);
        let mut memory = fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|text| {
                text.lines().find_map(|line| {
                    line.strip_prefix("MemAvailable:")?
                        .split_whitespace()
                        .next()?
                        .parse::<u64>()
                        .ok()
                })
            })
            .map(|kb| kb / 1024);
        let root = PathBuf::from("/sys/fs/cgroup");
        let group = fs::read_to_string("/proc/self/cgroup")
            .ok()
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("0::").map(str::to_owned))
            })
            .unwrap_or_else(|| "/".into());
        let directory = root.join(group.trim_start_matches('/'));
        for ancestor in directory
            .ancestors()
            .take_while(|path| path.starts_with(&root))
        {
            if let Some(limit) = fs::read_to_string(ancestor.join("cpu.max"))
                .ok()
                .and_then(|text| cpu_quota(&text))
            {
                cpu = cpu.min(limit);
            }
            let read_number = |name| {
                fs::read_to_string(ancestor.join(name))
                    .ok()?
                    .trim()
                    .parse::<u64>()
                    .ok()
            };
            if let (Some(limit), Some(used)) =
                (read_number("memory.max"), read_number("memory.current"))
            {
                let remaining = limit.saturating_sub(used) / 1024 / 1024;
                memory = Some(memory.map_or(remaining, |available| available.min(remaining)));
            }
        }
        Self {
            cpu_available: cpu,
            memory_available_mb: memory,
            max_concurrent_tasks: concurrency(cpu, memory),
            io_slots: 1,
            graphics_slots: 1,
            graphics_state: None,
            graphics_backend: None,
            virtual_machine: None,
            storage_available: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskStatus {
    id: Task,
    state: String,
    elapsed_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub phase: Phase,
    pub resources: Resources,
    tasks: Vec<TaskStatus>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

impl Report {
    fn new(resources: Resources) -> Self {
        Self {
            phase: Phase::Preparing,
            resources,
            warnings: vec![],
            error: None,
            elapsed_ms: 0,
            tasks: [
                Task::Library,
                Task::Hardware,
                Task::Services,
                Task::Emulators,
            ]
            .into_iter()
            .map(|id| TaskStatus {
                id,
                state: "pending".into(),
                elapsed_ms: 0,
            })
            .collect(),
        }
    }
    fn next_task(&self) -> Option<Task> {
        let running: Vec<_> = self
            .tasks
            .iter()
            .filter(|task| task.state == "running")
            .collect();
        if running.len() >= self.resources.max_concurrent_tasks {
            return None;
        }
        self.tasks
            .iter()
            .find(|task| {
                task.state == "pending"
                    && task.id.dependencies().iter().all(|dependency| {
                        self.tasks.iter().any(|entry| {
                            entry.id == *dependency
                                && matches!(entry.state.as_str(), "ready" | "degraded")
                        })
                    })
                    && (!task.id.uses_io() || !running.iter().any(|entry| entry.id.uses_io()))
            })
            .map(|task| task.id)
    }
}

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

enum Output {
    Library(SystemSettings, Vec<Platform>, Vec<Game>),
    Hardware(HardwareInfo),
    Services,
    Emulators(Vec<Emulator>),
}

type Notify = Arc<dyn Fn(Report) + Send + Sync>;
type Execute = Arc<
    dyn Fn(Task, Option<HardwareInfo>) -> Result<(Output, Vec<String>), EmuBoxError> + Send + Sync,
>;

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
        inner.report.error = Some(message);
        for task in &mut inner.report.tasks {
            if task.state == "running" {
                task.state = "error".into();
            } else if task.state == "pending" {
                task.state = "cancelled".into();
            }
        }
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
                let Some(task) = inner.report.next_task() else {
                    break;
                };
                inner
                    .report
                    .tasks
                    .iter_mut()
                    .find(|entry| entry.id == task)
                    .unwrap()
                    .state = "running".into();
                let hardware = inner.data.hardware.clone();
                drop(inner);
                self.publish(notify);
                let sender = sender.clone();
                let execute_task = execute_task.clone();
                std::thread::spawn(move || {
                    let started = Instant::now();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_task(task, hardware)
                    }))
                    .unwrap_or_else(|_| {
                        Err(EmuBoxError::ProcessFailed(
                            "Tarea de arranque interrumpida".into(),
                        ))
                    });
                    let _ = sender.send((task, started.elapsed().as_millis() as u64, result));
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
            let entry = inner
                .report
                .tasks
                .iter_mut()
                .find(|entry| entry.id == task)
                .unwrap();
            entry.state = if warnings.is_empty() {
                "ready"
            } else {
                "degraded"
            }
            .into();
            entry.elapsed_ms = elapsed;
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
            let finished = inner
                .report
                .tasks
                .iter()
                .all(|entry| matches!(entry.state.as_str(), "ready" | "degraded"));
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

fn execute(
    task: Task,
    hardware: Option<HardwareInfo>,
) -> Result<(Output, Vec<String>), EmuBoxError> {
    let mut warnings = Vec::new();
    let output = match task {
        Task::Library => {
            SystemService::get_config()?;
            let settings = SystemService::get_settings()?;
            download_manager::recover()?;
            crate::services::game_database::ensure_local_index()?;
            Output::Library(
                settings,
                GameService::get_platforms()?,
                GameService::get_games(None)?,
            )
        }
        Task::Hardware => {
            let hardware = SystemService::get_hardware_info()?;
            if hardware.graphics.detection_state
                == crate::models::graphics::DetectionState::Indeterminate
            {
                warnings.push(
                    "Deteccion grafica indeterminada; se conserva el backend operativo".into(),
                );
            }
            Output::Hardware(hardware)
        }
        Task::Services => {
            let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
            for service in ["bus", "pipewire-0", "pulse/native"] {
                if !runtime
                    .as_ref()
                    .is_some_and(|path| path.join(service).exists())
                {
                    warnings.push(format!("Servicio de sesion no disponible: {service}"));
                }
            }
            Output::Services
        }
        Task::Emulators => {
            let hardware = hardware.ok_or_else(|| {
                EmuBoxError::HardwareUnavailable("Falta instantanea de hardware".into())
            })?;
            let emulators = match EmulatorService::scan_with_hardware(&hardware) {
                Ok(emulators) => emulators,
                Err(error) => {
                    warnings.push(format!("Inventario de emuladores incompleto: {error}"));
                    EmulatorService::cached_with_hardware(&hardware)?
                }
            };
            if emulators
                .iter()
                .any(|emulator| emulator.version == "Instalado (version no disponible)")
            {
                warnings.push(
                    "No se pudo consultar la version de algunos emuladores dentro del plazo".into(),
                );
            }
            if let Err(error) = EmulatorService::apply_hardware_profile(&hardware) {
                warnings.push(format!("Perfiles de emuladores: {error}"));
            }
            Output::Emulators(emulators)
        }
    };
    Ok((output, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resources_reserve_capacity_and_do_not_use_gpu_brand() {
        assert_eq!(concurrency(1, Some(8192)), 1);
        assert_eq!(concurrency(16, Some(512)), 1);
        assert_eq!(concurrency(16, None), 1);
        assert_eq!(concurrency(4, Some(2048)), 2);
        assert_eq!(cpu_quota("100000 100000"), Some(1));
        assert_eq!(cpu_quota("max 100000"), None);
        assert_eq!(cpu_quota("100 0"), None);
    }
    #[test]
    fn task_dependencies_and_io_limit_are_enforced() {
        let mut resources = Resources::detect();
        resources.max_concurrent_tasks = 2;
        let mut report = Report::new(resources);
        assert_eq!(report.next_task(), Some(Task::Library));
        report.tasks[0].state = "running".into();
        assert_eq!(report.next_task(), Some(Task::Hardware));
        report.tasks[1].state = "running".into();
        assert_eq!(report.next_task(), None);
        report.tasks[1].state = "ready".into();
        report.tasks[2].state = "ready".into();
        assert_eq!(report.next_task(), None);
        report.tasks[0].state = "ready".into();
        assert_eq!(report.next_task(), Some(Task::Emulators));
    }
    #[test]
    fn constrained_startup_runs_one_task_at_a_time() {
        let mut resources = Resources::detect();
        resources.max_concurrent_tasks = 1;
        let mut report = Report::new(resources);
        for task in [
            Task::Library,
            Task::Hardware,
            Task::Services,
            Task::Emulators,
        ] {
            assert_eq!(report.next_task(), Some(task));
            let index = report
                .tasks
                .iter()
                .position(|entry| entry.id == task)
                .unwrap();
            report.tasks[index].state = "running".into();
            assert_eq!(report.next_task(), None);
            report.tasks[index].state = "ready".into();
        }
        assert_eq!(report.next_task(), None);
    }
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
            .tasks
            .iter()
            .skip(1)
            .all(|task| task.state == "cancelled"));

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
        release.send(()).unwrap();
        completion.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(startup.report().phase, Phase::Error);
        assert!(startup
            .report()
            .tasks
            .iter()
            .skip(1)
            .all(|task| task.state == "cancelled"));
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
