use iced::{subscription, Subscription};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::Lines;
use tokio::net::UnixStream;

// right now, very simple. I don't use anything more
// if the hyprland IPC API changes in the future, might not be needed, but right now some events
// don't give the window id, just the name
// also, hyprland ids start at 1
fn workspace_name_to_id(name: String) -> usize {
    name.parse::<usize>().unwrap() - 1
}

#[derive(Debug, Clone)]
pub enum HyprlandWorkspaceEvent {
    Noop,
    ChangeWorkspace {
        workspace_id: usize,
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
        workspace_id: usize,
    },
}

enum SubscriptionState {
    Starting,
    Ready {
        reader: Lines<BufReader<UnixStream>>,
    },
}

pub fn connect_to_socket() -> Subscription<HyprlandWorkspaceEvent> {
    struct SocketConnection;

    subscription::unfold(
        std::any::TypeId::of::<SocketConnection>(),
        SubscriptionState::Starting,
        move |state| async move {
            match state {
                SubscriptionState::Starting => {
                    let hyprland_instance_signature =
                        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
                    let socket_path =
                        format!("/tmp/hypr/{}/.socket2.sock", hyprland_instance_signature);
                    let stream = UnixStream::connect(socket_path).await.unwrap();
                    let reader = BufReader::new(stream).lines();
                    (
                        HyprlandWorkspaceEvent::Noop,
                        SubscriptionState::Ready { reader },
                    )
                }
                SubscriptionState::Ready { mut reader } => {
                    while let Some(line) = reader.next_line().await.unwrap() {
                        if let Some(command) = parse_hyprland_event(&line) {
                            return (command, SubscriptionState::Ready { reader });
                        }
                    }
                    unreachable!();
                }
            }
        },
    )
}

fn parse_hyprland_event(line: &str) -> Option<HyprlandWorkspaceEvent> {
    let mut line_split = line.split(">>");
    let command = line_split.next().unwrap();
    let args = line_split.next().unwrap_or_default();
    let mut split_args = args.split(',');

    match command {
        "workspace" => Some(HyprlandWorkspaceEvent::ChangeWorkspace {
            workspace_id: workspace_name_to_id(split_args.next().unwrap().into()),
        }),
        "openwindow" => {
            let address_str = split_args.next().unwrap();
            let window_address = u64::from_str_radix(address_str, 16).unwrap();
            Some(HyprlandWorkspaceEvent::OpenWindow {
                window_address,
                workspace_id: workspace_name_to_id(split_args.next().unwrap().into()),
            })
        }
        "closewindow" => {
            let address_str = split_args.next().unwrap();
            let window_address = u64::from_str_radix(address_str, 16).unwrap();
            Some(HyprlandWorkspaceEvent::CloseWindow { window_address })
        }
        "movewindow" => {
            let address_str = split_args.next().unwrap();
            let window_address = u64::from_str_radix(address_str, 16).unwrap();
            Some(HyprlandWorkspaceEvent::MoveWindow {
                window_address,
                workspace_id: workspace_name_to_id(split_args.next().unwrap().into()),
            })
        }
        _ => None,
    }
}

#[derive(Serialize, Deserialize)]
struct HyprlandWorkspaceFull {
    id: usize,
    windows: u32,
}
