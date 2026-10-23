# Antigravity integration (experimental)

Choose Antigravity → CLI in Settings → Agents, or run:

```bash
AskHuman agents mode antigravity cli
```

AskHuman installs a shared family always_on protocol at `~/.gemini/config/rules/askhuman-antigravity.md`.
CLI means calling the AskHuman command line. It does not limit the host to a TUI. A missing `agy` binary does not prevent IDE users from installing the shared rule; rule discovery, command execution and waiting must be verified for each client.
P0 is experimental. Installed CLI questions, attachments, independent unbound concurrent questions, todos and next-task handoff were tested on NixOS. Basic agy 1.2.9 calls were observed, but full native certification remains blocked.

Managed calls include a family hint:

```bash
AskHuman --caller-agent antigravity "Please confirm the next step"
```

The hint declares a family only. Native conversation UUIDs are not trusted bindings. Inherited foreign Agent family markers cause a conflict error.
Unbound `--show-last` refuses project-wide history fallback; ask the human again. Do not guess history from another session or recent logs.
Wait for the real answer using the client's available command tool. Moving a command into the background does not set a human-answer deadline.
Original TaskId continuation guidance applies only to an agy host with the corresponding tool contract; IDE and extension tools are not assumed identical.

Inspect, update or remove the integration:

```bash
AskHuman agents mode antigravity
AskHuman agents show antigravity
AskHuman agents update antigravity
AskHuman agents mode antigravity none
```

Repeated installation and current-version updates are no-ops. None removes only the exclusive file whose ownership the current instance can verify, retaining shared directories and other rules.
Unknown existing files, edits, file replacement, ledger errors and pending transactions refuse overwrite. Read errors are not displayed as “not installed”.
Dev Instance ownership is isolated from the main instance; instances do not adopt each other's global file. A temporary acceptance installation is not permanent main-instance configuration.

MCP, trusted history recovery, lifecycle/Stop, permission interception, interjection/Watch, task launch and Fork remain disabled.
agy 1.2.9 on NixOS has evidence for original TaskId continuation, scoped Request Review positive/negative controls, actual cancellation and structured short replies. Exact content is certified only in `result.structured_output.output`; free-text `result.response` aggregates waiting notices. [Real 30/120/600-second waits](../investigations/antigravity-cli-real-waits.md) and exact output from the originating task passed; full native certification remains incomplete. AskHuman installs no permission allow rules; temporary test settings were restored. Other agy instances share user-level settings.
Antigravity IDE, extensions, 2.0 and macOS have no completed real-host acceptance. Windows still needs the strict identity/reparse/ACL adapter implementation and native verification. Dedicated queries/mutations return an explicit unavailable error; other families' aggregate diagnostics and collaboration-style updates remain available.

See the [implementation evidence and remaining gates](../plans/antigravity-cli.md#p0-执行记录2026-10-04).

2026-10-16 [resumed validation](../investigations/antigravity-cli-n0-resume.md): agy1.2.16/AskHuman0.14.5 on NixOS demonstrated safe no-hint recovery refusal, native child metadata, and a real Unicode/space-path attachment reply. A Linux cold-popup blank-content bug was fixed. Prior1.2.9 wait certification is not inherited by the newer version; permission certification and trusted binding remain unavailable.
