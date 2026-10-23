//! Popup-launch performance instrumentation (opt-in via `ASKHUMAN_PERF`).
//!
//! When enabled, each milestone across the four execution environments (CLI / daemon / GUI helper
//! / frontend) appends one TSV line to `~/.askhuman/perf.log`:
//!
//! ```text
//! <epoch_ms>\t<perf_id>\t<stage>\t<pid>
//! ```
//!
//! All lines for one invocation share the CLI-generated `perf_id`, so a harness
//! (`scripts/perf-popup.mjs`) can stitch a single timeline together and compute per-segment deltas.
//!
//! Gating is by **`perf_id` being non-empty**, not by reading `ASKHUMAN_PERF` at the write site —
//! the daemon is long-lived and was started without the env var, so it relies on the per-request
//! `perf_id` propagated through `TaskRequest`. The helper receives the id via `ASKHUMAN_PERF_ID`
//! (set by the daemon when spawning). Disabled by default → zero file IO, zero log noise.

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Process-start timestamp (epoch ms), captured as early as possible in `main` so `cli.start`
/// reflects the true process birth rather than the moment we discover perf is enabled.
static START_MS: AtomicU64 = AtomicU64::new(0);

/// Capture the process start time. Cheap (one clock read); called unconditionally from `main`.
pub fn record_start() {
    START_MS.store(now_ms() as u64, Ordering::Relaxed);
}

/// Process start timestamp in epoch ms (0 if `record_start` was never called).
pub fn start_ms() -> u128 {
    START_MS.load(Ordering::Relaxed) as u128
}

/// Whether `ASKHUMAN_PERF` is truthy — used only by the CLI to decide whether to mint a `perf_id`.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| env_truthy("ASKHUMAN_PERF"))
}

/// Harness-supplied correlation id (`ASKHUMAN_PERF_ID`), if it is safe to write into the TSV log.
///
/// Restricted to `[A-Za-z][A-Za-z0-9._:-]{0,63}` so a value can neither forge extra columns nor
/// grow the log line unboundedly, and so it can never be mistaken for the legacy `<pid>-<ms>` form.
/// Invalid values are ignored by the caller, which mints its own id instead.
fn harness_id_from(value: Option<&str>) -> Option<String> {
    // No trimming: a padded value is malformed, not normalized, so the harness either gets the
    // exact key it wrote or clearly falls back.
    let value = value?;
    if value.is_empty() || value.len() > 64 {
        return None;
    }
    let mut chars = value.chars();
    let starts_with_letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    (starts_with_letter
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-')))
    .then(|| value.to_string())
}

/// Correlation id for one CLI invocation.
///
/// Only an Antigravity caller may adopt the harness id; every other agent family keeps the
/// historical `pid-ms` value so Claude/Codex/Cursor/Grok/Pi timelines are unchanged.
pub fn correlation_id(agent_kind: Option<&str>, harness: Option<&str>) -> String {
    if agent_kind == Some(crate::agents::AgentKind::Antigravity.as_str()) {
        if let Some(id) = harness_id_from(harness) {
            return id;
        }
    }
    minted_id()
}

/// Read `ASKHUMAN_PERF_ID` from the environment for [`correlation_id`].
pub fn harness_id_env() -> Option<String> {
    std::env::var("ASKHUMAN_PERF_ID").ok()
}

/// The environment key only when it is well formed, so the CLI does not pay early caller
/// detection for an invalid value.
pub fn valid_harness_id_env() -> Option<String> {
    harness_id_env().filter(|value| harness_id_from(Some(value)).is_some())
}

/// The historical CLI-minted id. Used unchanged when no harness key is present.
pub fn minted_id() -> String {
    format!("{}-{}", std::process::id(), now_ms())
}

/// Whether a correlation id may be written to `perf.log`.
///
/// Two forms exist: the legacy CLI-minted `<pid>-<epoch-ms>` and the harness-supplied
/// `[A-Za-z][A-Za-z0-9._:-]{0,63}`. The daemon-side paint mark uses the same check so a valid
/// harness key is not silently dropped.
pub fn valid_correlation_id(value: &str) -> bool {
    legacy_pid_ms_id(value) || harness_id_from(Some(value)).is_some()
}

fn legacy_pid_ms_id(value: &str) -> bool {
    let Some((pid, started_at_ms)) = value.split_once('-') else {
        return false;
    };
    if pid.is_empty()
        || started_at_ms.is_empty()
        || !pid.bytes().all(|byte| byte.is_ascii_digit())
        || !started_at_ms.bytes().all(|byte| byte.is_ascii_digit())
    {
        return false;
    }
    pid.parse::<u32>().is_ok_and(|parsed| parsed > 0) && started_at_ms.parse::<u128>().is_ok()
}

/// Runtime perf context for the **warm popup** (方案6): a prewarmed helper is spawned without any
/// perf env, so when it is later adopted for a request it learns the request's `perf_id` +
/// autodismiss from the injected `ShowPayload` and stores them here. `effective_id` / `autodismiss`
/// fall back to this when the env vars are absent. Single-use process → set once.
fn runtime_cell() -> &'static std::sync::Mutex<(String, bool)> {
    static RUNTIME: OnceLock<std::sync::Mutex<(String, bool)>> = OnceLock::new();
    RUNTIME.get_or_init(|| std::sync::Mutex::new((String::new(), false)))
}

/// Set the runtime perf context (warm popup adoption). Idempotent / last-write-wins.
pub fn set_runtime(perf_id: &str, autodismiss: bool) {
    if let Ok(mut g) = runtime_cell().lock() {
        *g = (perf_id.to_string(), autodismiss);
    }
}

/// Whether the harness asked the popup to auto-cancel right after first paint (test-only).
/// Honors the env var (cold helper) or the runtime context (warm helper, injected via `ShowPayload`).
pub fn autodismiss() -> bool {
    if env_truthy("ASKHUMAN_PERF_AUTODISMISS") {
        return true;
    }
    runtime_cell().lock().map(|g| g.1).unwrap_or(false)
}

/// Correlation id carried via `ASKHUMAN_PERF_ID` (daemon sets it on the spawned cold helper).
pub fn env_id() -> String {
    std::env::var("ASKHUMAN_PERF_ID").unwrap_or_default()
}

/// Effective correlation id for marks: the env id (cold helper / daemon-per-request) if present,
/// else the runtime id (warm helper, set on adoption). Empty → perf off → marks are no-ops.
pub fn effective_id() -> String {
    let env = env_id();
    if !env.is_empty() {
        return env;
    }
    runtime_cell()
        .lock()
        .map(|g| g.0.clone())
        .unwrap_or_default()
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            let v = v.trim();
            !v.is_empty() && v != "0"
        })
        .unwrap_or(false)
}

/// Current wall-clock in epoch milliseconds (consistent across same-machine processes).
pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Record a milestone for `perf_id` at the current time.
pub fn mark(perf_id: &str, stage: &str) {
    mark_at(perf_id, stage, now_ms());
}

/// Record the daemon-issued request id received by this CLI connection.
///
/// This is deliberately narrower than a daemon-side acceptance event: it proves only that this
/// client received an `Accepted` frame. Invalid ids and disabled per-task performance collection
/// are no-ops so the normal CLI path never creates a diagnostic file by default.
pub fn mark_cli_accepted(perf_id: &str, request_id: &str) {
    let Some(stage) = cli_accepted_stage(perf_id, request_id) else {
        return;
    };
    mark(perf_id, &stage);
}

fn cli_accepted_stage(perf_id: &str, request_id: &str) -> Option<String> {
    if perf_id.is_empty() {
        return None;
    }
    let parsed = uuid::Uuid::parse_str(request_id).ok()?;
    if parsed.is_nil() || parsed.hyphenated().to_string() != request_id {
        return None;
    }
    Some(format!("cli.accepted:{request_id}"))
}

/// Record a milestone for the effective correlation id (helper-side marks; honors warm runtime id).
pub fn mark_env(stage: &str) {
    mark(&effective_id(), stage);
}

/// Record the harness-provided spawn timestamp (`ASKHUMAN_PERF_SPAWN_TS`, epoch ms) under
/// `perf_id`, if present. Lets the harness measure a true end-to-end that includes OS process
/// creation + binary load (everything before `main` runs), which `cli.start` cannot see.
pub fn mark_spawn(perf_id: &str) {
    if perf_id.is_empty() {
        return;
    }
    if let Ok(v) = std::env::var("ASKHUMAN_PERF_SPAWN_TS") {
        if let Ok(ts) = v.trim().parse::<u128>() {
            mark_at(perf_id, "spawn", ts);
        }
    }
}

/// Record a milestone with an explicit timestamp. No-op when `perf_id` is empty (perf off).
/// The frontend passes its own `Date.now()` so timings reflect the page, not the IPC round trip.
pub fn mark_at(perf_id: &str, stage: &str, ts_ms: u128) {
    if perf_id.is_empty() {
        return;
    }
    let line = format!("{ts_ms}\t{perf_id}\t{stage}\t{}\n", std::process::id());
    // Best-effort append; O_APPEND keeps small concurrent cross-process writes from interleaving.
    let path = crate::paths::config_dir().join("perf.log");
    append_line(&path, &line);
}

fn append_line(path: &Path, line: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERF_ID: &str = "cli-123";
    const REQUEST_ID: &str = "4ed587dc-03e9-4b9d-aa01-65c5a2529efe";

    #[test]
    fn cli_accepted_requires_opt_in_and_canonical_non_nil_uuid() {
        assert_eq!(cli_accepted_stage("", REQUEST_ID), None);
        assert_eq!(cli_accepted_stage(PERF_ID, "not-a-uuid"), None);
        assert_eq!(
            cli_accepted_stage(PERF_ID, "00000000-0000-0000-0000-000000000000"),
            None
        );
        assert_eq!(
            cli_accepted_stage(PERF_ID, "4ED587DC-03E9-4B9D-AA01-65C5A2529EFE"),
            None
        );
        assert_eq!(
            cli_accepted_stage(PERF_ID, REQUEST_ID),
            Some(format!("cli.accepted:{REQUEST_ID}"))
        );
    }

    #[test]
    fn harness_id_accepts_only_bounded_safe_values() {
        assert_eq!(
            harness_id_from(Some("agy-20261019:run_1.2")),
            Some("agy-20261019:run_1.2".into())
        );
        assert_eq!(harness_id_from(Some("  agy-run  ")), None);
        assert_eq!(harness_id_from(None), None);
        assert_eq!(harness_id_from(Some("")), None);
        assert_eq!(harness_id_from(Some("   ")), None);
        assert_eq!(harness_id_from(Some("1agy")), None);
        assert_eq!(harness_id_from(Some("-agy")), None);
        assert_eq!(harness_id_from(Some("agy case")), None);
        assert_eq!(harness_id_from(Some("a b")), None);
        assert_eq!(harness_id_from(Some("a\tb")), None);
        assert_eq!(harness_id_from(Some("a\nb")), None);
        assert_eq!(harness_id_from(Some(&"x".repeat(65))), None);
    }

    #[test]
    fn only_antigravity_adopts_the_harness_id() {
        let harness = Some("agy-case-1");
        assert_eq!(correlation_id(Some("antigravity"), harness), "agy-case-1");

        // Every other family keeps the historical pid-ms id even when the env value is set.
        for kind in [
            Some("claude"),
            Some("codex"),
            Some("cursor"),
            Some("grok"),
            Some("pi"),
            None,
        ] {
            let id = correlation_id(kind, harness);
            assert!(
                id.starts_with(&format!("{}-", std::process::id())),
                "{kind:?} unexpectedly adopted {id}"
            );
        }

        // A malformed harness id is never used, not even for Antigravity.
        let id = correlation_id(Some("antigravity"), Some("bad id"));
        assert!(id.starts_with(&format!("{}-", std::process::id())));
    }

    #[test]
    fn minted_id_keeps_the_legacy_shape() {
        let id = minted_id();
        assert!(id.starts_with(&format!("{}-", std::process::id())));
        assert!(valid_correlation_id(&id));
        let epoch = id.split_once('-').map(|(_, rest)| rest).unwrap_or_default();
        assert!(epoch.bytes().all(|byte| byte.is_ascii_digit()));
        // Non-Antigravity callers keep the same shape even when a harness key is present.
        let other = correlation_id(Some("claude"), Some("agy-case-1"));
        let other_epoch = other.split_once('-').map(|(_, rest)| rest).unwrap_or_default();
        assert!(other.starts_with(&format!("{}-", std::process::id())));
        assert!(other_epoch.bytes().all(|byte| byte.is_ascii_digit()));
    }

    #[test]
    fn append_line_keeps_four_columns_and_repeated_ack_observations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("perf.log");
        let stage = cli_accepted_stage(PERF_ID, REQUEST_ID).unwrap();
        let first = format!("100\t{PERF_ID}\t{stage}\t42\n");
        let second = format!("101\t{PERF_ID}\t{stage}\t42\n");

        append_line(&path, &first);
        append_line(&path, &second);

        let lines = std::fs::read_to_string(path).unwrap();
        let records: Vec<_> = lines.lines().collect();
        assert_eq!(records, vec![first.trim_end(), second.trim_end()]);
        assert!(records.iter().all(|line| line.split('\t').count() == 4));
    }

    #[test]
    fn append_line_ignores_unwritable_target() {
        let dir = tempfile::tempdir().unwrap();
        append_line(dir.path(), "ignored\n");
    }
}
