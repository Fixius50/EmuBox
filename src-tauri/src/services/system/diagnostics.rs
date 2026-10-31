use crate::errors::EmuBoxError;
use crate::models::{DiagnosticReport, LogEntry};
use std::process::Command;

pub struct DiagnosticsService;

impl DiagnosticsService {
    pub fn get_system_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
        crate::services::log_service::system(limit)
    }

    pub fn get_emubox_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
        crate::services::log_service::session(limit)
    }

    pub fn get_diagnostics() -> Result<DiagnosticReport, EmuBoxError> {
        let system = crate::services::SystemService::get_system_info()?;
        let emulators = crate::services::EmulatorService::get_emulators()?;
        let installed = emulators
            .iter()
            .filter(|emulator| emulator.compatibility.status == "supported")
            .count();
        let hardware = &system.hardware;
        let summary = format!("architecture={} kernelArchitecture={} cpu={} cores={} memoryMiB={} gpuVendor={} renderer={} vulkan={} opengl={} openglAccelerated={} openglRenderer={} drm={} gamescope={} compositor={} device={} accelerated={} backend={} gpuKind={} vm={}",
            system.architecture, system.kernel_architecture, hardware.cpu_model, hardware.cpu_cores,
            hardware.total_memory_mb, hardware.gpu_vendor, hardware.gpu_renderer, hardware.vulkan_supported,
            hardware.opengl_supported, hardware.opengl_accelerated, hardware.opengl_renderer.as_deref().unwrap_or("unknown"),
            hardware.drm_available, hardware.gamescope_available, hardware.recommended_compositor, hardware.device_model,
            hardware.graphics_accelerated, hardware.graphics_backend, hardware.gpu_kind, hardware.is_virtual_machine);
        Ok(DiagnosticReport {
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            os_info: system.os_name,
            kernel_version: system.kernel_version,
            architecture: system.architecture,
            gpu_adapter: hardware.gpu_renderer.clone(),
            vulkan_ready: hardware.vulkan_supported,
            gamescope_ready: hardware.gamescope_ready,
            pipewire_ready: Command::new("pgrep")
                .args(["-x", "pipewire"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false),
            storage_mounted: std::path::Path::new(&crate::services::paths::games_dir()).is_dir(),
            emulators_installed_count: installed,
            emulators_missing_count: emulators.len() - installed,
            connected_gamepads_count: crate::services::SystemService::detect_gamepads().len(),
            recent_errors: Self::get_system_logs(Some(100)).ok().map(|entries| {
                entries
                    .into_iter()
                    .filter(|entry| entry.level == "error")
                    .collect()
            }),
            raw_summary_text: format!(
                "{summary}\ngraphicsEvidence={}",
                serde_json::to_string(&hardware.graphics)
                    .map_err(|error| EmuBoxError::Unknown(error.to_string()))?
            ),
        })
    }

    pub fn submit_report(error_context: Option<String>) -> Result<crate::models::ReportSubmissionResult, EmuBoxError> {
        let diag = Self::get_diagnostics()?;
        let now = chrono::Utc::now();
        let timestamp_str = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let file_timestamp = now.format("%Y%m%d-%H%M%S").to_string();

        let report_dir = std::path::Path::new("/var/log/emubox/reports");
        let _ = std::fs::create_dir_all(report_dir);
        let fallback_dir = std::env::temp_dir().join("emubox-reports");
        let target_dir = if report_dir.is_dir() { report_dir } else {
            let _ = std::fs::create_dir_all(&fallback_dir);
            fallback_dir.as_path()
        };
        let local_path = target_dir.join(format!("report-{file_timestamp}.md"));

        let error_title = error_context.as_deref().unwrap_or("Fallo no especificado durante arranque o sesion");

        let recent_errors_text = diag.recent_errors.as_ref()
            .map(|errs| errs.iter().map(|e| format!("[{}] {}", e.level, e.message)).collect::<Vec<_>>().join("\n"))
            .unwrap_or_else(|| "Sin errores recientes en logs".into());

        let body = format!(
            "### Reporte Automatico de Error - EmuBox Appliance\n\n            **Fecha:** {timestamp_str}\n            **Error:** `{error_title}`\n\n            #### Entorno del Sistema\n            - **SO:** {}\n            - **Kernel:** {}\n            - **Arquitectura:** {}\n            - **GPU Adapter:** {}\n            - **Vulkan:** {} | **Gamescope:** {} | **PipeWire:** {}\n            - **Emuladores:** {} instalados, {} ausentes\n            - **Mandos conectados:** {}\n\n            #### Diagnostico de Hardware\n            ```text\n            {}\n            ```\n\n            #### Errores recientes de sesion\n            ```text\n            {}\n            ```\n",
            diag.os_info,
            diag.kernel_version,
            diag.architecture,
            diag.gpu_adapter,
            diag.vulkan_ready,
            diag.gamescope_ready,
            diag.pipewire_ready,
            diag.emulators_installed_count,
            diag.emulators_missing_count,
            diag.connected_gamepads_count,
            diag.raw_summary_text,
            recent_errors_text
        );

        let _ = std::fs::write(&local_path, &body);

        let token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("EMUBOX_REPORT_TOKEN"))
            .or_else(|_| std::fs::read_to_string("/etc/emubox/github-token").map(|s| s.trim().to_string()))
            .ok()
            .filter(|t| !t.is_empty());

        let issue_title = format!("[Auto-Report] {}", error_title.chars().take(80).collect::<String>());

        if let Some(tok) = token {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("EmuBox-Appliance-Reporter/1.0")
                .build()
                .map_err(|e| EmuBoxError::ProcessFailed(e.to_string()))?;

            let payload = serde_json::json!({
                "title": issue_title,
                "body": body,
                "labels": ["bug", "automated-report"]
            });

            if let Ok(res) = client
                .post("https://api.github.com/repos/Fixius50/EmuBox/issues")
                .header("Authorization", format!("Bearer {tok}"))
                .header("Accept", "application/vnd.github+json")
                .json(&payload)
                .send()
            {
                if res.status().is_success() {
                    let issue_data: serde_json::Value = res.json().unwrap_or_default();
                    let url = issue_data.get("html_url").and_then(|u| u.as_str()).map(ToString::to_string);
                    return Ok(crate::models::ReportSubmissionResult {
                        success: true,
                        issue_url: url.clone(),
                        local_report_path: local_path.to_string_lossy().to_string(),
                        message: format!("Reporte publicado automaticamente en GitHub: {}", url.unwrap_or_default()),
                    });
                }
            }
        }

        let mut url = reqwest::Url::parse("https://github.com/Fixius50/EmuBox/issues/new")
            .unwrap_or_else(|_| reqwest::Url::parse("https://github.com/Fixius50/EmuBox").unwrap());
        url.query_pairs_mut()
            .append_pair("title", &issue_title)
            .append_pair("body", &body);
        let issue_url = url.to_string();
        let _ = std::fs::write(target_dir.join("latest-issue.url"), &issue_url);

        Ok(crate::models::ReportSubmissionResult {
            success: true,
            issue_url: Some(issue_url),
            local_report_path: local_path.to_string_lossy().to_string(),
            message: format!("Reporte guardado en {}", local_path.display()),
        })
    }
}
