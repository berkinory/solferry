//! Keeps the desktop app current: finds a signed release, stages it, and swaps it in on request.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use solferry_core::{Config, Error, Result, update};

use crate::service::Update;

const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_EVERY: Duration = Duration::from_hours(6);
const STAGE_DIR: &str = "updates";

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Downloading {
        version: String,
        percent: u8,
    },
    Ready {
        version: String,
    },
    /// Newer, but this copy can't replace itself, so the user gets the release page.
    Available {
        version: String,
        page: String,
    },
    Failed,
}

/// Where this copy of the app lives, when it is somewhere it can replace itself.
#[derive(Clone, Debug)]
enum Target {
    Bundle(PathBuf),
    AppImage(PathBuf),
    Executable(PathBuf),
}

impl Target {
    fn current() -> Option<Self> {
        let target = if cfg!(target_os = "macos") {
            let exe = std::env::current_exe().ok()?;
            let bundle = exe.ancestors().nth(3)?;
            bundle
                .extension()
                .is_some_and(|ext| ext == "app")
                .then(|| Self::Bundle(bundle.into()))?
        } else if cfg!(windows) {
            Self::Executable(std::env::current_exe().ok()?)
        } else {
            // Child processes inherit APPIMAGE, so it only counts when this binary runs from it.
            let image = std::env::var_os("APPIMAGE")?;
            let mounted = std::env::var_os("APPDIR")?;
            let exe = std::env::current_exe().ok()?;
            exe.starts_with(mounted).then(|| Self::AppImage(image.into()))?
        };
        writable(target.path().parent()?).then_some(target)
    }

    fn path(&self) -> &Path {
        match self {
            Self::Bundle(path) | Self::AppImage(path) | Self::Executable(path) => path,
        }
    }
}

fn writable(folder: &Path) -> bool {
    let probe = folder.join(".solferry-update-probe");
    let created = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(probe);
    created
}

/// Puts `new` where `target` is. If that fails, `target` is back where it was.
fn swap(new: &Path, target: &Path) -> io::Result<()> {
    let old = target.with_extension("old");
    let _ = remove(&old);
    std::fs::rename(target, &old)?;
    if let Err(error) = std::fs::rename(new, target) {
        std::fs::rename(&old, target)?;
        return Err(error);
    }
    let _ = remove(&old);
    Ok(())
}

fn remove(path: &Path) -> io::Result<()> {
    if path.is_dir() { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) }
}

fn succeeded(command: &mut Command) -> io::Result<()> {
    let status = command.stdout(Stdio::null()).stderr(Stdio::null()).status()?;
    if status.success() { Ok(()) } else { Err(io::Error::other(format!("{command:?} failed"))) }
}

fn replace_bundle(payload: &Path, bundle: &Path) -> io::Result<()> {
    let folder = bundle.parent().ok_or_else(|| io::Error::other("the app has no folder"))?;
    let unpacked = folder.join(".solferry-update");
    let _ = remove(&unpacked);
    succeeded(Command::new("/usr/bin/ditto").arg("-x").arg("-k").arg(payload).arg(&unpacked))?;
    let result = swap(&unpacked.join("Solferry.app"), bundle);
    let _ = remove(&unpacked);
    result
}

fn replace_appimage(payload: &Path, image: &Path) -> io::Result<()> {
    let staged = image.with_extension("new");
    std::fs::copy(payload, &staged)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
    }
    std::fs::rename(staged, image)
}

fn replace(target: &Target, payload: &Path) -> io::Result<()> {
    match target {
        Target::Bundle(bundle) => replace_bundle(payload, bundle),
        Target::AppImage(image) => replace_appimage(payload, image),
        Target::Executable(_) => self_replace::self_replace(payload),
    }
}

/// Starts the new version once this process has gone, so the two never share the device key.
fn relaunch(target: &Target) -> io::Result<()> {
    const WAIT: &str = r#"while kill -0 "$0" 2>/dev/null; do sleep 0.1; done; exec "$@""#;
    let pid = std::process::id().to_string();
    let mut command = match target {
        Target::Bundle(bundle) => {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", WAIT, &pid, "/usr/bin/open", "-n"]).arg(bundle);
            command
        }
        Target::AppImage(image) => {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", WAIT, &pid]).arg(image);
            command
        }
        Target::Executable(exe) => powershell_relaunch(&pid, exe),
    };
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map(|_| ())
}

fn powershell_relaunch(pid: &str, exe: &Path) -> Command {
    let path = exe.display().to_string().replace('\'', "''");
    let script =
        format!("Wait-Process -Id {pid} -ErrorAction SilentlyContinue; Start-Process '{path}'");
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

#[derive(Clone)]
pub struct Updater {
    data_dir: PathBuf,
    updates: UnboundedSender<Update>,
    busy: Arc<AtomicBool>,
    staged: Arc<Mutex<Option<(PathBuf, Target)>>>,
}

impl Updater {
    pub fn new(data_dir: PathBuf, updates: UnboundedSender<Update>) -> Self {
        Self { data_dir, updates, busy: Arc::default(), staged: Arc::default() }
    }

    fn report(&self, status: Status) {
        let _ = self.updates.unbounded_send(Update::Updater(status));
    }

    /// Checks shortly after launch and then every few hours, while the setting allows it.
    pub fn follow(&self) {
        if !update::enabled() {
            return;
        }
        let updater = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(FIRST_CHECK).await;
            loop {
                if Config::load(&updater.data_dir).is_ok_and(|config| config.check_updates) {
                    updater.run(false).await;
                }
                tokio::time::sleep(CHECK_EVERY).await;
            }
        });
    }

    pub fn check_now(&self) {
        let updater = self.clone();
        tokio::spawn(async move { updater.run(true).await });
    }

    async fn run(&self, by_hand: bool) {
        if self.busy.swap(true, Ordering::SeqCst) {
            return;
        }
        if by_hand {
            self.report(Status::Checking);
        }
        match self.find().await {
            Ok(status) => self.report(status),
            Err(error) => {
                tracing::warn!(%error, "update failed");
                if by_hand {
                    self.report(Status::Failed);
                }
            }
        }
        self.busy.store(false, Ordering::SeqCst);
    }

    async fn find(&self) -> Result<Status> {
        let Some(release) = update::check().await? else { return Ok(Status::UpToDate) };
        let version = release.version.clone();
        let (Some(asset), Some(target)) = (release.asset(), Target::current()) else {
            return Ok(Status::Available { version, page: release.page.clone() });
        };
        let folder = self.data_dir.join(STAGE_DIR);
        let payload = folder.join(format!("{version}.update"));
        if !update::matches(asset, &payload).await {
            let _ = tokio::fs::remove_dir_all(&folder).await;
            tokio::fs::create_dir_all(&folder)
                .await
                .map_err(|error| Error::Update(error.to_string()))?;
            let mut shown = None;
            update::download(asset, &payload, |received, total| {
                let percent = u8::try_from(received * 100 / total.max(1)).unwrap_or(100);
                if shown != Some(percent) {
                    shown = Some(percent);
                    self.report(Status::Downloading { version: version.clone(), percent });
                }
            })
            .await?;
        }
        *self.staged.lock().unwrap_or_else(PoisonError::into_inner) = Some((payload, target));
        Ok(Status::Ready { version })
    }

    /// Replaces the app with the staged release, starts it, and asks this copy to quit.
    pub fn install(&self) {
        let staged = self.staged.lock().unwrap_or_else(PoisonError::into_inner).clone();
        let Some((payload, target)) = staged else { return };
        let (out, folder) = (self.updates.clone(), self.data_dir.join(STAGE_DIR));
        tokio::task::spawn_blocking(move || {
            let result = replace(&target, &payload).and_then(|()| relaunch(&target));
            let update = match result {
                Ok(()) => {
                    let _ = std::fs::remove_dir_all(folder);
                    Update::Restarting
                }
                Err(error) => {
                    tracing::warn!(%error, "installing the update failed");
                    Update::InstallFailed
                }
            };
            let _ = out.unbounded_send(update);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder_with(root: &Path, name: &str, marker: &str) -> PathBuf {
        let path = root.join(name);
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("marker"), marker).unwrap();
        path
    }

    #[test]
    fn a_swap_leaves_only_the_new_version() {
        let root = tempfile::tempdir().unwrap();
        let old = folder_with(root.path(), "Solferry.app", "old");
        let new = folder_with(root.path(), "unpacked", "new");
        swap(&new, &old).unwrap();
        assert_eq!(std::fs::read_to_string(old.join("marker")).unwrap(), "new");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_failed_swap_puts_the_old_version_back() {
        let root = tempfile::tempdir().unwrap();
        let old = folder_with(root.path(), "Solferry.app", "old");
        assert!(swap(&root.path().join("missing"), &old).is_err());
        assert_eq!(std::fs::read_to_string(old.join("marker")).unwrap(), "old");
    }
}
