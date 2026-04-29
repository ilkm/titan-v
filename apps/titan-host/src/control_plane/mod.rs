//! Control-plane framing and message types shared with Titan Center clients.
//!
//! Definitions live in [`titan_common`] so the **Center** desktop app does not depend on this
//! crate’s UI stack; **TCP/QUIC serving** is implemented under [`crate::serve`].

pub use titan_common::{
    ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame, ControlResponse,
    FRAME_HEADER_LEN, MAX_PAYLOAD_BYTES, VmBrief, encode_control_host_frame, encode_request_frame,
    parse_header, read_control_host_frame,
};
