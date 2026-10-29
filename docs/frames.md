# Frames and the envelope

Protocol v1 carries terminal data as binary frames beside JSON-RPC (ADR 0024). This page is the byte layout; `test-vectors/frames.json` is its executable form, and the Rust and TS codecs both run it.

## Frames

A frame is one kind byte followed by a body. All integers are unsigned and **big-endian**.

| Kind | Name | Body | Direction |
|---|---|---|---|
| `0x00` | JSON | one UTF-8 JSON-RPC message | both |
| `0x01` | SNAPSHOT | `u32 attachment · u64 offset · VT bytes` | daemon → client |
| `0x02` | OUTPUT | `u32 attachment · u64 offset · bytes` | daemon → client |
| `0x03` | INPUT | `u32 attachment · bytes` | client → daemon |

- `attachment` is the id the daemon gave the attachment on this connection.
- `offset` is the session's output byte offset of the first byte that follows: SNAPSHOT is the terminal state as of `offset`, and OUTPUT continues from exactly there.
- INPUT is written to the PTY whole, never interleaved with another frame's bytes.
- The data after the fixed header may be empty.

## Transports

- **Stream** (Unix socket, stdio, SSH subsystem): every frame, JSON included, is wrapped in an **envelope**, `u32 length · frame`. `length` counts the kind byte and the body, not itself.
- **WebSocket**: JSON-RPC travels as text messages, and every other frame as one binary message holding the bare frame (`u8 kind · body`).

## Errors

A reader closes the connection on any of these:

| Code | When |
|---|---|
| `empty` | an envelope length of 0, so there is no kind byte |
| `unknown_kind` | a kind byte outside `0x00`–`0x03` |
| `short_body` | a SNAPSHOT or OUTPUT body under 12 bytes, or an INPUT body under 4 |
| `too_large` | an envelope length over 16 MiB |

A stream reader that holds fewer bytes than the length promises waits for more; that is not an error.

TS clients read `offset` as a `number` and refuse an offset above `Number.MAX_SAFE_INTEGER` (2⁵³ − 1 bytes of output).
