use battery::{
    errors::Error, units::ratio::percent, Manager, State
};
use iced::{color, widget::{column, container, text, vertical_space, Container}, Element, Length, Padding, Subscription};

pub struct BatteryDisplay {
    state: State,
    percent_charge: u32,
}

fn get_state_and_percent() -> Result<(State, u32), Error> {
    let manager = Manager::new()?;
    let battery = manager.batteries()?.next().unwrap()?;

    let percent_val = battery.state_of_charge().get::<percent>();

    let state = battery.state();

    Ok((state, percent_val as u32))
}

#[derive(Clone, Copy, Debug)]
pub enum BatteryMessage {
    NewState(State, u32),
    Error,
}

impl BatteryDisplay {
    pub fn new() -> Option<Self> {
        match get_state_and_percent() {
            Ok((State::Unknown, _)) => {
                log::error!("Unable to get battery information.");
                None
            }
            Ok((state, percent_charge)) => {
                Some(Self {
                    state,
                    percent_charge,
                })
            }
            Err(e) => {
                log::error!("Unable to get battery information: {}", e);
                None
            }
        }
    }

    fn icon(&self) -> char {
        match self.state{
            State::Charging if self.percent_charge <= 10 => '󰢟',
            State::Charging if self.percent_charge <= 20 => '󰢜',
            State::Charging if self.percent_charge <= 30 => '󰂆',
            State::Charging if self.percent_charge <= 40 => '󰂆',
            State::Charging if self.percent_charge <= 50 => '󰂈',
            State::Charging if self.percent_charge <= 60 => '󰢝',
            State::Charging if self.percent_charge <= 70 => '󰂉',
            State::Charging if self.percent_charge <= 80 => '󰢞',
            State::Charging if self.percent_charge <= 90 => '󰂊',
            State::Charging if self.percent_charge <= 99 => '󰂋',
            State::Charging => '󰂅',
            State::Discharging if self.percent_charge <= 10 => '󰂎',
            State::Discharging if self.percent_charge <= 20 => '󰁺',
            State::Discharging if self.percent_charge <= 30 => '󰁻',
            State::Discharging if self.percent_charge <= 40 => '󰁼',
            State::Discharging if self.percent_charge <= 50 => '󰁽',
            State::Discharging if self.percent_charge <= 60 => '󰁽',
            State::Discharging if self.percent_charge <= 70 => '󰁽',
            State::Discharging if self.percent_charge <= 80 => '󰂀',
            State::Discharging if self.percent_charge <= 90 => '󰂁',
            State::Discharging if self.percent_charge <= 99 => '󰂂',
            State::Discharging => '󰁹',
            State::Empty => '󰂎',
            State::Full => '󰁹',
            _ => unreachable!(),
        }
    }

    pub fn update(&mut self, message: BatteryMessage) {
        if let BatteryMessage::NewState(state,percent_charge) = message {
            self.state = state;
            self.percent_charge = percent_charge;
        }
    }

    pub fn subscription(&self) -> Subscription<BatteryMessage> {
        iced::time::every(std::time::Duration::from_millis(600)).map(|_| {
            match get_state_and_percent() {
                Ok((State::Unknown, _)) => {
                    log::error!("Unable to access battery information.");
                    BatteryMessage::Error
                }
                Ok((state, percent_charge)) => BatteryMessage::NewState(state, percent_charge),
                Err(e) => {
                    log::error!("Unable to access battery information : {}", e);
                    BatteryMessage::Error
                }
            }
        })
    }

    pub fn view(&self) -> Element<BatteryMessage> {
        Container::new(
            column![
                vertical_space(Length::Fill),
                text(format!("{} {}%", self.icon(), self.percent_charge)),
                vertical_space(Length::Fill),
            ]
        ).padding(Padding::from([0, 5, 0, 5]))
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle {})))
            .into()
    }
}

struct ContainerStyle;

impl container::StyleSheet for ContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(iced::Color::WHITE),
            background: Some(iced::Background::Color(color!(0x282828))),
            ..Default::default()
        }
    }

}
