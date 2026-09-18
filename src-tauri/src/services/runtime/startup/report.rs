use serde::Serialize;
use std::{fs, path::PathBuf};

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
pub(in crate::services::runtime::startup) enum Task {
    Library,
    Hardware,
    Services,
    Emulators,
}

impl Task {
    pub(in crate::services::runtime::startup) fn dependencies(self) -> &'static [Task] {
        match self {
            Self::Emulators => &[Self::Library, Self::Hardware],
            _ => &[],
        }
    }

    pub(in crate::services::runtime::startup) fn uses_io(self) -> bool {
        matches!(self, Self::Library | Self::Emulators)
    }

    pub(in crate::services::runtime::startup) fn telemetry_name(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Hardware => "hardware",
            Self::Services => "services",
            Self::Emulators => "emulators",
        }
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
    pub(super) fn detect() -> Self {
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
    pub(super) fn new(resources: Resources) -> Self {
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

    pub(super) fn next_task(&self) -> Option<Task> {
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

    pub(super) fn start_next_task(&mut self) -> Option<Task> {
        let task = self.next_task()?;
        self.tasks
            .iter_mut()
            .find(|entry| entry.id == task)
            .unwrap()
            .state = "running".into();
        Some(task)
    }

    pub(super) fn finish_task(&mut self, task: Task, elapsed_ms: u64, degraded: bool) {
        let entry = self
            .tasks
            .iter_mut()
            .find(|entry| entry.id == task)
            .unwrap();
        entry.state = if degraded { "degraded" } else { "ready" }.into();
        entry.elapsed_ms = elapsed_ms;
    }

    pub(super) fn all_tasks_finished(&self) -> bool {
        self.tasks
            .iter()
            .all(|entry| matches!(entry.state.as_str(), "ready" | "degraded"))
    }

    pub(super) fn running_task_names(&self) -> Vec<String> {
        self.tasks
            .iter()
            .filter(|task| task.state == "running")
            .map(|task| format!("{:?}", task.id))
            .collect()
    }

    pub(super) fn cancel_pending_and_error_running(&mut self) {
        for task in &mut self.tasks {
            if task.state == "running" {
                task.state = "error".into();
            } else if task.state == "pending" {
                task.state = "cancelled".into();
            }
        }
    }

    #[cfg(test)]
    pub(super) fn states_after(&self, skip: usize) -> Vec<&str> {
        self.tasks
            .iter()
            .skip(skip)
            .map(|task| task.state.as_str())
            .collect()
    }
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
        assert_eq!(report.start_next_task(), Some(Task::Library));
        assert_eq!(report.next_task(), Some(Task::Hardware));
        assert_eq!(report.start_next_task(), Some(Task::Hardware));
        assert_eq!(report.next_task(), None);
        report.finish_task(Task::Hardware, 1, false);
        report.finish_task(Task::Services, 1, false);
        assert_eq!(report.next_task(), None);
        report.finish_task(Task::Library, 1, false);
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
            assert_eq!(report.start_next_task(), Some(task));
            assert_eq!(report.next_task(), None);
            report.finish_task(task, 1, false);
        }
        assert_eq!(report.next_task(), None);
    }
}
