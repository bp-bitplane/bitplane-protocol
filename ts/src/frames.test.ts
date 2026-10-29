import { describe, expect, it } from "vitest";
import vectors from "../../test-vectors/frames.json" with { type: "json" };
import {
  decodeFrame,
  EnvelopeDecoder,
  encodeEnvelope,
  encodeFrame,
  type Frame,
  FrameError,
} from "./frames.ts";

describe("valid vectors", () => {
  for (const vector of vectors.valid) {
    const frame = frameOf(vector.frame);

    it(`${vector.name} encodes`, () => {
      expect(hex(encodeFrame(frame))).toBe(vector.websocket);
      expect(hex(encodeEnvelope(frame))).toBe(vector.envelope);
    });

    it(`${vector.name} decodes`, () => {
      expect(decodeFrame(bytes(vector.websocket))).toEqual(frame);
      expect(new EnvelopeDecoder().push(bytes(vector.envelope))).toEqual([frame]);
    });

    it(`${vector.name} decodes one byte at a time`, () => {
      const decoder = new EnvelopeDecoder();
      const frames = [...bytes(vector.envelope)].flatMap((b) => decoder.push(Uint8Array.of(b)));
      expect(frames).toEqual([frame]);
    });
  }
});

describe("invalid vectors", () => {
  for (const vector of vectors.invalid) {
    it(`${vector.name} is refused`, () => {
      const decode = () => new EnvelopeDecoder().push(bytes(vector.envelope));
      expect(decode).toThrow(FrameError);
      try {
        decode();
      } catch (error) {
        expect((error as FrameError).code).toBe(vector.error);
      }
    });
  }
});

describe("incomplete vectors", () => {
  for (const vector of vectors.incomplete) {
    it(`${vector.name} waits for more bytes`, () => {
      expect(new EnvelopeDecoder().push(bytes(vector.envelope))).toEqual([]);
    });
  }
});

type VectorFrame = { kind: string; data: string; attachment?: number; offset?: number };

function frameOf(v: VectorFrame): Frame {
  const data = bytes(v.data);
  switch (v.kind) {
    case "json":
      return { kind: "json", data };
    case "snapshot":
    case "output":
      return { kind: v.kind, attachment: v.attachment ?? 0, offset: v.offset ?? 0, data };
    case "input":
      return { kind: "input", attachment: v.attachment ?? 0, data };
    default:
      throw new Error(`unknown kind ${v.kind} in vectors`);
  }
}

function bytes(text: string): Uint8Array {
  return Uint8Array.from(text.match(/../g) ?? [], (pair) => Number.parseInt(pair, 16));
}

function hex(data: Uint8Array): string {
  return [...data].map((b) => b.toString(16).padStart(2, "0")).join("");
}
