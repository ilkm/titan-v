# Fleet control plane (Center ↔ many Hosts)

## QUIC + mTLS (command + telemetry)

- **RPC plane**: framed `ControlRequest/ControlResponse` on QUIC bi-streams (one RPC per stream).
- **Telemetry plane**: QUIC uni-stream push (`ControlPush`), including host heartbeat + desktop preview JPEG.

## Transport notes

- **Host**: QUIC endpoint runs in `titan-host::serve::run`, with mTLS identity/trust bootstrapped from host storage.
- **Center**: QUIC client side lives under `titan-center` net spawn layer; telemetry readers are per-host session keyed.
- **LAN discoverability** remains UDP beacon based (`HostAnnounceBeacon` / `CenterPollBeacon`) and is independent from QUIC stream lifecycle.

## Center fleet UI / state

- **`fleet_by_endpoint`**: per-host `HostLiveState` (VMs, volumes, telemetry flags).
- **Device cards**: “Fleet” checkbox + **Fleet Hello** sends `Hello` to all selected rows with bounded concurrency (`JoinSet` + `Semaphore` 32).
- **`net_busy` vs `fleet_busy`**: single-host RPC uses `net_busy`; fleet fan-out uses `fleet_busy` so list polling and fleet probes do not starve each other.

## Telemetry `host_key`

- `NetUiMsg::HostTelemetry` / `TelemetryLinkLost` carry **`host_key`** so multiple telemetry sessions can be routed once Center opens more than one stream.
- **Connect tab**: **Fleet telemetry** starts one QUIC telemetry reader per checked device (soft cap **8** concurrent streams, `TELEMETRY_MAX_CONCURRENT`). Each stream has its own `session_gen` so stale pushes after stop are ignored.
