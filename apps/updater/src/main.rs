#![cfg_attr(windows, windows_subsystem = "windows")]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

const WAIT_FOR_EXIT: Duration = Duration::from_secs(60);
const INSTALLER: &str = "msiexec.exe";
const FAILED_EXTENSION: &str = "failed";

struct Plan {
    installer: PathBuf,
    previous_engine: u32,
    relaunch: PathBuf,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(plan) = parse(&arguments) else {
        return ExitCode::FAILURE;
    };
    let log = plan.installer.with_extension("log");
    wait_for_exit(plan.previous_engine);
    let outcome = Command::new(system_tool(INSTALLER))
        .args([
            "/i".as_ref(),
            plan.installer.as_os_str(),
            "/qn".as_ref(),
            "/norestart".as_ref(),
        ])
        .arg("/l*v")
        .arg(&log)
        .stdin(Stdio::null())
        .status();
    let is_installed = outcome.as_ref().is_ok_and(|status| status.success());
    record(&log, &format!("installation : {outcome:?}"));
    let cleanup = if is_installed {
        fs::remove_file(&plan.installer)
    } else {
        fs::rename(
            &plan.installer,
            plan.installer.with_extension(FAILED_EXTENSION),
        )
    };
    if let Err(error) = cleanup {
        record(&log, &format!("nettoyage : {error}"));
    }
    if let Err(error) = Command::new(&plan.relaunch).spawn() {
        record(&log, &format!("relance : {error}"));
    }
    if is_installed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse(arguments: &[String]) -> Option<Plan> {
    let [installer, previous_engine, relaunch] = arguments else {
        return None;
    };
    let installer = PathBuf::from(installer);
    let is_installer = installer
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("msi"));
    (is_installer && installer.is_file()).then_some(Plan {
        installer,
        previous_engine: previous_engine.parse().ok()?,
        relaunch: PathBuf::from(relaunch),
    })
}

fn system_tool(name: &str) -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(|root| PathBuf::from(root).join("System32").join(name))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

fn record(log: &Path, message: &str) {
    let file = OpenOptions::new().create(true).append(true).open(log);
    if let Ok(mut file) = file {
        let _ignored = writeln!(file, "[rustpaper-updater] {message}");
    }
}

#[cfg(windows)]
fn wait_for_exit(process_id: u32) {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
    };
    let Ok(process) = (unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, process_id) }) else {
        return;
    };
    let milliseconds = u32::try_from(WAIT_FOR_EXIT.as_millis()).unwrap_or(u32::MAX);
    unsafe {
        WaitForSingleObject(process, milliseconds);
        let _closed = CloseHandle(process);
    }
}

#[cfg(not(windows))]
fn wait_for_exit(process_id: u32) {
    let deadline = std::time::Instant::now() + WAIT_FOR_EXIT;
    let process = PathBuf::from(format!("/proc/{process_id}"));
    while process.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_arguments_that_are_not_an_existing_installer() {
        let arguments = ["setup.exe".to_owned(), "1".to_owned(), "app".to_owned()];
        assert!(parse(&arguments).is_none());
        assert!(parse(&[]).is_none());
    }
}
