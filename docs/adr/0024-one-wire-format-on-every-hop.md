---
status: accepted
date: 2026-10-25
---

# One wire format on every hop, and `hello` comes first

Every hop between a client and a daemon speaks the same protocol v1: cc to its local daemon over a Unix socket now, and through the gateway, stdio, an SSH subsystem or a WebSocket later. A hop may change the transport, never the messages, so the gateway relays without translating and a client works the same against a local or a remote host.

Control is **JSON-RPC 2.0**. Requests run concurrently and replies are matched by `id`, so they may arrive out of order. Terminal data travels as **binary frames**, never inside JSON:

| Kind | Frame | Body |
|---|---|---|
| `0x00` | JSON | one JSON-RPC message |
| `0x01` | SNAPSHOT | `u32 attachment · u64 offset · VT bytes` |
| `0x02` | OUTPUT | `u32 attachment · u64 offset · bytes` |
| `0x03` | INPUT | `u32 attachment · bytes`, client → daemon, written to the PTY whole |

On a stream transport (Unix socket, stdio, SSH subsystem) each frame is wrapped in the **envelope** `u32 BE length · u8 kind · body`. On a WebSocket, text messages carry JSON-RPC and binary messages carry `u8 kind · body`, since the transport already delimits messages. Frames are addressed by an attachment id that is local to the connection; the offset is the session's output byte offset, which is what lets an attachment resume after SNAPSHOT without losing or repeating a byte.

**`hello` must be the first request on a connection.** The client sends `{ protocolVersion, minProtocolVersion, client: { name, version, via? } }`; the daemon answers `{ protocolVersion, daemon: { name, version, instance }, capabilities }` or refuses a version range it cannot serve. `instance` is random per daemon process, so a client that reconnects can tell a dropped connection from a restarted daemon. `capabilities` lists every module with its availability (ADR 0023).

## Considered Options

- **Terminal bytes inside JSON-RPC** (base64 strings). Rejected: a third more bytes on the hottest path, and an encode and decode per chunk on both ends.
- **A text protocol with escaped output**, as tmux control mode does with octal escapes. Rejected for the same cost, and because it is not a format the gateway or a browser can pass through.
- **A different framing per transport.** Rejected: every hop would need its own codec and the gateway would have to translate. WebSocket only drops the length prefix, which its own message boundaries replace.
- **Negotiating the version later, or per method.** Rejected: nothing on a connection is meaningful before both ends agree on a version, so `hello` gates everything.

## Consequences

- The byte order of the integers inside a frame body is not fixed yet; the test vectors fix it. The frame layout and envelope are specified with those vectors, and both hand-written codecs (Rust and TS) are tested against them (ADR 0021).
- Flow control is per attachment and acknowledged in JSON-RPC (`session.attachment.ack`), so a slow client never stalls the PTY or another attachment.
- An older daemon refuses a newer client's minimum version in `hello`; the client then asks before restarting it with its own version.
- Decisions: [Protocol v1: the method set](https://github.com/bp-bitplane/bitplane-protocol/issues/3) (prototype v3 on `alfonz/prototype-protocol-v1`).
