use iced::{subscription, Subscription};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::Lines;
use tokio::net::UnixStream;

use crate::hyprland::get_hyprland_socket_address;

use super::HyprlandCommunicationError;

// right now, very simple. I don't use anything more
// if the hyprland IPC API changes in the future, might not be needed, but right now some events
// don't give the window id, just the name
// also, hyprland ids start at 1
// TODO : make this into an error
fn workspace_name_to_id(name: &str) -> usize {
    name.parse::<usize>().unwrap() - 1
}

/// An event, as communicated by Hyprland through the socket
#[derive(Debug, Clone)]
pub enum HyprlandWorkspaceEvent {
    // do nothing
    Noop,
    ChangeActiveWorkspace {
        new_workspace_id: usize,
    },
    OpenWindow {
        window_address: u64,
        workspace_id: usize,
    },
    CloseWindow {
        window_address: u64,
    },
    MoveWindow {
        window_address: u64,
        new_workspace_id: usize,
    },
    // we don't send the contents of the error, because the main UI doesn't need to know it. It
    // just needs to display that there has been an error.
    Error,
}

enum SubscriptionState {
    Starting,
    Ongoing {
        reader: Lines<BufReader<UnixStream>>,
    },
    // if we're in this state, it should have been communicated to the widgets manager that should
    // just kill the subscription (i.e. stop returning it)
    Error,
}

/// start an async task in the background that listens to the socket Hyprland uses to communicate
/// events. Each relevant event is then sent as a message to the relevant part of the ui.
pub fn connect_to_socket() -> Subscription<HyprlandWorkspaceEvent> {
    struct SocketConnection;

    subscription::unfold(
        std::any::TypeId::of::<SocketConnection>(),
        SubscriptionState::Starting,
        move |state| async move {
            match state {
                SubscriptionState::Starting => {
                    let socket_path = match get_hyprland_socket_address() {
                        Ok(path) => path,
                        Err(e) => {
                            log::error!("{}", e);
                            return (HyprlandWorkspaceEvent::Error, SubscriptionState::Error);
                        }
                    };
                    match UnixStream::connect(&socket_path).await {
                        Ok(stream) => {
                            let reader = BufReader::new(stream).lines();
                            (
                                // iced requires us to send a message here, and I think it's cleaner to
                                // send a 'do nothing' message here rather than wait here for the first
                                // message for however long that takes
                                HyprlandWorkspaceEvent::Noop,
                                SubscriptionState::Ongoing { reader },
                            )
                        }
                        Err(error) => {
                            let e = HyprlandCommunicationError::SocketConnectionError { socket_path, error };
                            log::error!("{}", e);
                            (
                                HyprlandWorkspaceEvent::Error,
                                SubscriptionState::Error,
                            )
                        }
                    }
                }
                SubscriptionState::Ongoing { mut reader } => {
                    while let Some(line) = reader.next_line().await.unwrap() {
                        match parse_hyprland_event(&line) {
                            Ok(Some(command)) => return (command, SubscriptionState::Ongoing { reader }),
                            Err(e) => {
                                log::error!("{}", e);
                                return (HyprlandWorkspaceEvent::Error, SubscriptionState::Error)
                            }
                            _ => ()
                        }
                    }
                    unreachable!();
                },
                SubscriptionState::Error => {
                    iced::futures::future::pending().await
                }
            }
        },
    )
}

/// function that parses the hyprland events as they are sent to the socket. It also filters the
/// events that we want. When we receive an event that we don't care about, (for instance a window
/// has been fullscreened), it just sends back None. We do this rather than
/// `HyprlandWorkspaceEvent::Noop` because a `Noop` would get sent, and it doesn't need to.
/// TODO : parse better. use nom?
fn parse_hyprland_event(line: &str) -> Result<Option<HyprlandWorkspaceEvent>, HyprlandCommunicationError> {
    let mut line_split = line.split(">>");
    let command = line_split.next().ok_or(HyprlandCommunicationError::EventParsingError { event: line.into() })?;
    let args = line_split.next().unwrap_or_default();
    let mut split_args = args.split(',');

    match command {
        "workspace" => {
            let workspace_id = split_args.next().ok_or(HyprlandCommunicationError::EventArgsParsingError { event: line.into(), args: args.into() })?;
            Ok(Some(HyprlandWorkspaceEvent::ChangeActiveWorkspace {
                new_workspace_id: workspace_name_to_id(workspace_id),
            }))
        }
        "openwindow" => {
            let address_str = split_args.next()
                .ok_or(HyprlandCommunicationError::EventArgsParsingError { event: line.into(), args: args.into() })?;
            let workspace_id = split_args.next()
                .ok_or(HyprlandCommunicationError::EventArgsParsingError { event: line.into(), args: args.into() })?;
            let window_address = u64::from_str_radix(address_str, 16)
                .map_err(|error| HyprlandCommunicationError::WindowAddressParsingError { command: line.into(), address: address_str.into(), error})?;
            Ok(Some(HyprlandWorkspaceEvent::OpenWindow {
                window_address,
                workspace_id: workspace_name_to_id(workspace_id),
            }))
        }
        "closewindow" => {
            let address_str = split_args.next()
                .ok_or(HyprlandCommunicationError::EventArgsParsingError { event: line.into(), args: args.into() })?;
            let window_address = u64::from_str_radix(address_str, 16)
                .map_err(|error| HyprlandCommunicationError::WindowAddressParsingError { command: line.into(), address: address_str.into(), error})?;
            Ok(Some(HyprlandWorkspaceEvent::CloseWindow { window_address }))
        }
        "movewindow" => {
            let address_str = split_args.next()
                .ok_or(HyprlandCommunicationError::EventArgsParsingError { event: line.into(), args: args.into() })?;
            let window_address = u64::from_str_radix(address_str, 16)
                .map_err(|error| HyprlandCommunicationError::WindowAddressParsingError { command: line.into(), address: address_str.into(), error})?;
            let workspace_id = split_args.next()
                .ok_or(HyprlandCommunicationError::EventArgsParsingError { event: line.into(), args: args.into() })?;
            Ok(Some(HyprlandWorkspaceEvent::MoveWindow {
                window_address,
                new_workspace_id: workspace_name_to_id(workspace_id),
            }))
        }
        _ => Ok(None),
    }
}
