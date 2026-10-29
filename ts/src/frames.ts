// Binary frames and the stream envelope, as specified in docs/frames.md.

export const MAX_FRAME_LEN = 16 * 1024 * 1024;

export type Frame =
  | { kind: "json"; data: Uint8Array }
  | { kind: "snapshot"; attachment: number; offset: number; data: Uint8Array }
  | { kind: "output"; attachment: number; offset: number; data: Uint8Array }
  | { kind: "input"; attachment: number; data: Uint8Array };

export type FrameErrorCode = "empty" | "unknown_kind" | "short_body" | "too_large";

export class FrameError extends Error {
  constructor(
    readonly code: FrameErrorCode,
    message: string,
  ) {
    super(message);
    this.name = "FrameError";
  }
}

/** `u8 kind · body`: the WebSocket binary message. */
export function encodeFrame(frame: Frame): Uint8Array {
  const header = frame.kind === "json" ? 1 : frame.kind === "input" ? 5 : 13;
  const out = new Uint8Array(header + frame.data.length);
  const view = new DataView(out.buffer);
  out[0] = KIND_BYTE[frame.kind];
  if (frame.kind !== "json") {
    view.setUint32(1, frame.attachment);
  }
  if (frame.kind === "snapshot" || frame.kind === "output") {
    view.setBigUint64(5, BigInt(frame.offset));
  }
  out.set(frame.data, header);
  return out;
}

/** Reads `u8 kind · body`, the inverse of `encodeFrame`. */
export function decodeFrame(bytes: Uint8Array): Frame {
  if (bytes.length === 0) {
    throw new FrameError("empty", "frame has no kind byte");
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const kindByte = bytes[0] as number;
  switch (kindByte) {
    case 0x00:
      return { kind: "json", data: bytes.slice(1) };
    case 0x01:
    case 0x02: {
      const kind = kindByte === 0x01 ? "snapshot" : "output";
      if (bytes.length < 13) {
        throw new FrameError("short_body", `${kind} frame body is too short`);
      }
      const offset = view.getBigUint64(5);
      if (offset > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw new RangeError(`offset ${offset} exceeds Number.MAX_SAFE_INTEGER`);
      }
      return {
        kind,
        attachment: view.getUint32(1),
        offset: Number(offset),
        data: bytes.slice(13),
      };
    }
    case 0x03:
      if (bytes.length < 5) {
        throw new FrameError("short_body", "input frame body is too short");
      }
      return { kind: "input", attachment: view.getUint32(1), data: bytes.slice(5) };
    default:
      throw new FrameError("unknown_kind", `unknown frame kind 0x${kindByte.toString(16)}`);
  }
}

/** `u32 BE length · u8 kind · body`, for stream transports. */
export function encodeEnvelope(frame: Frame): Uint8Array {
  const body = encodeFrame(frame);
  const out = new Uint8Array(4 + body.length);
  new DataView(out.buffer).setUint32(0, body.length);
  out.set(body, 4);
  return out;
}

/** Splits a byte stream into frames; feed chunks as they arrive. */
export class EnvelopeDecoder {
  private buffer = new Uint8Array(0);

  push(chunk: Uint8Array): Frame[] {
    const joined = new Uint8Array(this.buffer.length + chunk.length);
    joined.set(this.buffer);
    joined.set(chunk, this.buffer.length);

    const frames: Frame[] = [];
    let at = 0;
    while (joined.length - at >= 4) {
      const length = new DataView(joined.buffer, at).getUint32(0);
      if (length === 0) {
        throw new FrameError("empty", "frame has no kind byte");
      }
      if (length > MAX_FRAME_LEN) {
        throw new FrameError("too_large", `frame of ${length} bytes exceeds the limit`);
      }
      if (joined.length - at - 4 < length) {
        break;
      }
      frames.push(decodeFrame(joined.subarray(at + 4, at + 4 + length)));
      at += 4 + length;
    }
    this.buffer = joined.slice(at);
    return frames;
  }
}

const KIND_BYTE = { json: 0x00, snapshot: 0x01, output: 0x02, input: 0x03 } as const;
