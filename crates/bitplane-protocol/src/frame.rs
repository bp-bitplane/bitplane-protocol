//! Binary frames and the stream envelope, as specified in `docs/frames.md`.

use std::fmt;

/// Upper bound on an envelope's length field (kind byte plus body).
pub const MAX_FRAME_LEN: u32 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Json = 0x00,
    Snapshot = 0x01,
    Output = 0x02,
    Input = 0x03,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// One UTF-8 JSON-RPC message.
    Json(Vec<u8>),
    /// Terminal state as of session byte `offset`.
    Snapshot {
        attachment: u32,
        offset: u64,
        data: Vec<u8>,
    },
    /// Live PTY bytes starting at session byte `offset`.
    Output {
        attachment: u32,
        offset: u64,
        data: Vec<u8>,
    },
    /// Client → daemon, written to the PTY whole.
    Input { attachment: u32, data: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// A length of zero: there is no kind byte.
    Empty,
    UnknownKind(u8),
    /// The body is shorter than the kind's fixed header.
    ShortBody(Kind),
    TooLarge(u32),
}

impl Kind {
    pub fn from_byte(byte: u8) -> Result<Self, FrameError> {
        match byte {
            0x00 => Ok(Kind::Json),
            0x01 => Ok(Kind::Snapshot),
            0x02 => Ok(Kind::Output),
            0x03 => Ok(Kind::Input),
            other => Err(FrameError::UnknownKind(other)),
        }
    }
}

impl Frame {
    pub fn kind(&self) -> Kind {
        match self {
            Frame::Json(_) => Kind::Json,
            Frame::Snapshot { .. } => Kind::Snapshot,
            Frame::Output { .. } => Kind::Output,
            Frame::Input { .. } => Kind::Input,
        }
    }

    /// `u8 kind · body`: the WebSocket binary message, and the part of an
    /// envelope after its length.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![self.kind() as u8];
        match self {
            Frame::Json(data) => out.extend_from_slice(data),
            Frame::Snapshot {
                attachment,
                offset,
                data,
            }
            | Frame::Output {
                attachment,
                offset,
                data,
            } => {
                out.extend_from_slice(&attachment.to_be_bytes());
                out.extend_from_slice(&offset.to_be_bytes());
                out.extend_from_slice(data);
            }
            Frame::Input { attachment, data } => {
                out.extend_from_slice(&attachment.to_be_bytes());
                out.extend_from_slice(data);
            }
        }
        out
    }

    /// Reads `u8 kind · body`, the inverse of [`Frame::encode`].
    pub fn decode(bytes: &[u8]) -> Result<Frame, FrameError> {
        let (&kind, body) = bytes.split_first().ok_or(FrameError::Empty)?;
        let kind = Kind::from_byte(kind)?;
        let short = || FrameError::ShortBody(kind);
        Ok(match kind {
            Kind::Json => Frame::Json(body.to_vec()),
            Kind::Snapshot | Kind::Output => {
                let (attachment, rest) = split_u32(body).ok_or_else(short)?;
                let (offset, data) = split_u64(rest).ok_or_else(short)?;
                let data = data.to_vec();
                if kind == Kind::Snapshot {
                    Frame::Snapshot {
                        attachment,
                        offset,
                        data,
                    }
                } else {
                    Frame::Output {
                        attachment,
                        offset,
                        data,
                    }
                }
            }
            Kind::Input => {
                let (attachment, data) = split_u32(body).ok_or_else(short)?;
                Frame::Input {
                    attachment,
                    data: data.to_vec(),
                }
            }
        })
    }

    /// `u32 BE length · u8 kind · body`, for stream transports.
    pub fn encode_envelope(&self) -> Vec<u8> {
        let frame = self.encode();
        let mut out = Vec::with_capacity(4 + frame.len());
        out.extend_from_slice(&(frame.len() as u32).to_be_bytes());
        out.extend_from_slice(&frame);
        out
    }
}

/// Reads one envelope from the front of `buf`. `Ok(None)` means more bytes
/// are needed; otherwise returns the frame and how many bytes it used.
pub fn decode_envelope(buf: &[u8]) -> Result<Option<(Frame, usize)>, FrameError> {
    let Some((len, rest)) = split_u32(buf) else {
        return Ok(None);
    };
    if len == 0 {
        return Err(FrameError::Empty);
    }
    if len > MAX_FRAME_LEN {
        return Err(FrameError::TooLarge(len));
    }
    let len = len as usize;
    if rest.len() < len {
        return Ok(None);
    }
    Ok(Some((Frame::decode(&rest[..len])?, 4 + len)))
}

fn split_u32(bytes: &[u8]) -> Option<(u32, &[u8])> {
    let (head, rest) = bytes.split_first_chunk::<4>()?;
    Some((u32::from_be_bytes(*head), rest))
}

fn split_u64(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let (head, rest) = bytes.split_first_chunk::<8>()?;
    Some((u64::from_be_bytes(*head), rest))
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => write!(f, "frame has no kind byte"),
            FrameError::UnknownKind(kind) => write!(f, "unknown frame kind {kind:#04x}"),
            FrameError::ShortBody(kind) => write!(f, "{kind:?} frame body is too short"),
            FrameError::TooLarge(len) => {
                write!(
                    f,
                    "frame of {len} bytes exceeds the {MAX_FRAME_LEN}-byte limit"
                )
            }
        }
    }
}

impl std::error::Error for FrameError {}
