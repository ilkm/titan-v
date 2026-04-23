use std::io::ErrorKind;

use titan_common::{decode_request_payload, parse_header, ControlRequest, WireError};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use super::errors::ServeError;

pub(super) async fn read_exact_tcp(sock: &mut TcpStream, buf: &mut [u8]) -> std::io::Result<()> {
    let mut off = 0usize;
    while off < buf.len() {
        let n = sock.read(&mut buf[off..]).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "connection closed before frame completed",
            ));
        }
        off += n;
    }
    Ok(())
}

pub(super) async fn read_one_request(
    sock: &mut TcpStream,
) -> Result<Option<ControlRequest>, ServeError> {
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    match read_exact_tcp(sock, &mut hdr).await {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let (_ver, len) = parse_header(&hdr)?;
    let mut payload = vec![0u8; len as usize];
    if let Err(e) = read_exact_tcp(sock, &mut payload).await {
        if e.kind() == ErrorKind::UnexpectedEof {
            return Err(WireError::UnexpectedEof.into());
        }
        return Err(e.into());
    }
    Ok(Some(decode_request_payload(&payload)?))
}
