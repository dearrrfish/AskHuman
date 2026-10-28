//! Dev Instance discovery and CLI entry redirection.
//!
//! Normal calls use the system instance regardless of cwd markers. `dev exec`
//! explicitly selects a worktree binary and home; invoking that binary directly
//! also binds its home. An explicit `ASKHUMAN_HOME` remains isolated.
//! See `docs/specs/dev-instance-parallel.md`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Env var: overrides `paths::config_dir()` (instance home root).
pub const ASKHUMAN_HOME_ENV: &str = "ASKHUMAN_HOME";

/// Marker file created by `dev enable` under the worktree root.
pub const ENABLED_MARKER: &str = "enabled";

/// Relative directory name under the worktree root.
pub const DEV_DIR: &str = ".askhuman-dev";

/// Whether this process is running in a Dev Instance (non-empty `ASKHUMAN_HOME`).
pub fn is_dev_instance() -> bool {
    std::env::var_os(ASKHUMAN_HOME_ENV).is_some_and(|v| !v.is_empty())
}

/// Whether this process must leave **user-level global state** alone: the login items and the
/// Agent hook files (`~/.cursor/hooks.json`, `~/.codex/hooks.json`, `~/.claude/settings.json`, the
/// Pi extension).
///
/// An instance (a Dev Instance, or any process run with its own `ASKHUMAN_HOME`) shares those
/// files with the production install. Writing them from an instance hijacks production: the
/// entries would point at the instance binary and follow the instance's own, usually empty,
/// preferences. This happened to the login items (2026-07-25) and twice more in 2026-10, when an
/// isolated QA daemon's startup migration rewrote the user's Stop hooks and dropped `confirm`.
///
/// The env var alone is not enough: launchd restarts a daemon without the instance env, so the
/// executable path (a `.askhuman-dev` segment) is checked as well.
///
/// This guards *automatic* writes. Explicit user actions (`AskHuman agents ...`, the settings
/// page) are not blocked.
pub fn is_dev_instance_context() -> bool {
    is_dev_instance_context_for(
        std::env::var_os(ASKHUMAN_HOME_ENV).as_deref(),
        std::env::current_exe().ok().as_deref(),
    )
}

fn is_dev_instance_context_for(home_env: Option<&OsStr>, exe: Option<&Path>) -> bool {
    // An empty value counts as unset, exactly like `paths::config_dir()`.
    if home_env.is_some_and(|value| !value.is_empty()) {
        return true;
    }
    exe.is_some_and(|path| path.components().any(|part| part.as_os_str() == DEV_DIR))
}

/// Walk upward from `start` looking for `<ancestor>/.askhuman-dev/enabled`.
/// Returns the worktree root (`ancestor`) on the nearest hit.
pub fn find_dev_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    if cur.is_file() {
        cur.pop();
    }
    loop {
        let marker = cur.join(DEV_DIR).join(ENABLED_MARKER);
        if marker.is_file() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Instance home directory for a worktree root: `<root>/.askhuman-dev/home`.
pub fn instance_home(root: &Path) -> PathBuf {
    root.join(DEV_DIR).join("home")
}

/// Instance binary path for a worktree root.
pub fn instance_bin(root: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "AskHuman.exe"
    } else {
        "AskHuman"
    };
    root.join(DEV_DIR).join("bin").join(name)
}

/// Command class for dispatcher policy (spec D18 / plan §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandClass {
    /// Spawned child roles: inherit the resolved home, never redirect by cwd.
    Skip,
    /// Help / version: run without loading config or requiring an instance bin.
    Meta,
    /// Dev Instance management selects its home explicitly and stays in the dispatcher binary. This lets
    /// Windows purge an instance without running from the executable being deleted.
    Dev,
    /// Settings / history / config / channel: use the resolved invocation target.
    Config,
    /// Ask / daemon / mcp / …: use the resolved invocation target.
    Runtime,
}

/// Classify `argv` (full args including program name at `[0]`).
pub fn classify_command(argv: &[String]) -> CommandClass {
    let Some(first) = argv.get(1).map(|s| s.as_str()) else {
        // bare `AskHuman` with no args → treat as runtime-ish help path; no redirect needed
        // beyond env. Use Meta so missing bin does not hard-fail before help.
        return CommandClass::Meta;
    };
    match first {
        "--popup"
        | "--popup-host"
        | "--gui-host"
        | "__permission-diff-worker"
        | "__permission-shell-worker"
        | "__update-worker"
        | "__npm-update-worker" => CommandClass::Skip,
        "dev" => CommandClass::Dev,
        "--help" | "-h" | "--version" | "-v" | "--agent-help" | "--scripting-help" => {
            CommandClass::Meta
        }
        "--settings" | "--history" | "--todos" | "config" | "channel" => CommandClass::Config,
        _ => CommandClass::Runtime,
    }
}

/// A resolved invocation target. Resolution never reads configuration or contacts a daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub kind: &'static str,
    pub bin: PathBuf,
    pub home: PathBuf,
    pub reason: &'static str,
}

/// Use the installation entry captured at startup, never a replaced running inode.
pub fn installed_entry() -> Result<PathBuf, String> {
    crate::executable::launch_path()
        .map(stable_entry_path)
        .map_err(|e| format!("cannot resolve executable installation entry: {e}"))
}

fn stable_entry_path(path: PathBuf) -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        if let Some(entry) = path.as_os_str().as_bytes().strip_suffix(b" (deleted)") {
            return std::ffi::OsString::from_vec(entry.to_vec()).into();
        }
    }
    path
}

fn executable_root(exe: &Path) -> Option<&Path> {
    let bin = exe.parent()?;
    let dev = bin.parent()?;
    if bin.file_name()? == "bin"
        && dev.file_name()? == DEV_DIR
        && exe.file_name()? == instance_bin(Path::new("")).file_name()?
    {
        dev.parent()
    } else {
        None
    }
}

fn absolute_from(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    let normalize = |p: &Path| {
        // Existing homes may be symlinked; missing homes still need lexical normalization.
        if let Ok(p) = std::fs::canonicalize(p) {
            return p;
        }
        let mut normalized = PathBuf::new();
        for component in p.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other.as_os_str()),
            }
        }
        normalized
    };
    normalize(a) == normalize(b)
}

/// `dev_root` is supplied only for explicit `dev exec` / `dev target --dev`.
/// Cwd markers alone never select a runtime instance.
pub fn resolve_target(
    cwd: &Path,
    exe: &Path,
    home_env: Option<&OsStr>,
    system_home: &Path,
    dev_root: Option<&Path>,
) -> Result<Target, String> {
    let explicit = home_env
        .filter(|v| !v.is_empty())
        .map(|v| absolute_from(cwd, Path::new(v)));
    let own_root = executable_root(exe);
    if let (Some(own), Some(requested)) = (own_root, dev_root) {
        if !same_path(own, requested) {
            return Err(format!(
                "Dev Instance binary {} cannot select another worktree {}",
                exe.display(),
                requested.display()
            ));
        }
    }
    if let Some(root) = dev_root.or(own_root) {
        let home = instance_home(root);
        if let Some(explicit) = explicit {
            if !same_path(&explicit, &home) {
                return Err(format!("ASKHUMAN_HOME={} conflicts with Dev Instance home {}; unset it or use this instance's home", explicit.display(), home.display()));
            }
        }
        return Ok(Target {
            kind: "dev",
            bin: instance_bin(root),
            home,
            reason: if dev_root.is_some() {
                "explicit dev selection"
            } else {
                "dev binary installation entry"
            },
        });
    }
    Ok(Target {
        kind: if explicit.is_some() {
            "explicit-home"
        } else {
            "system"
        },
        bin: exe.to_path_buf(),
        home: explicit
            .clone()
            .unwrap_or_else(|| system_home.to_path_buf()),
        reason: if explicit.is_some() {
            "explicit ASKHUMAN_HOME"
        } else {
            "default system instance (cwd markers affect installation only)"
        },
    })
}

pub fn current_target(dev: bool) -> Result<Target, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("cannot resolve cwd: {e}"))?;
    let root = if dev {
        Some(find_dev_root(&cwd).ok_or_else(|| {
            format!(
                "no enabled Dev Instance from {}; run `AskHuman dev enable` first",
                cwd.display()
            )
        })?)
    } else {
        None
    };
    resolve_target(
        &cwd,
        &installed_entry()?,
        std::env::var_os(ASKHUMAN_HOME_ENV).as_deref(),
        &crate::paths::home().join(".askhuman"),
        root.as_deref(),
    )
}

/// Apply a resolved target before any config, locale or GUI initialization.
pub fn apply_target(target: &Target) {
    if target.kind != "system" {
        std::env::set_var(ASKHUMAN_HOME_ENV, &target.home);
        std::env::set_var("ASKHUMAN_NO_KEYCHAIN", "1");
    }
}

/// Bootstrap all roles, including daemon/popup/workers. Inherited homes remain intact.
pub fn maybe_enter_dev_instance() {
    match current_target(false) {
        Ok(target) => apply_target(&target),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}

pub fn validate_dev_binary(bin: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(bin).map_err(|e| format!("Dev Instance binary is missing or inaccessible: {}: {e}. Run `./scripts/install.sh` in this worktree first", bin.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "Dev Instance binary is not a file: {}",
            bin.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!("Dev Instance binary is not executable: {}. Run `./scripts/install.sh` in this worktree first", bin.display()));
        }
    }
    Ok(())
}

/// Only AskHuman arguments are accepted; `--` separates the wrapper from its payload.
pub fn dev_exec_args(args: &[String]) -> Result<&[String], String> {
    if args.first().map(String::as_str) != Some("--") || args.len() < 2 {
        return Err("usage: AskHuman dev exec -- <AskHuman arguments>".into());
    }
    Ok(&args[1..])
}

pub fn exec_dev(args: &[String]) -> ! {
    let prepared = dev_exec_args(args).and_then(|payload| {
        let target = current_target(true)?;
        validate_dev_binary(&target.bin)?;
        let mut argv = vec![target.bin.to_string_lossy().into_owned()];
        argv.extend_from_slice(payload);
        Ok((target, argv))
    });
    match prepared {
        Ok((target, argv)) => reexec_into(&target.bin, &target.home, &argv),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}

fn reexec_into(bin: &Path, home: &Path, argv: &[String]) -> ! {
    let args: Vec<&str> = argv.iter().skip(1).map(|s| s.as_str()).collect();
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(bin)
            .args(&args)
            .env(ASKHUMAN_HOME_ENV, home)
            .env("ASKHUMAN_NO_KEYCHAIN", "1")
            .exec();
        eprintln!(
            "error: failed to re-exec Dev Instance binary {}: {err}",
            bin.display()
        );
        std::process::exit(1);
    }
    #[cfg(not(unix))]
    {
        let status = std::process::Command::new(bin)
            .args(&args)
            .env(ASKHUMAN_HOME_ENV, home)
            .env("ASKHUMAN_NO_KEYCHAIN", "1")
            .status();
        match status {
            Ok(s) => std::process::exit(s.code().unwrap_or(1)),
            Err(e) => {
                eprintln!(
                    "error: failed to re-exec Dev Instance binary {}: {e}",
                    bin.display()
                );
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "askhuman-routing-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn enabled_marker_never_selects_runtime_target() {
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(DEV_DIR)).unwrap();
        fs::write(tmp.path().join(DEV_DIR).join(ENABLED_MARKER), "").unwrap();
        let exe = Path::new("/system/bin/AskHuman");
        let target =
            resolve_target(tmp.path(), exe, None, Path::new("/system/home"), None).unwrap();
        assert!(find_dev_root(tmp.path()).is_some());
        assert_eq!(target.kind, "system");
        assert_eq!(target.bin, exe);
        assert_eq!(target.home, Path::new("/system/home"));
        // Missing dev bin also cannot break ordinary system invocations.
        assert!(!instance_bin(tmp.path()).exists());
    }

    #[test]
    fn explicit_home_is_preserved_and_absolutized() {
        let target = resolve_target(
            Path::new("/cwd"),
            Path::new("/bin/AskHuman"),
            Some(OsStr::new("qa/home")),
            Path::new("/system"),
            None,
        )
        .unwrap();
        assert_eq!(target.kind, "explicit-home");
        assert_eq!(target.home, Path::new("/cwd/qa/home"));
        assert_eq!(
            resolve_target(
                Path::new("/cwd"),
                Path::new("/bin/AskHuman"),
                Some(OsStr::new("")),
                Path::new("/system"),
                None
            )
            .unwrap()
            .kind,
            "system"
        );
    }

    #[test]
    fn dev_selection_and_direct_binary_bind_own_home() {
        let root = Path::new("/work/tree");
        let bin = instance_bin(root);
        for selection in [None, Some(root)] {
            let target = resolve_target(
                Path::new("/elsewhere"),
                &bin,
                None,
                Path::new("/system"),
                selection,
            )
            .unwrap();
            assert_eq!(target.kind, "dev");
            assert_eq!(target.home, instance_home(root));
            assert_eq!(target.bin, bin);
        }
        // The marker is intentionally absent: installed executable identity alone is enough.
        let target = resolve_target(
            Path::new("/work/tree"),
            Path::new("/bin/AskHuman"),
            None,
            Path::new("/system"),
            Some(root),
        )
        .unwrap();
        assert_eq!(target.home, instance_home(root));
    }

    #[test]
    fn explicit_conflicts_fail_without_switching_instances() {
        let root = Path::new("/work/tree");
        for exe in [instance_bin(root), PathBuf::from("/bin/AskHuman")] {
            assert!(resolve_target(
                Path::new("/"),
                &exe,
                Some(OsStr::new("/other/home")),
                Path::new("/system"),
                Some(root)
            )
            .unwrap_err()
            .contains("conflicts"));
        }
        assert!(resolve_target(
            Path::new("/"),
            &instance_bin(root),
            None,
            Path::new("/system"),
            Some(Path::new("/other/tree"))
        )
        .is_err());
        assert!(resolve_target(
            root,
            &instance_bin(root),
            Some(OsStr::new(".askhuman-dev/./home")),
            Path::new("/system"),
            None
        )
        .is_ok());
    }

    #[test]
    fn dev_exec_payload_preserves_arguments_verbatim() {
        let args: Vec<String> = [
            "--",
            "-q",
            "spaces and $literal",
            "-f",
            "./relative file.md",
            "--output",
            "json",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        assert_eq!(dev_exec_args(&args).unwrap(), &args[1..]);
        assert!(dev_exec_args(&[]).is_err());
        assert!(dev_exec_args(&["--".into()]).is_err());
        assert!(dev_exec_args(&["--help".into()]).is_err());
    }

    #[test]
    fn missing_and_non_executable_dev_binary_fail_closed() {
        let tmp = TestDir::new();
        let bin = instance_bin(tmp.path());
        assert!(validate_dev_binary(&bin).unwrap_err().contains("missing"));
        fs::create_dir_all(bin.parent().unwrap()).unwrap();
        fs::write(&bin, "not executable").unwrap();
        #[cfg(unix)]
        assert!(validate_dev_binary(&bin)
            .unwrap_err()
            .contains("not executable"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn deleted_linux_image_keeps_installation_identity() {
        let entry = stable_entry_path(PathBuf::from(
            "/work/tree/.askhuman-dev/bin/AskHuman (deleted)",
        ));
        assert_eq!(executable_root(&entry), Some(Path::new("/work/tree")));
        assert_eq!(entry, instance_bin(Path::new("/work/tree")));
    }

    // Child-only probe exercises the real bootstrap without accessing configuration or IPC.
    #[test]
    fn routing_process_probe() {
        let Some(mode) = std::env::var_os("ASKHUMAN_ROUTING_TEST_CHILD") else {
            return;
        };
        maybe_enter_dev_instance();
        if mode == "exec" {
            exec_dev(&[
                "--".into(),
                "-q".into(),
                "spaces and $literal".into(),
                "-f".into(),
                "./attachment.md".into(),
            ]);
        }
        println!(
            "resolved-home={}",
            std::env::var(ASKHUMAN_HOME_ENV).unwrap_or_default()
        );
        println!(
            "keychain={}",
            std::env::var("ASKHUMAN_NO_KEYCHAIN").unwrap_or_default()
        );
    }

    #[cfg(unix)]
    #[test]
    fn process_dev_exec_preserves_cwd_metadata_arguments_and_exit() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TestDir::new();
        let bin = instance_bin(tmp.path());
        fs::create_dir_all(bin.parent().unwrap()).unwrap();
        fs::write(tmp.path().join(DEV_DIR).join(ENABLED_MARKER), "").unwrap();
        fs::write(
            &bin,
            r#"#!/bin/sh
printf '%s\n' "$ASKHUMAN_HOME" "$PWD" "$ASKHUMAN_MCP_AGENT_SESSION_ID" "$ASKHUMAN_NO_KEYCHAIN" "$@"
exit 23
"#,
        )
        .unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["routing_process_probe", "--nocapture"])
            .current_dir(tmp.path())
            .env("ASKHUMAN_ROUTING_TEST_CHILD", "exec")
            .env("ASKHUMAN_MCP_AGENT_SESSION_ID", "session-keep")
            .env_remove(ASKHUMAN_HOME_ENV)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(23),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        for value in [
            instance_home(tmp.path()).to_string_lossy().as_ref(),
            tmp.path().to_string_lossy().as_ref(),
            "session-keep",
            "spaces and $literal",
            "./attachment.md",
        ] {
            assert!(stdout.contains(value), "missing {value}: {stdout}");
        }
    }

    #[test]
    fn process_default_and_internal_roles_preserve_home_selection() {
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(DEV_DIR)).unwrap();
        fs::write(tmp.path().join(DEV_DIR).join(ENABLED_MARKER), "").unwrap();
        for role in [
            "daemon",
            "--popup",
            "--popup-host",
            "--gui-host",
            "__permission-shell-worker",
            "__update-worker",
        ] {
            for home in [None, Some("relative-isolated-home")] {
                let mut child = std::process::Command::new(std::env::current_exe().unwrap());
                child
                    .args(["routing_process_probe", "--nocapture"])
                    .current_dir(tmp.path())
                    .env("ASKHUMAN_ROUTING_TEST_CHILD", role)
                    .env_remove(ASKHUMAN_HOME_ENV)
                    .env_remove("ASKHUMAN_NO_KEYCHAIN");
                if let Some(home) = home {
                    child.env(ASKHUMAN_HOME_ENV, home);
                }
                let output = child.output().unwrap();
                assert!(
                    output.status.success(),
                    "{role}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let stdout = String::from_utf8_lossy(&output.stdout);
                let expected = home
                    .map(|v| tmp.path().join(v).display().to_string())
                    .unwrap_or_default();
                assert!(
                    stdout.contains(&format!("resolved-home={expected}\n")),
                    "{role}: {stdout}"
                );
                if home.is_some() {
                    assert!(stdout.contains("keychain=1"));
                }
            }
        }
    }

    #[test]
    fn process_dev_exec_missing_binary_has_no_stdout_or_fallback() {
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(DEV_DIR)).unwrap();
        fs::write(tmp.path().join(DEV_DIR).join(ENABLED_MARKER), "").unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["routing_process_probe", "--nocapture", "--quiet"])
            .current_dir(tmp.path())
            .env("ASKHUMAN_ROUTING_TEST_CHILD", "exec")
            .env_remove(ASKHUMAN_HOME_ENV)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("binary is missing"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("resolved-home="));
    }

    #[test]
    fn process_direct_dev_binary_binds_home_without_marker() {
        let tmp = TestDir::new();
        let bin = instance_bin(tmp.path());
        fs::create_dir_all(bin.parent().unwrap()).unwrap();
        fs::copy(std::env::current_exe().unwrap(), &bin).unwrap();
        let output = std::process::Command::new(&bin)
            .args(["routing_process_probe", "--nocapture"])
            .current_dir(std::env::temp_dir())
            .env("ASKHUMAN_ROUTING_TEST_CHILD", "direct")
            .env_remove(ASKHUMAN_HOME_ENV)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(&format!(
                "resolved-home={}",
                instance_home(tmp.path()).display()
            )),
            "{stdout}"
        );
        assert!(stdout.contains("keychain=1"), "{stdout}");
    }

    #[test]
    fn instance_context_follows_home_env_or_exe_path() {
        let ordinary = Path::new("/usr/local/bin/AskHuman");
        let in_worktree = Path::new("/work/wt/.askhuman-dev/bin/AskHuman");
        let home = OsStr::new("/tmp/qa/home");

        // The case that rewrote the user's Stop hooks: an isolated home with an ordinary binary.
        assert!(is_dev_instance_context_for(Some(home), Some(ordinary)));
        // launchd restarts a daemon without the instance env; its path still gives it away.
        assert!(is_dev_instance_context_for(None, Some(in_worktree)));
        assert!(is_dev_instance_context_for(Some(home), Some(in_worktree)));
        // The production install: no instance env, ordinary path.
        assert!(!is_dev_instance_context_for(None, Some(ordinary)));
        // Nothing to go on: not treated as an instance.
        assert!(!is_dev_instance_context_for(None, None));
        // An empty value is "unset", like `paths::config_dir()`.
        assert!(!is_dev_instance_context_for(
            Some(OsStr::new("")),
            Some(ordinary)
        ));
        // Only a whole path segment counts, not a lookalike.
        for lookalike in [
            "/x/not.askhuman-dev/bin/AskHuman",
            "/x/.askhuman-dev-old/bin/AskHuman",
            "/x/askhuman-dev/bin/AskHuman",
        ] {
            assert!(!is_dev_instance_context_for(
                None,
                Some(Path::new(lookalike))
            ));
        }
    }

    #[test]
    fn find_dev_root_nearest_marker() {
        let tmp = TestDir::new();
        let root = tmp.path().join("wt");
        let nested = root.join("src").join("deep");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir_all(root.join(DEV_DIR)).unwrap();
        fs::write(root.join(DEV_DIR).join(ENABLED_MARKER), b"").unwrap();

        assert_eq!(find_dev_root(&nested).as_deref(), Some(root.as_path()));
        assert_eq!(find_dev_root(&root).as_deref(), Some(root.as_path()));
        assert!(find_dev_root(tmp.path()).is_none());
    }

    #[test]
    fn classify_command_matrix() {
        let prog = |rest: &[&str]| {
            let mut v = vec!["AskHuman".into()];
            v.extend(rest.iter().map(|s| (*s).to_string()));
            v
        };
        assert_eq!(classify_command(&prog(&["--popup"])), CommandClass::Skip);
        assert_eq!(classify_command(&prog(&["--gui-host"])), CommandClass::Skip);
        assert_eq!(
            classify_command(&prog(&["__update-worker"])),
            CommandClass::Skip
        );
        assert_eq!(
            classify_command(&prog(&["__npm-update-worker"])),
            CommandClass::Skip
        );
        assert_eq!(
            classify_command(&prog(&["__permission-diff-worker"])),
            CommandClass::Skip
        );
        assert_eq!(
            classify_command(&prog(&["dev", "enable"])),
            CommandClass::Dev
        );
        assert_eq!(classify_command(&prog(&["--version"])), CommandClass::Meta);
        assert_eq!(
            classify_command(&prog(&["--settings"])),
            CommandClass::Config
        );
        assert_eq!(classify_command(&prog(&["--todos"])), CommandClass::Config);
        assert_eq!(
            classify_command(&prog(&["channel", "list"])),
            CommandClass::Config
        );
        assert_eq!(
            classify_command(&prog(&["daemon", "status"])),
            CommandClass::Runtime
        );
        assert_eq!(classify_command(&prog(&["hello?"])), CommandClass::Runtime);
    }
}
