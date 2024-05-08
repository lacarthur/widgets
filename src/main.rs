use iced::{
    executor, widget::text, Command, Element, Subscription, Theme
};

use iced_layershell::{
    reexport::Anchor,
    settings::{LayerShellSettings, Settings},
    Application,
};

use widgets::hyprland::{subscription::HyprlandWorkspaceEvent, ui::{WorkspaceDisplay, WorkspaceDisplayMessage}};

use log::error;

#[derive(Debug, Clone)]
enum ApplicationMessage {
    WorkspaceMessage(WorkspaceDisplayMessage),
}

/// the main app, that represents all of the widgets
struct MyWidgets {
    workspace_display: Option<WorkspaceDisplay>,
}

impl Application for MyWidgets {
    type Executor = executor::Default;
    type Message = ApplicationMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let workspace_display = match WorkspaceDisplay::create_from_commands() {
            Err(e) => {
                error!("Error communicating with Hyprland : {}", e);
                None
            }
            Ok(workspace_display) => Some(workspace_display),
        };

    
        (
            Self {
                workspace_display,
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("MyWidgets")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            ApplicationMessage::WorkspaceMessage(WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::Error)) => {
                self.workspace_display = None;
            }
            ApplicationMessage::WorkspaceMessage(msg) => {
                if let Some(display) = self.workspace_display.as_mut() {
                    display.update(msg);
                }
            }
        };
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        if let Some(workspace_display) = &self.workspace_display {
            workspace_display.view().map(ApplicationMessage::WorkspaceMessage)
        } else {
            text("Workspaces aren't working. Check the logs.").into()
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        if let Some(workspace_display) = &self.workspace_display {
            workspace_display.subscription().map(ApplicationMessage::WorkspaceMessage)
        } else {
            Subscription::none()
        }
    }
}


fn main() -> Result<(), iced_layershell::Error> {
    env_logger::init();

    MyWidgets::run(Settings {
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
