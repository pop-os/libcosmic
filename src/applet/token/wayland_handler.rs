use std::os::{
    fd::{FromRawFd, RawFd},
    unix::net::UnixStream,
};

use super::subscription::{TokenRequest, TokenUpdate};
use cctk::{
    sctk::{
        self,
        activation::{RequestData, RequestDataExt},
        reexports::{calloop, calloop_wayland_source::WaylandSource},
        seat::{SeatHandler, SeatState},
    },
    wayland_client::{
        self,
        protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    },
};
use iced_futures::futures::channel::mpsc::UnboundedSender;
use sctk::{
    activation::{ActivationHandler, ActivationState},
    registry::{ProvidesRegistryState, RegistryState},
};
use wayland_client::{Connection, QueueHandle, globals::registry_queue_init};

struct AppData {
    exit: bool,
    queue_handle: QueueHandle<Self>,
    registry_state: RegistryState,
    activation_state: Option<ActivationState>,
    tx: UnboundedSender<TokenUpdate>,
    seat_state: SeatState,
}

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    sctk::registry_handlers!();
}

struct ExecRequestData {
    data: RequestData,
    exec: String,
}

impl RequestDataExt for ExecRequestData {
    fn app_id(&self) -> Option<&str> {
        self.data.app_id()
    }

    fn seat_and_serial(&self) -> Option<(&WlSeat, u32)> {
        self.data.seat_and_serial()
    }

    fn surface(&self) -> Option<&WlSurface> {
        self.data.surface()
    }
}

impl ActivationHandler for AppData {
    type RequestData = ExecRequestData;
    fn new_token(&mut self, token: String, data: &ExecRequestData) {
        let _ = self.tx.unbounded_send(TokenUpdate::ActivationToken {
            token: Some(token),
            exec: data.exec.clone(),
        });
    }
}

impl SeatHandler for AppData {
    fn seat_state(&mut self) -> &mut sctk::seat::SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        _: sctk::seat::Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        _: sctk::seat::Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
}

pub(crate) fn wayland_handler(
    tx: UnboundedSender<TokenUpdate>,
    rx: calloop::channel::Channel<TokenRequest>,
) {
    let socket = std::env::var("X_PRIVILEGED_WAYLAND_SOCKET")
        .ok()
        .and_then(|fd| {
            fd.parse::<RawFd>()
                .ok()
                .map(|fd| unsafe { UnixStream::from_raw_fd(fd) })
        });

    let conn = if let Some(socket) = socket {
        Connection::from_socket(socket).unwrap()
    } else {
        Connection::connect_to_env().unwrap()
    };
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();

    let mut event_loop = calloop::EventLoop::<AppData>::try_new().unwrap();
    let qh = event_queue.handle();
    let wayland_source = WaylandSource::new(conn, event_queue);
    let handle = event_loop.handle();
    wayland_source
        .insert(handle.clone())
        .expect("Failed to insert wayland source.");

    if handle
        .insert_source(rx, |event, _, state| match event {
            calloop::channel::Event::Msg(TokenRequest { app_id, exec }) => {
                if let Some(activation_state) = state.activation_state.as_ref() {
                    activation_state.request_token_with_data(
                        &state.queue_handle,
                        ExecRequestData {
                            data: RequestData {
                                app_id: Some(app_id),
                                seat_and_serial: state
                                    .seat_state
                                    .seats()
                                    .next()
                                    .map(|seat| (seat, 0)),
                                surface: None,
                            },
                            exec,
                        },
                    );
                } else {
                    let _ = state
                        .tx
                        .unbounded_send(TokenUpdate::ActivationToken { token: None, exec });
                }
            }
            calloop::channel::Event::Closed => {
                state.exit = true;
            }
        })
        .is_err()
    {
        return;
    }
    let registry_state = RegistryState::new(&globals);
    let mut app_data = AppData {
        exit: false,
        tx,
        seat_state: SeatState::new(&globals, &qh),
        activation_state: ActivationState::bind::<AppData>(&globals, &qh).ok(),
        queue_handle: qh,
        registry_state,
    };

    loop {
        if app_data.exit {
            break;
        }
        event_loop.dispatch(None, &mut app_data).unwrap();
    }
}

sctk::delegate_activation!(AppData, ExecRequestData);
sctk::delegate_seat!(AppData);
sctk::delegate_registry!(AppData);
