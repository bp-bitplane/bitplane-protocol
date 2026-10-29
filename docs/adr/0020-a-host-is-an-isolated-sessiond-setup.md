---
status: accepted
date: 2026-10-26
---

# A host is an isolated sessiond setup, and isolation between identities stops at the host

A **host** is one isolated sessiond setup: one OS account in one OS environment (a machine, a VM or a container). It has one **daemon**, the host's `bitplane-sessiond`, which runs as one **daemon instance** per protocol version; after an update the old instance drains and exits. Two OS accounts on one machine are two hosts, and so is every container. Hosts are never grouped by **machine**, which is a plain word here, not a domain concept. The map (protocol#1) and its sub-issues said both "one daemon per host" and "per-user daemon on a host"; with this definition the first is the canonical phrasing, and "user" never means the OS account.

Setups never overlap: a setup reads and writes only inside its OS account's home, XDG directories and user-service namespace (systemd user unit with linger, launchd agent). No system unit, LaunchDaemon, `/run`, `/opt`, shared install or root. The definition of a host depends on this rule, so a ticket that wants a system-wide install has to supersede this ADR first.

Users and access are kept apart from OS accounts. A **user** is a human using any of the bitplane tools; an **identity** is a named principal the gateway knows (`alice`, `team-a`, `ci-bot`), and a user holds one or more of them. Bots have identities but no user. Personal mode has one implicit identity, `local`. The host is the isolation boundary: the gateway's access list (identity → hosts, part of the M4 auth plugin) decides who reaches a host, and every identity on a host is fully trusted there. sessiond never authorizes per identity; identities that must not see each other's work get separate hosts, either separate OS accounts or containers from the M3 Docker driver.

## Considered Options

- **One shared daemon per machine, run as a system service.** Rejected: it needs root, its own auth layer and path confinement per OS account, and the peer-uid check on the socket would no longer isolate anything.
- **Per-identity checks inside sessiond** (session filtering, path restriction, cancel rights by identity, as the M4 notes on protocol#6, protocol#8, sessiond#4 and cc#26 assumed). Rejected: a second, weaker isolation layer inside a host, when the OS already enforces one at the host boundary.
- **No permissions in M4, separation in a later milestone.** Rejected: a company gateway where every identity reaches every host isn't shippable, and the access list is small.

## Consequences

- `startedBy` gains the identity in M4, next to the client; the user is recorded only in the gateway's audit log.
- cc's tab filtering (cc#26 §8) is a view convenience, not security. How it works stays in the fog.
- Where the user → identity mapping lives stays in the fog, leaning toward the auth plugin.
- Each daemon instance keeps its socket and pidfile in `$XDG_RUNTIME_DIR/bitplane/sessiond/<version>/` as `sessiond.sock` and `sessiond.pid` (`$TMPDIR` on macOS, which keeps the socket path under the 104-byte `sun_path` limit), and its `sessions.json` and log in `$XDG_STATE_HOME/bitplane/sessiond/<version>/`.
- Containers are hosts by definition; reaching one (`docker exec` or SSH inside it) and starting the daemon without systemd stay in the fog.
