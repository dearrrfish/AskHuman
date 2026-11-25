//! Keep the installation entry distinct from the identity of the running image.
use std::{io, path::Path, path::PathBuf, process::Command, sync::OnceLock};

/// A one-shot handoff between native AskHuman launchers, never a user configuration override.
pub(crate) const INTERNAL_LAUNCH_ENTRY_ENV: &str = "ASKHUMAN_INTERNAL_LAUNCH_ENTRY";

/// Explicit public profile entrypoint override (e.g. from a Nix wrapper or Home Manager activation).
pub const PUBLIC_EXECUTABLE_ENV: &str = "ASKHUMAN_PUBLIC_EXECUTABLE";

static ENTRY: OnceLock<Result<Entry, String>> = OnceLock::new();

#[derive(Debug)]
enum Entry {
    Direct(PathBuf),
    #[cfg(unix)]
    Nix(nix::Binding),
}

impl Entry {
    fn program(&self, _persistent: bool) -> io::Result<PathBuf> {
        match self {
            Self::Direct(path) => Ok(path.clone()),
            #[cfg(unix)]
            Self::Nix(binding) => binding.program(_persistent),
        }
    }

    fn public_program(&self) -> io::Result<PathBuf> {
        match self {
            Self::Direct(path) => Ok(path.clone()),
            #[cfg(unix)]
            Self::Nix(binding) => binding.public_program(),
        }
    }

    fn image(&self) -> io::Result<PathBuf> {
        match self {
            Self::Direct(path) => Ok(path.clone()),
            #[cfg(unix)]
            Self::Nix(binding) => Ok(binding.resolve()?.image),
        }
    }

    fn is_nix(&self) -> bool {
        match self {
            Self::Direct(_) => false,
            #[cfg(unix)]
            Self::Nix(_) => true,
        }
    }
}

fn capture() -> io::Result<Entry> {
    // Consume even an invalid handoff before GTK, Tokio, Dev re-exec, or external children exist.
    let inherited = take_launch_entry();
    let role = std::env::args_os().nth(1);
    let running = std::env::current_exe()?;
    #[cfg(unix)]
    {
        let store = Path::new(option_env!("ASKHUMAN_NIX_STORE_DIR").unwrap_or("/nix/store"));
        if nix::should_bind_nix_entry(
            &running,
            store,
            option_env!("ASKHUMAN_NIX_STORE_DIR").is_some(),
        ) {
            let (uid, home, user) = nix::account()?;
            let candidates = nix::profile_candidates(&home, &user);
            let public_override = std::env::var_os(PUBLIC_EXECUTABLE_ENV);
            return nix::Binding::capture(
                &running,
                store,
                public_override.as_deref().map(Path::new),
                &candidates,
                uid,
                Path::new("/"),
            )
            .map(Entry::Nix);
        }
    }
    #[cfg(target_os = "linux")]
    {
        Ok(Entry::Direct(select_launch_path(
            running,
            role.as_deref(),
            inherited.as_deref(),
        )))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (role, inherited);
        Ok(Entry::Direct(running))
    }
}

fn entry() -> io::Result<&'static Entry> {
    ENTRY
        .get_or_init(|| capture().map_err(|error| error.to_string()))
        .as_ref()
        .map_err(|error| io::Error::other(error.clone()))
}

/// Capture before dispatch: after an ELF exchange `current_exe()` points at the retained image.
/// `launch_path` also lazily captures for tests and library callers; production calls this from
/// `main` before any potential exchange.
pub fn capture_launch_path() {
    let _ = entry();
}

/// Whether the running process was captured as a Nix store entry.
#[allow(dead_code)]
pub fn is_nix_entry() -> bool {
    entry().is_ok_and(Entry::is_nix)
}

/// Program for an independent successor. A bound Nix profile is revalidated on every use.
pub fn launch_path() -> io::Result<PathBuf> {
    entry()?.program(false)
}

/// Stable entry for login items. Nix store generations alone are not persistent entries.
#[cfg(unix)]
#[allow(dead_code)]
pub fn persistent_launch_path() -> io::Result<PathBuf> {
    entry()?.program(true)
}

/// Public entry for integrations (hooks, MCP configs, Pi extension, Antigravity plugin).
/// Uses `ASKHUMAN_PUBLIC_EXECUTABLE` or a bound profile symlink when available, falling back to
/// the package wrapper (`bin/AskHuman`), never `.AskHuman-wrapped`.
#[allow(dead_code)]
pub fn public_program_path() -> io::Result<PathBuf> {
    entry()?.public_program()
}

/// Disk image for update detection, never the wrapper script or the IPC process identity.
#[allow(dead_code)]
pub fn image_path() -> io::Result<PathBuf> {
    entry()?.image()
}

/// Resolve Nix `makeWrapper` binaries (`bin/AskHuman` -> `bin/.AskHuman-wrapped`) to the underlying
/// native binary when a sibling wrapped executable exists. Returns `None` for ordinary non-wrapper
/// binaries so callers like `binhash.json` keep their original path key across platforms.
#[allow(dead_code)]
pub fn unwrap_nix_wrapper_if_present(path: &Path) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let (parent, name) = (
            canonical.parent()?,
            canonical.file_name().and_then(|n| n.to_str())?,
        );
        if !name.is_empty() && !name.starts_with('.') && !name.ends_with("-wrapped") {
            for candidate in [
                parent.join(format!(".{name}-wrapped")),
                parent.join(format!("{name}-wrapped")),
            ] {
                if std::fs::metadata(&candidate).is_ok_and(|meta| meta.is_file()) {
                    return Some(candidate);
                }
            }
        }
        None
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

/// Resolve Nix `makeWrapper` binaries (`bin/AskHuman` -> `bin/.AskHuman-wrapped`) to the underlying
/// ELF binary so content fingerprints and install diagnostics compare the same native executable.
#[allow(dead_code)]
pub fn unwrap_nix_wrapper(path: &Path) -> PathBuf {
    unwrap_nix_wrapper_if_present(path)
        .unwrap_or_else(|| std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
}

/// Whether two configured/expected program paths refer to the same installation target.
///
/// Exact matches always return `true`. On Unix, if both are absolute and at least one is a Nix
/// store or Nix profile path, they also match when both canonicalize/unwrap to the same package
/// wrapper or ELF image inside the Nix store.
#[allow(dead_code)]
#[cfg(test)]
pub(crate) fn same_program_target_in(configured: &str, expected: &str, store: &Path) -> bool {
    same_program_target_with_store(configured, expected, store)
}

#[allow(dead_code)]
pub fn same_program_target(configured: &str, expected: &str) -> bool {
    let store = Path::new(option_env!("ASKHUMAN_NIX_STORE_DIR").unwrap_or("/nix/store"));
    same_program_target_with_store(configured, expected, store)
}

#[allow(dead_code)]
fn same_program_target_with_store(configured: &str, expected: &str, store: &Path) -> bool {
    if configured == expected {
        return true;
    }
    #[cfg(unix)]
    {
        let left = Path::new(configured);
        let right = Path::new(expected);
        if !left.is_absolute() || !right.is_absolute() {
            return false;
        }
        let Ok(canon_left) = std::fs::canonicalize(left) else {
            return false;
        };
        let Ok(canon_right) = std::fs::canonicalize(right) else {
            return false;
        };
        let canon_store = std::fs::canonicalize(store).unwrap_or_else(|_| store.to_path_buf());
        let resolved_left = unwrap_nix_wrapper(&canon_left);
        let resolved_right = unwrap_nix_wrapper(&canon_right);
        if resolved_left != resolved_right {
            return false;
        }
        let in_store = nix::package_root(&resolved_left, &canon_store).is_ok();
        let nix_profile_hint = is_nix_profile_path(left) || is_nix_profile_path(right);
        in_store || (nix_profile_hint && canon_left.starts_with(&canon_store))
    }
    #[cfg(not(unix))]
    {
        let _ = store;
        false
    }
}

#[cfg(unix)]
#[allow(dead_code)]
fn is_nix_profile_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.starts_with("/etc/profiles/per-user/")
        || s.starts_with("/run/current-system/sw/")
        || s.contains("/.nix-profile/")
        || s.contains("/.local/state/nix/profiles/")
}

fn take_launch_entry() -> Option<std::ffi::OsString> {
    let inherited = std::env::var_os(INTERNAL_LAUNCH_ENTRY_ENV);
    std::env::remove_var(INTERNAL_LAUNCH_ENTRY_ENV);
    inherited
}

#[cfg(any(target_os = "linux", test))]
fn select_launch_path(
    running: PathBuf,
    role: Option<&std::ffi::OsStr>,
    inherited: Option<&std::ffi::OsStr>,
) -> PathBuf {
    let internal_role = role.is_some_and(|role| {
        ["--popup", "--popup-host", "--gui-host"]
            .iter()
            .any(|expected| role == std::ffi::OsStr::new(expected))
    });
    if let Some(entry) = inherited.map(Path::new).filter(|entry| {
        internal_role
            && entry.is_absolute()
            && !entry
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
            && (*entry == running || retained_entry_matches(&running, entry))
    }) {
        entry.to_path_buf()
    } else {
        running
    }
}

#[cfg(any(target_os = "linux", test))]
fn retained_entry_matches(running: &Path, entry: &Path) -> bool {
    let Some(directory) = running.parent() else {
        return false;
    };
    let retained = directory
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix(".askhuman-install-"))
        .and_then(|suffix| suffix.split_once('-'))
        .is_some_and(|(pid, nonce)| {
            !pid.is_empty()
                && !nonce.is_empty()
                && pid.bytes().all(|byte| byte.is_ascii_digit())
                && nonce.bytes().all(|byte| byte.is_ascii_digit())
        });
    // The publisher retains every old image as AskHuman, even when the entry is a symlink to
    // an installation target with another basename. A different build at the entry is expected.
    retained
        && running.file_name() == Some(std::ffi::OsStr::new("AskHuman"))
        && entry.file_name().is_some()
        && entry.parent() == directory.parent()
}

/// Attach the canonical entry while choosing the running image independently in Command::new.
#[allow(dead_code)]
pub(crate) fn inherit_launch_entry(command: &mut Command) -> io::Result<()> {
    crate::development::routing::guard_fixture_program(command.get_program())?;
    #[cfg(target_os = "linux")]
    set_launch_entry(command, &launch_path()?);
    #[cfg(not(target_os = "linux"))]
    clear_launch_context(command);
    Ok(())
}

/// An independent launch uses its explicit target as the child's canonical entry.
#[allow(dead_code)]
pub(crate) fn set_launch_entry(command: &mut Command, entry: &Path) {
    clear_launch_context(command);
    #[cfg(target_os = "linux")]
    command.env(INTERNAL_LAUNCH_ENTRY_ENV, entry);
    #[cfg(not(target_os = "linux"))]
    let _ = entry;
}

/// Dev re-exec chooses a different installation and must not carry another instance's context.
pub(crate) fn clear_launch_context(command: &mut Command) {
    command.env_remove(INTERNAL_LAUNCH_ENTRY_ENV);
}

#[cfg(unix)]
mod nix {
    use std::fs;
    use std::io;
    use std::os::unix::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    extern "C" {
        fn geteuid() -> u32;
    }

    fn invalid(message: &str) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, message)
    }

    pub(super) struct Metadata {
        pub(super) version: u32,
        pub(super) family: String,
        pub(super) wrapper: PathBuf,
        pub(super) image: PathBuf,
    }

    #[derive(Debug)]
    pub(super) struct Package {
        pub(super) wrapper: PathBuf,
        pub(super) image: PathBuf,
    }

    #[derive(Debug)]
    pub(super) struct Binding {
        store: PathBuf,
        package: PathBuf,
        public_override: Option<PathBuf>,
        profile: Option<PathBuf>,
        uid: u32,
        trust_root: PathBuf,
    }

    fn is_nix_base32_hash(hash: &str) -> bool {
        hash.len() == 32
            && hash
                .bytes()
                .all(|b| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&b))
    }

    pub(super) fn package_root(path: &Path, store: &Path) -> io::Result<PathBuf> {
        let suffix = path
            .strip_prefix(store)
            .map_err(|_| invalid("Nix entry: target is outside the compiled store"))?;
        let mut components = suffix.components();
        let Some(Component::Normal(output)) = components.next() else {
            return Err(invalid("Nix entry: missing store output"));
        };
        let output_str = output
            .to_str()
            .ok_or_else(|| invalid("Nix entry: invalid store package name"))?;
        let Some((hash, name)) = output_str.split_once('-') else {
            return Err(invalid("Nix entry: invalid store package component"));
        };
        if !is_nix_base32_hash(hash) || name.is_empty() {
            return Err(invalid("Nix entry: invalid store package hash"));
        }
        if suffix
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(invalid("Nix entry: parent traversal in store path"));
        }
        Ok(store.join(output))
    }

    pub(super) fn should_bind_nix_entry(
        running: &Path,
        store: &Path,
        compiled_nix_store: bool,
    ) -> bool {
        let Ok(root) = package_root(running, store) else {
            return false;
        };
        if compiled_nix_store {
            return true;
        }
        if fs::symlink_metadata(root.join("share/askhuman/entry-v1.json")).is_ok() {
            return true;
        }
        matches!(
            running.strip_prefix(&root).ok().and_then(|p| p.to_str()),
            Some("bin/.AskHuman-wrapped" | "bin/AskHuman-wrapped" | "bin/AskHuman")
        )
    }

    /// Get account identity from the OS (`/etc/passwd` keyed by `geteuid()`), falling back to
    /// validated environment only when `/etc/passwd` lacks an entry for the effective UID.
    pub(super) fn account() -> io::Result<(u32, PathBuf, String)> {
        let uid = unsafe { geteuid() };
        if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
            let uid_str = uid.to_string();
            for line in passwd.lines() {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let mut fields = line.split(':');
                let Some(user) = fields.next() else { continue };
                let _passwd = fields.next();
                let Some(line_uid) = fields.next() else {
                    continue;
                };
                if line_uid != uid_str {
                    continue;
                }
                let _gid = fields.next();
                let _gecos = fields.next();
                let Some(home) = fields.next() else { continue };
                if !user.is_empty()
                    && !user.contains('/')
                    && user != "."
                    && user != ".."
                    && !home.is_empty()
                    && Path::new(home).is_absolute()
                {
                    return Ok((uid, PathBuf::from(home), user.to_string()));
                }
            }
        }
        let user = std::env::var("USER")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|u| !u.is_empty() && !u.contains('/') && u != "." && u != "..")
            .ok_or_else(|| invalid("Nix entry: account lookup failed"))?;
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|h| h.is_absolute())
            .ok_or_else(|| invalid("Nix entry: account home lookup failed"))?;
        Ok((uid, home, user))
    }

    pub(super) fn profile_candidates(home: &Path, user: &str) -> Vec<PathBuf> {
        vec![
            home.join(".nix-profile/bin/AskHuman"),
            home.join(".local/state/nix/profiles/profile/bin/AskHuman"),
            Path::new("/etc/profiles/per-user")
                .join(user)
                .join("bin/AskHuman"),
            PathBuf::from("/run/current-system/sw/bin/AskHuman"),
        ]
    }

    fn relative(path: &Path) -> bool {
        !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
    }

    fn no_parent_traversal(path: &Path) -> bool {
        path.is_absolute()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
    }

    type JsonChars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

    fn skip_ws(c: &mut JsonChars<'_>) {
        while c.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
            c.next();
        }
    }

    fn parse_json_string(c: &mut JsonChars<'_>) -> Result<String, String> {
        if c.next() != Some('"') {
            return Err("expected string".into());
        }
        let mut out = String::new();
        loop {
            match c.next() {
                Some('"') => return Ok(out),
                Some('\\') => match c.next() {
                    Some(esc @ ('"' | '\\' | '/')) => out.push(esc),
                    Some('b') => out.push('\u{0008}'),
                    Some('f') => out.push('\u{000c}'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => {
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            hex.push(c.next().ok_or("truncated unicode escape")?);
                        }
                        let code =
                            u32::from_str_radix(&hex, 16).map_err(|_| "invalid unicode escape")?;
                        out.push(char::from_u32(code).ok_or("invalid unicode scalar")?);
                    }
                    _ => return Err("unsupported JSON escape".into()),
                },
                Some(ch) if ch >= ' ' => out.push(ch),
                _ => return Err("unterminated or invalid JSON string".into()),
            }
        }
    }

    fn parse_json_u32(c: &mut JsonChars<'_>) -> Result<u32, String> {
        let mut digits = String::new();
        while c.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            digits.push(c.next().unwrap());
        }
        if digits.is_empty() {
            return Err("expected integer".into());
        }
        digits.parse::<u32>().map_err(|e| e.to_string())
    }

    fn skip_json_number(c: &mut JsonChars<'_>) -> Result<(), String> {
        if c.peek() == Some(&'-') {
            c.next();
        }
        match c.next() {
            Some('0') => {}
            Some('1'..='9') => {
                while c.peek().is_some_and(|d| d.is_ascii_digit()) {
                    c.next();
                }
            }
            _ => return Err("invalid JSON number".into()),
        }
        if c.peek() == Some(&'.') {
            c.next();
            if !c.peek().is_some_and(|d| d.is_ascii_digit()) {
                return Err("invalid JSON fraction".into());
            }
            while c.peek().is_some_and(|d| d.is_ascii_digit()) {
                c.next();
            }
        }
        if matches!(c.peek(), Some('e' | 'E')) {
            c.next();
            if matches!(c.peek(), Some('+' | '-')) {
                c.next();
            }
            if !c.peek().is_some_and(|d| d.is_ascii_digit()) {
                return Err("invalid JSON exponent".into());
            }
            while c.peek().is_some_and(|d| d.is_ascii_digit()) {
                c.next();
            }
        }
        Ok(())
    }

    fn skip_json_literal(c: &mut JsonChars<'_>, expected: &str) -> Result<(), String> {
        for ch in expected.chars() {
            if c.next() != Some(ch) {
                return Err(format!("expected `{expected}`"));
            }
        }
        Ok(())
    }

    fn skip_json_value(c: &mut JsonChars<'_>, depth: usize) -> Result<(), String> {
        if depth > 32 {
            return Err("JSON nesting too deep".into());
        }
        skip_ws(c);
        match c.peek() {
            Some('"') => {
                let _ = parse_json_string(c)?;
                Ok(())
            }
            Some('-') | Some('0'..='9') => skip_json_number(c),
            Some('t') => skip_json_literal(c, "true"),
            Some('f') => skip_json_literal(c, "false"),
            Some('n') => skip_json_literal(c, "null"),
            Some('[') => {
                c.next();
                skip_ws(c);
                if c.peek() == Some(&']') {
                    c.next();
                    return Ok(());
                }
                loop {
                    skip_json_value(c, depth + 1)?;
                    skip_ws(c);
                    match c.next() {
                        Some(',') => continue,
                        Some(']') => return Ok(()),
                        _ => return Err("expected ',' or ']'".into()),
                    }
                }
            }
            Some('{') => {
                c.next();
                skip_ws(c);
                if c.peek() == Some(&'}') {
                    c.next();
                    return Ok(());
                }
                loop {
                    skip_ws(c);
                    let _ = parse_json_string(c)?;
                    skip_ws(c);
                    if c.next() != Some(':') {
                        return Err("expected ':'".into());
                    }
                    skip_json_value(c, depth + 1)?;
                    skip_ws(c);
                    match c.next() {
                        Some(',') => continue,
                        Some('}') => return Ok(()),
                        _ => return Err("expected ',' or '}'".into()),
                    }
                }
            }
            _ => Err("unsupported JSON value".into()),
        }
    }

    /// Minimal pure-std JSON parser for `share/askhuman/entry-v1.json` (kept dependency-free so
    /// standalone `rustc` checks like `scripts/install-linux.node-test.mjs` compile directly).
    pub(super) fn parse_metadata(bytes: &[u8]) -> Result<Metadata, String> {
        let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        let mut chars = text.chars().peekable();

        skip_ws(&mut chars);
        if chars.next() != Some('{') {
            return Err("expected JSON object".into());
        }
        let mut version = None;
        let mut family = None;
        let mut wrapper = None;
        let mut image = None;
        skip_ws(&mut chars);
        if chars.peek() != Some(&'}') {
            loop {
                skip_ws(&mut chars);
                let key = parse_json_string(&mut chars)?;
                skip_ws(&mut chars);
                if chars.next() != Some(':') {
                    return Err("expected ':'".into());
                }
                skip_ws(&mut chars);
                match key.as_str() {
                    "version" => version = Some(parse_json_u32(&mut chars)?),
                    "family" => family = Some(parse_json_string(&mut chars)?),
                    "wrapper" => wrapper = Some(PathBuf::from(parse_json_string(&mut chars)?)),
                    "image" => image = Some(PathBuf::from(parse_json_string(&mut chars)?)),
                    _ => skip_json_value(&mut chars, 0)?,
                }
                skip_ws(&mut chars);
                match chars.next() {
                    Some(',') => continue,
                    Some('}') => break,
                    _ => return Err("expected ',' or '}'".into()),
                }
            }
        } else {
            chars.next();
        }
        skip_ws(&mut chars);
        if chars.next().is_some() {
            return Err("trailing characters after JSON object".into());
        }
        Ok(Metadata {
            version: version.ok_or("missing version")?,
            family: family.ok_or("missing family")?,
            wrapper: wrapper.ok_or("missing wrapper")?,
            image: image.ok_or("missing image")?,
        })
    }

    /// Whether UID 0 is mapped in the current Linux user namespace. On real hosts this is true;
    /// inside an unprivileged Nix build sandbox only `nixbld` is mapped and host-root store paths
    /// appear with the kernel overflowuid (65534).
    fn uid_zero_mapped() -> bool {
        let Ok(text) = fs::read_to_string("/proc/self/uid_map") else {
            return true;
        };
        text.lines().any(|line| {
            let mut parts = line.split_whitespace();
            let inside = parts.next().and_then(|s| s.parse::<u32>().ok());
            let _outside = parts.next();
            let count = parts.next().and_then(|s| s.parse::<u32>().ok());
            inside == Some(0) && count.is_some_and(|c| c > 0)
        })
    }

    /// Validate every directory and symlink traversed, including symlink target ancestors.
    /// The injected root lets fixtures exercise the same checks without trusting /tmp.
    fn trusted(path: &Path, uid: u32, root: &Path) -> io::Result<PathBuf> {
        let root_mapped = uid_zero_mapped();
        fn walk(
            path: &Path,
            uid: u32,
            root: &Path,
            root_mapped: bool,
            hops: usize,
        ) -> io::Result<PathBuf> {
            if hops > 40 {
                return Err(invalid("Nix entry: too many symlinks"));
            }
            let suffix = path
                .strip_prefix(root)
                .map_err(|_| invalid("Nix entry: path outside trusted root"))?;
            let mut resolved = root.to_path_buf();
            let check = |path: &Path| -> io::Result<fs::Metadata> {
                let meta = fs::symlink_metadata(path)?;
                let trusted_owner =
                    meta.uid() == 0 || meta.uid() == uid || (!root_mapped && meta.uid() == 65534);
                // The root-owned sticky store directory can be writable by nixbld. Sticky
                // semantics protect existing entries; every child still needs a trusted owner.
                let sticky_root_dir = meta.is_dir()
                    && meta.mode() & 0o1000 != 0
                    && (meta.uid() == 0 || (!root_mapped && trusted_owner));
                if !trusted_owner
                    || (!meta.file_type().is_symlink()
                        && meta.mode() & 0o022 != 0
                        && !sticky_root_dir)
                {
                    return Err(invalid("Nix entry: unsafe owner or write permissions"));
                }
                Ok(meta)
            };
            check(root)?;
            for component in suffix.components() {
                match component {
                    Component::Normal(part) => resolved.push(part),
                    Component::CurDir => continue,
                    Component::ParentDir => {
                        if resolved == root {
                            return Err(invalid("Nix entry: path escapes trusted root"));
                        }
                        resolved.pop();
                        continue;
                    }
                    _ => return Err(invalid("Nix entry: invalid path component")),
                }
                let meta = check(&resolved)?;
                if meta.file_type().is_symlink() {
                    let target = fs::read_link(&resolved)?;
                    let target = if target.is_absolute() {
                        target
                    } else {
                        resolved.parent().unwrap().join(target)
                    };
                    resolved = walk(&target, uid, root, root_mapped, hops + 1)?;
                }
            }
            Ok(resolved)
        }
        walk(path, uid, root, root_mapped, 0)
    }

    fn verify_executable_in_root(path: &Path, root: &Path) -> io::Result<()> {
        let meta = fs::metadata(path)?;
        if !path.starts_with(root) || !meta.is_file() || meta.mode() & 0o111 == 0 {
            return Err(invalid(
                "Nix entry: executable escapes package or is not executable",
            ));
        }
        Ok(())
    }

    fn package(root: &Path, uid: u32, trust_root: &Path) -> io::Result<Package> {
        let metadata_candidate = root.join("share/askhuman/entry-v1.json");
        if fs::symlink_metadata(&metadata_candidate).is_ok() {
            let metadata_path = trusted(&metadata_candidate, uid, trust_root)?;
            if !metadata_path.starts_with(root) {
                return Err(invalid("Nix entry: metadata escapes package"));
            }
            let metadata = parse_metadata(&fs::read(metadata_path)?)
                .map_err(|error| invalid(&format!("Nix entry: invalid metadata: {error}")))?;
            if metadata.version != 1
                || metadata.family != "askhuman"
                || metadata.wrapper != Path::new("bin/AskHuman")
                || !relative(&metadata.image)
            {
                return Err(invalid("Nix entry: incompatible package metadata"));
            }
            let wrapper = trusted(&root.join(metadata.wrapper), uid, trust_root)?;
            let image = trusted(&root.join(metadata.image), uid, trust_root)?;
            verify_executable_in_root(&wrapper, root)?;
            verify_executable_in_root(&image, root)?;
            return Ok(Package { wrapper, image });
        }

        // Fallback automatic detection for wrapped or unwrapped Nix store packages without entry-v1.json.
        let wrapper = trusted(&root.join("bin/AskHuman"), uid, trust_root)?;
        verify_executable_in_root(&wrapper, root)?;
        let image = [
            root.join("bin/.AskHuman-wrapped"),
            root.join("bin/AskHuman-wrapped"),
        ]
        .into_iter()
        .find(|candidate| fs::symlink_metadata(candidate).is_ok())
        .map(|candidate| {
            let resolved = trusted(&candidate, uid, trust_root)?;
            verify_executable_in_root(&resolved, root)?;
            Ok::<PathBuf, io::Error>(resolved)
        })
        .transpose()?
        .unwrap_or_else(|| wrapper.clone());
        Ok(Package { wrapper, image })
    }

    impl Binding {
        pub(super) fn capture(
            image: &Path,
            store: &Path,
            public_override: Option<&Path>,
            candidates: &[PathBuf],
            uid: u32,
            trust_root: &Path,
        ) -> io::Result<Self> {
            let root = package_root(image, store)?;
            let image = trusted(image, uid, trust_root)?;
            let initial = package(&root, uid, trust_root)?;
            if initial.image != image && initial.wrapper != image {
                return Err(invalid(
                    "Nix entry: metadata does not identify the running image",
                ));
            }
            let valid_override = public_override
                .filter(|candidate| no_parent_traversal(candidate))
                .and_then(|candidate| {
                    let target = trusted(candidate, uid, trust_root).ok()?;
                    (target == initial.wrapper || target == initial.image)
                        .then(|| candidate.to_path_buf())
                });
            let mut profile = valid_override
                .as_ref()
                .filter(|candidate| !candidate.starts_with(store))
                .cloned();
            if profile.is_none() {
                profile = candidates
                    .iter()
                    .find(|candidate| {
                        trusted(candidate, uid, trust_root).is_ok_and(|target| {
                            target == initial.wrapper || target == initial.image
                        })
                    })
                    .cloned();
            }
            let public_override = valid_override.map(|candidate| {
                if candidate.starts_with(store) {
                    initial.wrapper.clone()
                } else {
                    candidate
                }
            });
            Ok(Self {
                store: store.to_path_buf(),
                package: root,
                public_override,
                profile,
                uid,
                trust_root: trust_root.to_path_buf(),
            })
        }

        pub(super) fn resolve(&self) -> io::Result<Package> {
            if let Some(profile) = &self.profile {
                let target = trusted(profile, self.uid, &self.trust_root)?;
                let root = package_root(&target, &self.store)?;
                let current = package(&root, self.uid, &self.trust_root)?;
                if target != current.wrapper && target != current.image {
                    return Err(invalid("Nix entry: profile is not the package wrapper"));
                }
                Ok(current)
            } else {
                package(&self.package, self.uid, &self.trust_root)
            }
        }

        pub(super) fn program(&self, persistent: bool) -> io::Result<PathBuf> {
            let current = self.resolve()?;
            match &self.profile {
                Some(profile) => Ok(profile.clone()),
                None if persistent => Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Nix login startup requires a trusted stable AskHuman profile entry; install this package in a user or system profile first",
                )),
                None => Ok(self
                    .public_override
                    .clone()
                    .unwrap_or(current.wrapper)),
            }
        }

        pub(super) fn public_program(&self) -> io::Result<PathBuf> {
            let current = self.resolve()?;
            if let Some(override_path) = &self.public_override {
                return Ok(override_path.clone());
            }
            if let Some(profile) = &self.profile {
                return Ok(profile.clone());
            }
            Ok(current.wrapper)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::same_program_target_in;
        use super::*;
        use std::os::unix::fs::{symlink, PermissionsExt};

        struct Fixture {
            dir: tempfile::TempDir,
            store: PathBuf,
            profile: PathBuf,
        }

        impl Fixture {
            fn new() -> Self {
                let dir = tempfile::tempdir().unwrap();
                let store = dir.path().join("store");
                let profile = dir.path().join("profile/bin/AskHuman");
                fs::create_dir_all(profile.parent().unwrap()).unwrap();
                Self {
                    dir,
                    store,
                    profile,
                }
            }

            fn pkg_dir(&self, suffix: &str) -> PathBuf {
                self.store
                    .join(format!("0123456789abcdfghijklmnpqrsvwxyz-{suffix}"))
            }

            fn package(&self, suffix: &str) -> PathBuf {
                let root = self.pkg_dir(suffix);
                fs::create_dir_all(root.join("bin")).unwrap();
                fs::create_dir_all(root.join("share/askhuman")).unwrap();
                fs::write(root.join("bin/AskHuman"), "#!/bin/sh\nexit 0\n").unwrap();
                fs::write(root.join("bin/.AskHuman-wrapped"), suffix).unwrap();
                for name in ["AskHuman", ".AskHuman-wrapped"] {
                    fs::set_permissions(
                        root.join("bin").join(name),
                        fs::Permissions::from_mode(0o755),
                    )
                    .unwrap();
                }
                self.metadata(&root, "askhuman", "bin/.AskHuman-wrapped");
                root
            }

            fn metadata(&self, root: &Path, family: &str, image: &str) {
                fs::write(
                    root.join("share/askhuman/entry-v1.json"),
                    format!(
                        r#"{{"version":1,"family":"{family}","wrapper":"bin/AskHuman","image":"{image}"}}"#
                    ),
                )
                .unwrap();
            }

            fn switch(&self, root: &Path) {
                let _ = fs::remove_file(&self.profile);
                symlink(root.join("bin/AskHuman"), &self.profile).unwrap();
            }

            fn bind(&self, root: &Path) -> io::Result<Binding> {
                self.bind_with_override(root, None)
            }

            fn bind_with_override(
                &self,
                root: &Path,
                public_override: Option<&Path>,
            ) -> io::Result<Binding> {
                Binding::capture(
                    &root.join("bin/.AskHuman-wrapped"),
                    &self.store,
                    public_override,
                    std::slice::from_ref(&self.profile),
                    unsafe { geteuid() },
                    self.dir.path(),
                )
            }
        }

        #[test]
        fn metadata_rejects_escape_and_wrong_family() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            for image in ["../outside", "/bin/sh", "bin/../../outside"] {
                f.metadata(&a, "askhuman", image);
                assert!(f.bind(&a).is_err());
            }
            f.metadata(&a, "another-app", "bin/.AskHuman-wrapped");
            assert!(f.bind(&a).is_err());
            f.metadata(&a, "askhuman", "bin/.AskHuman-wrapped");
            fs::remove_file(a.join("bin/.AskHuman-wrapped")).unwrap();
            let b = f.package("askhuman-b");
            symlink(
                b.join("bin/.AskHuman-wrapped"),
                a.join("bin/.AskHuman-wrapped"),
            )
            .unwrap();
            assert!(f.bind(&a).is_err());
        }

        #[test]
        fn wrapped_package_without_entry_metadata_auto_detects_wrapper_and_image() {
            let f = Fixture::new();
            let a = f.package("askhuman-no-meta");
            fs::remove_file(a.join("share/askhuman/entry-v1.json")).unwrap();
            f.switch(&a);
            let binding = f.bind(&a).unwrap();
            assert_eq!(binding.program(false).unwrap(), f.profile);
            assert_eq!(binding.program(true).unwrap(), f.profile);
            assert_eq!(
                binding.resolve().unwrap().image,
                a.join("bin/.AskHuman-wrapped")
            );
            assert_eq!(binding.resolve().unwrap().wrapper, a.join("bin/AskHuman"));

            // Also works without any matching profile symlink: launch_path is bin/AskHuman, never .AskHuman-wrapped.
            fs::remove_file(&f.profile).unwrap();
            let unbound = f.bind(&a).unwrap();
            assert_eq!(unbound.program(false).unwrap(), a.join("bin/AskHuman"));
            assert_eq!(unbound.public_program().unwrap(), a.join("bin/AskHuman"));
            assert!(unbound.program(true).is_err());
        }

        #[test]
        fn public_executable_env_override_honors_valid_paths_and_rejects_invalid() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            let b = f.package("askhuman-b");

            // Symlink pointing to wrapper is valid for both launch and persistent login.
            let custom_profile = f.dir.path().join("etc-profiles/bin/AskHuman");
            fs::create_dir_all(custom_profile.parent().unwrap()).unwrap();
            symlink(a.join("bin/AskHuman"), &custom_profile).unwrap();
            let bound = f.bind_with_override(&a, Some(&custom_profile)).unwrap();
            assert_eq!(bound.public_program().unwrap(), custom_profile);
            assert_eq!(bound.program(false).unwrap(), custom_profile);
            assert_eq!(bound.program(true).unwrap(), custom_profile);

            // Store wrapper override works for public_program/launch_path but not persistent login.
            let store_override = f
                .bind_with_override(&a, Some(&a.join("bin/AskHuman")))
                .unwrap();
            assert_eq!(
                store_override.public_program().unwrap(),
                a.join("bin/AskHuman")
            );
            assert_eq!(
                store_override.program(false).unwrap(),
                a.join("bin/AskHuman")
            );
            assert!(store_override.program(true).is_err());

            // Invalid overrides (relative, .., pointing to wrong package, nonexistent) fall back.
            let wrong_symlink = f.dir.path().join("wrong/bin/AskHuman");
            fs::create_dir_all(wrong_symlink.parent().unwrap()).unwrap();
            symlink(b.join("bin/AskHuman"), &wrong_symlink).unwrap();
            f.switch(&a);
            for invalid in [
                PathBuf::from("bin/AskHuman"),
                f.dir.path().join("etc-profiles/../bin/AskHuman"),
                f.dir.path().join("nonexistent/AskHuman"),
                wrong_symlink,
            ] {
                let fallback = f.bind_with_override(&a, Some(&invalid)).unwrap();
                assert_eq!(fallback.public_program().unwrap(), f.profile);
            }
        }

        #[test]
        fn profile_tracks_upgrade_and_rollback_without_changing_binding() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            let b = f.package("askhuman-b");
            f.switch(&a);
            let binding = f.bind(&a).unwrap();
            for root in [&a, &b, &a] {
                f.switch(root);
                assert_eq!(binding.program(false).unwrap(), f.profile);
                assert_eq!(binding.program(true).unwrap(), f.profile);
                assert_eq!(
                    binding.resolve().unwrap().image,
                    root.join("bin/.AskHuman-wrapped")
                );
            }
        }

        #[test]
        fn no_matching_profile_allows_wrapper_but_not_login_startup() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            let b = f.package("askhuman-b");
            let unbound = f.bind(&a).unwrap();
            assert_eq!(unbound.program(false).unwrap(), a.join("bin/AskHuman"));
            assert!(unbound.program(true).is_err());
            f.switch(&b);
            let binding = f.bind(&a).unwrap();
            assert_eq!(binding.program(false).unwrap(), a.join("bin/AskHuman"));
            assert_eq!(
                binding.program(true).unwrap_err().kind(),
                io::ErrorKind::Unsupported
            );
            f.switch(&a);
            assert!(binding.program(true).is_err());
        }

        #[test]
        fn invalid_bound_profile_never_falls_back_to_old_generation() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            let b = f.package("askhuman-b");
            f.switch(&a);
            let binding = f.bind(&a).unwrap();
            f.switch(&b);
            f.metadata(&b, "another-app", "bin/.AskHuman-wrapped");
            assert!(binding.program(false).is_err());
            assert!(binding.resolve().is_err());
            fs::remove_file(&f.profile).unwrap();
            assert!(binding.program(false).is_err());
            assert!(binding.program(true).is_err());
        }

        #[test]
        fn unsafe_profile_permissions_are_rejected() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            f.switch(&a);
            let binding = f.bind(&a).unwrap();
            fs::set_permissions(
                f.profile.parent().unwrap(),
                fs::Permissions::from_mode(0o777),
            )
            .unwrap();
            assert!(binding.resolve().is_err());
            assert!(f.bind(&a).unwrap().program(true).is_err());
        }

        #[test]
        fn darwin_can_use_the_same_wrapper_and_image() {
            let f = Fixture::new();
            let a = f.package("askhuman-darwin");
            f.metadata(&a, "askhuman", "bin/AskHuman");
            let binding = Binding::capture(
                &a.join("bin/AskHuman"),
                &f.store,
                None,
                &[],
                unsafe { geteuid() },
                f.dir.path(),
            )
            .unwrap();
            assert_eq!(binding.resolve().unwrap().image, a.join("bin/AskHuman"));
        }

        #[test]
        fn same_program_target_matches_nix_store_wrapper_and_profile_symlink() {
            let f = Fixture::new();
            let a = f.package("askhuman-a");
            let b = f.package("askhuman-b");
            f.switch(&a);
            let profile_str = f.profile.to_str().unwrap();
            let wrapper_str = a.join("bin/AskHuman").to_str().unwrap().to_string();
            let wrapped_str = a
                .join("bin/.AskHuman-wrapped")
                .to_str()
                .unwrap()
                .to_string();
            let other_wrapper = b.join("bin/AskHuman").to_str().unwrap().to_string();

            assert!(same_program_target_in(&wrapper_str, profile_str, &f.store));
            assert!(same_program_target_in(profile_str, &wrapper_str, &f.store));
            assert!(same_program_target_in(&wrapped_str, profile_str, &f.store));
            // Also matches when the store root traverses a symlink (e.g. macOS /var -> /private/var).
            let store_alias = f.dir.path().join("store-alias");
            symlink(&f.store, &store_alias).unwrap();
            assert!(same_program_target_in(
                &wrapper_str,
                profile_str,
                &store_alias
            ));
            assert!(!same_program_target_in(
                &other_wrapper,
                profile_str,
                &f.store
            ));
            // Non-Nix symlinks are not treated as equivalent.
            let non_nix_target = f.dir.path().join("ordinary/AskHuman");
            let non_nix_link = f.dir.path().join("ordinary/link");
            fs::create_dir_all(non_nix_target.parent().unwrap()).unwrap();
            fs::write(&non_nix_target, "bin").unwrap();
            symlink(&non_nix_target, &non_nix_link).unwrap();
            assert!(!same_program_target_in(
                non_nix_target.to_str().unwrap(),
                non_nix_link.to_str().unwrap(),
                &f.store
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn direct_keeps_the_installation_path_across_atomic_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AskHuman");
        std::fs::write(&path, b"old image").unwrap();
        let entry = Entry::Direct(path.clone());
        let replacement = dir.path().join("new");
        std::fs::write(&replacement, b"new image").unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert_eq!(entry.program(false).unwrap(), path);
        assert_eq!(entry.program(true).unwrap(), path);
        assert_eq!(entry.public_program().unwrap(), path);
        assert_eq!(std::fs::read(entry.image().unwrap()).unwrap(), b"new image");
    }

    #[test]
    fn retained_popup_roles_preserve_the_entry_without_changing_the_image() {
        let root = std::env::temp_dir().join("askhuman-entry-test");
        let image = root.join(".askhuman-install-12-345").join("AskHuman");
        // Symlink destinations can have a different basename from the public AskHuman entry.
        let entry = root.join("custom-name");
        for role in ["--popup", "--popup-host", "--gui-host"] {
            assert_eq!(
                select_launch_path(
                    image.clone(),
                    Some(OsStr::new(role)),
                    Some(entry.as_os_str())
                ),
                entry
            );
        }
        assert_eq!(image.file_name(), Some(OsStr::new("AskHuman")));
    }

    #[test]
    fn public_roles_and_unrelated_entries_cannot_override_the_launch_path() {
        let root = std::env::temp_dir().join("askhuman-entry-test");
        let image = root.join(".askhuman-install-12-345").join("AskHuman");
        let entry = root.join("AskHuman");
        for role in [None, Some("mcp"), Some("--settings"), Some("question")] {
            assert_eq!(
                select_launch_path(image.clone(), role.map(OsStr::new), Some(entry.as_os_str())),
                image
            );
        }
        for invalid in [
            PathBuf::from("AskHuman"),
            root.join("elsewhere").join("AskHuman"),
            root.join("..").join("AskHuman"),
        ] {
            let chosen = select_launch_path(
                image.clone(),
                Some(OsStr::new("--popup-host")),
                Some(invalid.as_os_str()),
            );
            assert_eq!(chosen, image, "must not redirect to {}", invalid.display());
        }
        for name in [".askhuman-install-x-123", ".askhuman-install-12-", "other"] {
            let unrelated = root.join(name).join("AskHuman");
            assert_eq!(
                select_launch_path(
                    unrelated.clone(),
                    Some(OsStr::new("--popup")),
                    Some(entry.as_os_str())
                ),
                unrelated
            );
        }
    }

    #[test]
    fn dev_reexec_clears_context_without_changing_the_selected_binary() {
        let selected = std::env::temp_dir()
            .join("other-dev-instance")
            .join("AskHuman");
        let mut command = Command::new(&selected);
        command.env(INTERNAL_LAUNCH_ENTRY_ENV, "another installation");
        clear_launch_context(&mut command);
        assert_eq!(command.get_program(), selected.as_os_str());
        assert!(command
            .get_envs()
            .any(|(key, value)| { key == INTERNAL_LAUNCH_ENTRY_ENV && value.is_none() }));
    }

    #[cfg(unix)]
    #[test]
    fn launch_entry_preserves_non_utf8_installation_paths() {
        use std::os::unix::ffi::OsStringExt;
        let root =
            std::env::temp_dir().join(std::ffi::OsString::from_vec(b"ah-entry-\xff".to_vec()));
        let image = root.join(".askhuman-install-12-345").join("AskHuman");
        let entry = root.join("AskHuman");
        assert_eq!(
            select_launch_path(image, Some(OsStr::new("--popup")), Some(entry.as_os_str())),
            entry
        );
    }

    #[test]
    fn launch_context_is_consumed_before_descendant_launches() {
        const MODE: &str = "ASKHUMAN_LAUNCH_CONTEXT_TEST";
        const TEST: &str =
            "executable::tests::launch_context_is_consumed_before_descendant_launches";
        match std::env::var(MODE).as_deref() {
            Ok("child") => {
                assert_eq!(take_launch_entry(), Some("one-shot".into()));
                assert!(std::env::var_os(INTERNAL_LAUNCH_ENTRY_ENV).is_none());
                let output = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture"])
                    .env(MODE, "grandchild")
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok("grandchild") => assert!(std::env::var_os(INTERNAL_LAUNCH_ENTRY_ENV).is_none()),
            _ => {
                let output = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture"])
                    .env(MODE, "child")
                    .env(INTERNAL_LAUNCH_ENTRY_ENV, "one-shot")
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn unwrap_nix_wrapper_maps_wrapper_and_symlinks_to_wrapped_binary() {
        let dir = tempfile::tempdir().unwrap();
        let store_bin = dir.path().join("store/pkg/bin");
        std::fs::create_dir_all(&store_bin).unwrap();
        let wrapper = store_bin.join("AskHuman");
        let wrapped = store_bin.join(".AskHuman-wrapped");
        std::fs::write(&wrapper, "#!/bin/sh\n").unwrap();
        std::fs::write(&wrapped, "elf-bytes").unwrap();

        let profile_bin = dir.path().join("profile/bin");
        std::fs::create_dir_all(&profile_bin).unwrap();
        let symlink_entry = profile_bin.join("AskHuman");
        std::os::unix::fs::symlink(&wrapper, &symlink_entry).unwrap();

        let expected = std::fs::canonicalize(&wrapped).unwrap();
        assert_eq!(unwrap_nix_wrapper(&wrapper), expected);
        assert_eq!(unwrap_nix_wrapper(&symlink_entry), expected);
        assert_eq!(unwrap_nix_wrapper(&wrapped), expected);

        // Plain binary without a sibling `.<name>-wrapped` stays as-is (canonicalized).
        let plain_dir = dir.path().join("plain");
        std::fs::create_dir_all(&plain_dir).unwrap();
        let plain = plain_dir.join("AskHuman");
        std::fs::write(&plain, "plain-elf").unwrap();
        assert_eq!(
            unwrap_nix_wrapper(&plain),
            std::fs::canonicalize(&plain).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn nix_parse_metadata_skips_arbitrary_json_fields() {
        let json = br#"{
            "futureFlag": true,
            "disabled": false,
            "nothing": null,
            "count": -42.5e+2,
            "tags": ["nix", 1, {"nested": [false, null, "ok"]}, []],
            "nested": {"a": 1, "b": {"c": "deep"}},
            "version": 1,
            "family": "askhuman",
            "wrapper": "bin/AskHuman",
            "image": "bin/.AskHuman-wrapped"
        }"#;
        let meta = super::nix::parse_metadata(json).expect("should skip unknown JSON values");
        assert_eq!(meta.version, 1);
        assert_eq!(meta.family, "askhuman");
        assert_eq!(meta.wrapper, PathBuf::from("bin/AskHuman"));
        assert_eq!(meta.image, PathBuf::from("bin/.AskHuman-wrapped"));
    }
}
