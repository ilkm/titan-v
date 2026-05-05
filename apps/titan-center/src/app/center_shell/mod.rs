//! Shell lifecycle: bootstrap, per-frame ticks, paint, control-plane net, LAN bridges, session helpers.

mod bootstrap;
mod frame_tick;
mod net_inbox;
mod net_lan;
mod net_loop;
mod paint;
mod persist_session;
mod tofu;
