use iced::{
    executor, widget::{row, text}, Command, Element, Subscription, Theme
};

use iced_layershell::{
    reexport::Anchor,
    settings::{LayerShellSettings, Settings},
    Application,
};

use widgets::{battery_display::{BatteryDisplay, BatteryMessage}, clock::{Clock, ClockMessage}, hyprland::{subscription::HyprlandWorkspaceEvent, ui::{WorkspaceDisplay, WorkspaceDisplayMessage}}};

use log::error;

#[derive(Debug, Clone)]
enum ApplicationMessage {
    Workspace(WorkspaceDisplayMessage),
    Clock(ClockMessage),
    Battery(BatteryMessage),
}

/// the main app, that represents all of the widgets
struct MyWidgets {
    workspace_display: Option<WorkspaceDisplay>,
    battery_display: Option<BatteryDisplay>,
    clock: Clock,
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
                clock: Clock::default(),
                battery_display: BatteryDisplay::new(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("MyWidgets")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            ApplicationMessage::Workspace(WorkspaceDisplayMessage::EventReceived(HyprlandWorkspaceEvent::Error)) => {
                self.workspace_display = None;
            }
            ApplicationMessage::Workspace(msg) => {
                if let Some(display) = self.workspace_display.as_mut() {
                    display.update(msg);
                }
            }
            ApplicationMessage::Clock(msg) => {
                self.clock.update(msg);
            }
            ApplicationMessage::Battery(BatteryMessage::Error) => self.battery_display = None,
            ApplicationMessage::Battery(msg) => {
                if let Some(display) = self.battery_display.as_mut() {
                    display.update(msg);
                }
            }
        };
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let workspace = if let Some(workspace_display) = &self.workspace_display {
            workspace_display.view().map(ApplicationMessage::Workspace)
        } else {
            text("Workspaces aren't working. Check the logs.").into()
        };

        let battery = if let Some(bat_display) = &self.battery_display {
            bat_display.view().map(ApplicationMessage::Battery)
        } else {
            text("Battery isn't working. Check the logs.").into()
        };

        let clock = self.clock.view().map(ApplicationMessage::Clock);

        row!(
            workspace, 
            clock, 
            battery,
        ).into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let workspace_subscription = if let Some(workspace_display) = &self.workspace_display {
            workspace_display.subscription().map(ApplicationMessage::Workspace)
        } else {
            Subscription::none()
        };

        let battery_subscription = if let Some(battery_display) = &self.battery_display {
            battery_display.subscription().map(ApplicationMessage::Battery)
        } else {
            Subscription::none()
        };

        let clock_subscription = self.clock.subscription().map(ApplicationMessage::Clock);

        Subscription::batch([
            workspace_subscription, 
            battery_subscription, 
            clock_subscription,
        ])

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
