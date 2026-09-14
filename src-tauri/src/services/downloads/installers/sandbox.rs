use super::failure;
use crate::{errors::EmuBoxError, services::binary_service::resolve_executable};
use std::{
    path::Path,
    process::{Child, Command, Stdio},
};

pub(super) struct Worker(pub(super) Child);
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub(super) fn sandbox(
    source: &Path,
    output: &Path,
    tool: &Path,
    input: &str,
) -> Result<Command, EmuBoxError> {
    let limiter = resolve_executable("prlimit")
        .ok_or_else(|| failure("Falta prlimit (util-linux) para limitar el preparador"))?;
    let bwrap = resolve_executable("bwrap")
        .ok_or_else(|| failure("Falta bubblewrap; no se ejecuta el preparador sin aislamiento"))?;
    let mut command = Command::new(limiter);
    let memory_limit = if input.ends_with(".pkg") {
        "--as=137438953472"
    } else {
        "--as=8589934592"
    };
    command
        .args([
            memory_limit,
            "--fsize=107374182400",
            "--cpu=1800",
            "--core=0",
            "--",
        ])
        .arg(bwrap)
        .args([
            "--unshare-all",
            "--die-with-parent",
            "--new-session",
            "--ro-bind",
            "/usr",
            "/usr",
            "--symlink",
            "usr/lib",
            "/lib",
            "--symlink",
            "usr/lib",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/tmp",
            "--dir",
            "/input",
            "--dir",
            "/tool",
            "--ro-bind",
        ])
        .arg(source)
        .arg(input)
        .arg("--ro-bind")
        .arg(tool)
        .arg("/tool/prepare")
        .arg("--bind")
        .arg(output)
        .arg("/output")
        .args([
            "--clearenv",
            "--setenv",
            "PATH",
            "/usr/bin",
            "--setenv",
            "LANG",
            "C.UTF-8",
            "--setenv",
            "HOME",
            "/output/home",
            "--setenv",
            "XDG_CONFIG_HOME",
            "/output/home/config",
            "--setenv",
            "XDG_CACHE_HOME",
            "/output/home/cache",
            "--setenv",
            "XDG_DATA_HOME",
            "/output/home/data",
            "--chdir",
            "/output",
            "--",
            "/tool/prepare",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    Ok(command)
}
