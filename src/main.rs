use iced::{
    color, executor,
    wayland::layer_surface::Anchor,
    widget::{container, horizontal_space, row, text, Container},
    Application, Background, Command, Element, Length, Settings, Subscription, Theme,
};

use widgets::{
    battery_display::{BatteryDisplay, BatteryMessage},
    clock::{Clock, ClockMessage},
    hyprland::{
        subscription::HyprlandWorkspaceEvent,
        ui::{WorkspaceDisplay, WorkspaceDisplayMessage},
    },
};

use log::error;

const HEIGHT: u32 = 25;
const MARGIN: u32 = 5;
const SCREEN_WIDTH: u32 = 1366;

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

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            ApplicationMessage::Workspace(WorkspaceDisplayMessage::EventReceived(
                HyprlandWorkspaceEvent::Error,
            )) => {
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

    fn view(&self, _id: iced::window::Id) -> Element<Self::Message> {
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

        Container::new(row!(
            Container::new(workspace).width(Length::FillPortion(1)),
            Container::new(row![
                horizontal_space(Length::Fill),
                clock,
                horizontal_space(Length::Fill)
            ])
            .width(Length::FillPortion(1)),
            Container::new(row![horizontal_space(Length::Fill), battery])
                .width(Length::FillPortion(1)),
        ))
        .style(iced::theme::Container::Custom(Box::new(MainContainerStyle)))
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let workspace_subscription = if let Some(workspace_display) = &self.workspace_display {
            workspace_display
                .subscription()
                .map(ApplicationMessage::Workspace)
        } else {
            Subscription::none()
        };

        let battery_subscription = if let Some(battery_display) = &self.battery_display {
            battery_display
                .subscription()
                .map(ApplicationMessage::Battery)
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

    fn title(&self, _id: iced::window::Id) -> String {
        String::from("Widgets")
    }
}

struct MainContainerStyle;

impl container::StyleSheet for MainContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(color!(0x282828))),
            ..Default::default()
        }
    }
}

fn main() -> Result<(), iced::Error> {
    env_logger::init();

    MyWidgets::run(Settings {
        initial_surface: iced::wayland::InitialSurface::LayerSurface(
            iced::wayland::actions::layer_surface::SctkLayerSurfaceSettings {
                layer: iced::wayland::layer_surface::Layer::Background,
                anchor: Anchor::TOP,
                size: Some((Some(SCREEN_WIDTH - 2 * MARGIN), Some(HEIGHT))),
                exclusive_zone: HEIGHT as i32,
                ..Default::default()
            }
        ),
        default_text_size: iced::Pixels(17.0),
        ..Default::default()
    })
}
