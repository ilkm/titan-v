# Fleet control plane (Center ↔ many Hosts)

## TCP (current command + telemetry)

- **Command TCP**: framed `MAGIC` + version + length + **rkyv** payload (`titan-common::wire::codec`).
- **Telemetry TCP**: command port + `CONTROL_PLANE_TELEMETRY_PORT_OFFSET` (default +1); Host pushes `ControlPush` including ~**3 FPS** desktop JPEG for card thumbnails.

## Socket tuning

- **Host**: listeners are created via `socket2` (`titan_host::tcp_tune::tcp_listen_tokio`) then wrapped as Tokio `TcpListener`.
- **Center**: after `TcpStream::connect`, `set_nodelay(true)` via `titan_center::app::tcp_tune::tune_connected_stream`.

## QUIC (experimental)

- **UDP** bind address: command `SocketAddr` + `CONTROL_PLANE_QUIC_PORT_OFFSET` (see `titan_common::control_plane_quic_addr`).
- **Host** starts a background `quinn` listener with a **self-signed** localhost certificate and ALPN `titan-fleet-v1`. Production should use a shared CA / pinned certs; Center clients must not use skip-verify outside lab.

## rkyv + zstd (shared helpers)

- **`FleetRkyvPing`**: minimal rkyv roundtrip type for future v2 framing (`titan_common::fleet_rkyv_*`).
- **`maybe_zstd_compress`**: optional zstd for **control-sized** blobs; **do not** compress raw JPEG pushes.

## Center fleet UI / state

- **`fleet_by_endpoint`**: per-host `HostLiveState` (VMs, volumes, telemetry flags).
- **Device cards**: “Fleet” checkbox + **Fleet Hello** sends `Hello` to all selected rows with bounded concurrency (`JoinSet` + `Semaphore` 32).
- **`net_busy` vs `fleet_busy`**: single-host RPC uses `net_busy`; fleet fan-out uses `fleet_busy` so list polling and fleet probes do not starve each other.

## Telemetry `host_key`

- `NetUiMsg::HostTelemetry` / `TelemetryLinkLost` carry **`host_key`** so multiple telemetry sessions can be routed once Center opens more than one stream.
- **Connect tab**: **Fleet telemetry** starts one TCP telemetry reader per checked device (soft cap **8** concurrent streams, `TELEMETRY_MAX_CONCURRENT`). Each stream has its own `session_gen` so stale pushes after stop are ignored.
