// SPDX-FileCopyrightText: 2025 fspy <gh@fspy.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turns off displays when the user is idle

use std::collections::HashMap;

use wayland_client::protocol::{wl_output, wl_registry, wl_seat};
use wayland_client::{delegate_noop, Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};
use wayland_protocols_wlr::output_power_management::v1::client::{
    zwlr_output_power_manager_v1,
    zwlr_output_power_v1::{self, Mode},
};

#[derive(Default)]
struct State {
    outputs: HashMap<u32, wl_output::WlOutput>,
    seat: Option<wl_seat::WlSeat>,
    output_power_manager: Option<zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1>,
    idle_notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    timeout_ms: u32,
    idle_notification: Option<ext_idle_notification_v1::ExtIdleNotificationV1>,
    is_idle: bool,
    protocols_ready: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<State>,
    ) {
        match event {
            wl_registry::Event::Global {
                name, interface, ..
            } => {
                match &interface[..] {
                    "wl_output" => {
                        let output = registry.bind::<wl_output::WlOutput, _, _>(name, 2, qh, ());
                        state.outputs.insert(name, output);
                    }
                    "wl_seat" => {
                        state.seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(name, 7, qh, ()));
                    }
                    "zwlr_output_power_manager_v1" => {
                        state.output_power_manager = Some(
                            registry
                                .bind::<zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1, _, _>(
                                    name,
                                    1,
                                    qh,
                                    (),
                                ),
                        );
                    }
                    "ext_idle_notifier_v1" => {
                        state.idle_notifier = Some(
                            registry.bind::<ext_idle_notifier_v1::ExtIdleNotifierV1, _, _>(
                                name,
                                1,
                                qh,
                                (),
                            ),
                        );
                    }
                    _ => return,
                }
                state.check_protocols_ready();
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some(output) = state.outputs.remove(&name) {
                    output.release();
                }
            }
            _ => {}
        }
    }
}

delegate_noop!(State: ignore wl_seat::WlSeat);
delegate_noop!(State: ignore wl_output::WlOutput);
delegate_noop!(State: ext_idle_notifier_v1::ExtIdleNotifierV1);
delegate_noop!(State: zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1);
delegate_noop!(State: ignore zwlr_output_power_v1::ZwlrOutputPowerV1);

impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, ()> for State {
    fn event(
        state: &mut Self,
        notification: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => {
                if !state.is_idle {
                    tracing::info!("System went idle, turning off displays");
                    state.is_idle = true;
                    state.set_display_power_mode(Mode::Off, qh);
                }
            }
            ext_idle_notification_v1::Event::Resumed => {
                if state.is_idle {
                    tracing::info!("System resumed from idle, turning on displays");
                    state.is_idle = false;
                    state.set_display_power_mode(Mode::On, qh);
                }
                notification.destroy();
                state.idle_notification = None;
                state.create_idle_notification(qh);
            }
            _ => {
                tracing::warn!("Received unexpected idle notification event");
            }
        }
    }
}

impl State {
    fn set_display_power_mode(&self, mode: Mode, qh: &QueueHandle<Self>) {
        if let Some(power_manager) = self.output_power_manager.as_ref() {
            self.outputs.iter().for_each(|(_, output)| {
                let power_control = power_manager.get_output_power(output, qh, ());
                power_control.set_mode(mode);
                power_control.destroy();
            });
        } else {
            tracing::warn!("No output power manager available to set display power mode");
        }
    }

    fn create_idle_notification(&mut self, qh: &QueueHandle<Self>) {
        if let (Some(idle_notifier), Some(seat)) = (&self.idle_notifier, &self.seat) {
            self.idle_notification =
                Some(idle_notifier.get_idle_notification(self.timeout_ms, seat, qh, ()));
        } else {
            tracing::warn!("Cannot create idle notification: missing idle notifier or seat");
        }
    }

    fn has_required_protocols(&self) -> bool {
        self.output_power_manager.is_some()
            && self.idle_notifier.is_some()
            && !self.outputs.is_empty()
    }

    fn check_protocols_ready(&mut self) {
        self.protocols_ready = self.has_required_protocols();
    }
}

pub fn run_idle_loop(timeout_ms: u32) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting idle monitoring with {}ms timeout", timeout_ms);

    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    let mut state = State {
        timeout_ms,
        ..Default::default()
    };

    // Discover all available Wayland globals
    event_queue.roundtrip(&mut state)?;

    if !state.has_required_protocols() {
        if state.output_power_manager.is_none() {
            tracing::warn!("zwlr_output_power_manager_v1 not supported by the Wayland compositor. Idle functionality disabled.");
        }
        if state.idle_notifier.is_none() {
            tracing::warn!("ext_idle_notifier_v1 not supported by the Wayland compositor. Idle functionality disabled.");
        }
        if state.outputs.is_empty() {
            tracing::warn!("No outputs found. Idle functionality disabled.");
        }
        return Ok(());
    }

    tracing::info!(
        "Found {} output(s), setting up idle monitoring",
        state.outputs.len()
    );

    state.create_idle_notification(&qh);

    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}
