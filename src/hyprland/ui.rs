use super::{
    get_active_workspace, get_windows, subscription::HyprlandWorkspaceEvent, switch_to_workspace,
    HyprlandCommunicationError, NUM_WORKSPACES,
};
use std::collections::HashMap;

use iced::{
    alignment::Horizontal,
    color,
    widget::{button, text, Button, Row},
    Border, Color, Element, Length, Padding, Radius,
};

use log::error;

#[derive(Debug, Clone)]
pub enum WorkspaceDisplayMessage {
    EventReceived(HyprlandWorkspaceEvent),
    WorkspaceButtonClicked(usize),
}

pub struct WorkspaceDisplay {
    active_workspace: usize,
    /// How many windows there are in each workspace.
    window_count: [u32; NUM_WORKSPACES],
    /// The workspace where the window is, indexed by the window address
    windows: HashMap<u64, usize>,
}

impl WorkspaceDisplay {
    /// create a `WorkspaceDisplay` and initialize it with the current values by querying
    /// `hyprctl`.
    pub fn create_from_commands() -> Result<Self, HyprlandCommunicationError> {
        let active_workspace = get_active_workspace()?;
        let (windows, window_count) = get_windows()?;
        Ok(Self {
            active_workspace,
            window_count,
            windows,
        })
    }

    pub fn update(&mut self, message: WorkspaceDisplayMessage) {
        match message {
            WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::MoveWindow {
                window_address,
                new_workspace_id,
            }) => {
                match self.windows.insert(window_address, new_workspace_id) {
                    Some(previous_workspace) => {
                        self.window_count[new_workspace_id] += 1;
                        self.window_count[previous_workspace] -= 1;
                    }
                    None => {
                        let e = HyprlandCommunicationError::RequestInexistantWindow {
                            requested_address: window_address,
                            addresses_in_memory: self.windows.keys().copied().collect(),
                        };
                        log::error!("{}", e);
                        // TODO switch to error mode or sth
                    }
                }
            }
            WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::OpenWindow {
                window_address,
                workspace_id,
            }) => {
                self.window_count[workspace_id] += 1;
                self.windows.insert(window_address, workspace_id);
            }
            WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::CloseWindow {
                window_address,
            }) => match self.windows.remove(&window_address) {
                Some(workspace_id) => self.window_count[workspace_id] -= 1,
                None => {
                    let e = HyprlandCommunicationError::RequestInexistantWindow {
                        requested_address: window_address,
                        addresses_in_memory: self.windows.keys().copied().collect(),
                    };
                    log::error!("{}", e);
                }
            },
            WorkspaceDisplayMessage::EventReceived(
                HyprlandWorkspaceEvent::ChangeActiveWorkspace { new_workspace_id },
            ) => {
                self.active_workspace = new_workspace_id;
            }
            WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::Error) => {
                // this is handled above, and so it is a noop here
            }
            WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::Noop) => (),
            WorkspaceDisplayMessage::WorkspaceButtonClicked(id) => {
                if let Err(e) = switch_to_workspace(id) {
                    error!("{}", e);
                    std::process::abort();
                }
            }
        }
    }

    pub fn view(&self) -> Element<WorkspaceDisplayMessage> {
        let buttons = self
            .window_count
            .iter()
            .enumerate()
            .map(|(id, &window_count)| {
                let val = format!("{}", id + 1);
                let style = if window_count == 0 {
                    color!(0x808080)
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
                .padding(Padding::from(0))
                .width(Length::Fixed(25.0))
                .into()
            })
            .collect::<Vec<Element<_>>>();

        Row::with_children(buttons).into()
    }

    pub fn subscription(&self) -> iced::Subscription<WorkspaceDisplayMessage> {
        crate::hyprland::subscription::connect_to_socket()
            .map(WorkspaceDisplayMessage::EventReceived)
    }
}

struct ActiveWorkspaceButtonStyle;
struct InactiveWorkspaceButtonStyle;

impl button::StyleSheet for ActiveWorkspaceButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(color!(0x665c54))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(5.0),
            },
            ..Default::default()
        }
    }
}

impl button::StyleSheet for InactiveWorkspaceButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(color!(0x282828))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(5.0),
            },
            ..Default::default()
        }
    }
}
