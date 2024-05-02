use std::collections::HashMap;

use iced::{
    executor, alignment::Horizontal, Color, Command, 
    Element, Theme,
    widget::{button, text, Button, Row},
};

use iced_layershell::{
    reexport::Anchor,
    settings::{LayerShellSettings, Settings},
    Application,
};

use widgets::hyprland_workspaces::{self, HyprlandWorkspaceEvent};

use serde::Deserialize;

// I am not dealing with dynamic workspaces.
const NUM_WORKSPACES: usize = 10;

struct WorkspaceDisplay {
    active_workspace: usize,
    window_count: [u32; NUM_WORKSPACES],
    windows: HashMap<u64, usize>,
}

#[derive(Deserialize, Debug)]
struct WorkspaceDeserialized {
    id: usize,
}

#[derive(Deserialize, Debug)]
struct HyprlandClientDeserialized {
    address: String,
    workspace: WorkspaceDeserialized,
}

fn get_windows() -> (HashMap<u64, usize>, [u32; NUM_WORKSPACES]) {
    let data = std::process::Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output()
        .unwrap()
        .stdout;

    let data = std::str::from_utf8(&data).unwrap();

    let clients: Vec<HyprlandClientDeserialized> = serde_json::from_str(data).unwrap();

    let mut windows = HashMap::new();
    let mut count_windows = [0; NUM_WORKSPACES];

    for client in clients {
        let id = client.workspace.id - 1;
        count_windows[id] += 1;
        let address = client.address.strip_prefix("0x").unwrap();
        let address = u64::from_str_radix(address, 16).unwrap();
        windows.insert(address, id);
    }
    (windows, count_windows)
}

fn get_active_workspace() -> usize {
    let data = std::process::Command::new("hyprctl")
        .arg("activeworkspace")
        .arg("-j")
        .output()
        .unwrap()
        .stdout;

    let data = std::str::from_utf8(&data).unwrap();

    let active_workspace: WorkspaceDeserialized = serde_json::from_str(data).unwrap();

    active_workspace.id - 1
}

#[derive(Debug, Clone)]
enum WorkspaceDisplayMessage {
    Event(HyprlandWorkspaceEvent),
    WorkspaceButtonClicked(usize),
}

impl Application for WorkspaceDisplay {
    type Executor = executor::Default;
    type Message = WorkspaceDisplayMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let active_workspace = get_active_workspace();
        let (windows, window_count) = get_windows();
        (
            Self {
                active_workspace,
                window_count,
                windows,
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("BarWorkspaces")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            WorkspaceDisplayMessage::Event(HyprlandWorkspaceEvent::MoveWindow {
                window_address,
                workspace_id,
            }) => {
                let previous_workspace = self.windows.insert(window_address, workspace_id).unwrap();
                self.window_count[workspace_id] += 1;
                self.window_count[previous_workspace] -= 1;
            }
            WorkspaceDisplayMessage::Event(HyprlandWorkspaceEvent::OpenWindow {
                window_address,
                workspace_id,
            }) => {
                self.window_count[workspace_id] += 1;
                self.windows.insert(window_address, workspace_id);
            }
            WorkspaceDisplayMessage::Event(HyprlandWorkspaceEvent::CloseWindow {
                window_address,
            }) => {
                let workspace_id = self.windows.remove(&window_address).unwrap();
                self.window_count[workspace_id] -= 1;
            }
            WorkspaceDisplayMessage::Event(HyprlandWorkspaceEvent::ChangeWorkspace {
                workspace_id,
            }) => {
                self.active_workspace = workspace_id;
            }
            WorkspaceDisplayMessage::Event(HyprlandWorkspaceEvent::Noop) => (),
            WorkspaceDisplayMessage::WorkspaceButtonClicked(id) => {
                std::process::Command::new("hyprctl")
                    .arg("dispatch")
                    .arg(format!("workspace {}", id + 1))
                    .status()
                    .unwrap();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let buttons = self
            .window_count
            .iter()
            .enumerate()
            .map(|(id, &window_count)| {
                let val = format!("{}", id + 1);
                let style = if window_count == 0 {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
                Button::new(
                    text(val)
                        .horizontal_alignment(Horizontal::Center)
                        .style(style),
                )
                .height(30)
                .width(30)
                .on_press(WorkspaceDisplayMessage::WorkspaceButtonClicked(id))
                .style(if id == self.active_workspace {
                    iced::theme::Button::custom(ActiveWorkspaceButtonStyle {})
                } else {
                    iced::theme::Button::custom(InactiveWorkspaceButtonStyle {})
                })
                .into()
            })
            .collect::<Vec<Element<_>>>();

        Row::from_vec(buttons).into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        hyprland_workspaces::connect_to_socket().map(WorkspaceDisplayMessage::Event)
    }
}

struct ActiveWorkspaceButtonStyle;
struct InactiveWorkspaceButtonStyle;

impl button::StyleSheet for ActiveWorkspaceButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: Some(iced::Background::Color(Color::new(1.0, 1.0, 0.0, 1.0))),
            text_color: Color::WHITE,
            border: Default::default(),
            shadow: Default::default(),
        }
    }
}

impl button::StyleSheet for InactiveWorkspaceButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: Some(iced::Background::Color(Color::new(0.0, 1.0, 1.0, 1.0))),
            text_color: Color::WHITE,
            border: Default::default(),
            shadow: Default::default(),
        }
    }
}

fn main() -> Result<(), iced_layershell::Error> {
    WorkspaceDisplay::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((1356, 30)),
            exclusize_zone: 30,
            anchor: Anchor::Top | Anchor::Right | Anchor::Left,
            ..Default::default()
        },
        default_text_size: iced::Pixels(15.0),
        ..Default::default()
    })
}
