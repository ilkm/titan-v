# Fleet control plane (Center ↔ many Hosts)

## TCP (command + telemetry)

- **Command TCP**: framed `MAGIC` + version + length + **rkyv** payload (`titan-common::wire::codec`).
- **Telemetry TCP**: command port + `CONTROL_PLANE_TELEMETRY_PORT_OFFSET` (default +1); host pushes `ControlPush` including ~**3 FPS** desktop JPEG for card thumbnails.

## Socket tuning

- **Host**: listeners are created via `socket2` (`titan_host::tcp_tune::tcp_listen_tokio`) then wrapped as Tokio `TcpListener`.
- **Center**: after `TcpStream::connect`, `set_nodelay(true)` via `titan_center::app::tcp_tune::tune_connected_stream`.

**Note:** Experimental QUIC fleet UDP and optional zstd framing helpers were removed from the codebase; LAN control stays on **TCP** (command + telemetry) as above.

## Center fleet UI / state

- **`fleet_by_endpoint`**: per-host `HostLiveState` (VMs, volumes, telemetry flags).
- **Device cards**: “Fleet” checkbox + **Fleet Hello** sends `Hello` to all selected rows with bounded concurrency (`JoinSet` + `Semaphore` 32).
- **`net_busy` vs `fleet_busy`**: single-host RPC uses `net_busy`; fleet fan-out uses `fleet_busy` so list polling and fleet probes do not starve each other.

## Telemetry `host_key`

- `NetUiMsg::HostTelemetry` / `TelemetryLinkLost` carry **`host_key`** so multiple telemetry sessions can be routed once Center opens more than one stream.
- **Connect tab**: **Fleet telemetry** starts one TCP telemetry reader per checked device (soft cap **8** concurrent streams, `TELEMETRY_MAX_CONCURRENT`). Each stream has its own `session_gen` so stale pushes after stop are ignored.
