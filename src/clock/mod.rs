use chrono::{DateTime, Datelike, Local, NaiveDate};
use iced::{widget::{text, Button}, Element, Subscription};

/// The state of the clock widget
enum State {
    // display the current date
    Date,
    // display the current time
    Time,
}

pub struct Clock {
    now: DateTime<Local>,
    state: State,
}

#[derive(Debug, Clone, Copy)]
pub enum ClockMessage {
    Tick(DateTime<Local>),
    ChangeState,
}

fn format_date(date: NaiveDate) -> String {
    let day_of_the_week = match date.weekday() {
        chrono::Weekday::Mon => "Lundi",
        chrono::Weekday::Tue => "Mardi",
        chrono::Weekday::Wed => "Mercredi",
        chrono::Weekday::Thu => "Jeudi",
        chrono::Weekday::Fri => "Vendredi",
        chrono::Weekday::Sat => "Samedi",
        chrono::Weekday::Sun => "Dimanche",
    };

    let month =  match date.month0() {
        0 => "janvier",
        1 => "février",
        2 => "mars",
        3 => "avril",
        4 => "mai",
        5 => "juin",
        6 => "juillet",
        7 => "août",
        8 => "septembre",
        9 => "octobre",
        10 => "novembre",
        11 => "décembre",
        _ => unreachable!(),
    };

    format!("{} {} {}", day_of_the_week, date.day0() + 1, month)
}

impl Clock {
    pub fn update(&mut self, message: ClockMessage) {
        match message {
            ClockMessage::Tick(new_time) => self.now = new_time,
            ClockMessage::ChangeState => {
                match self.state {
                    State::Date => self.state = State::Time,
                    State::Time => self.state = State::Date,
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<ClockMessage> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| {
            ClockMessage::Tick(
                Local::now()
            )
        })
    }

    pub fn view(&self) -> Element<ClockMessage> {
        let button_text = match self.state {
            State::Date => format_date(self.now.date_naive()),
            State::Time => self.now.time().format("%H:%M").to_string(),
        };
        Button::new(text(button_text))
            .on_press(ClockMessage::ChangeState)
            .into()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self { now: Local::now(), state: State::Time }
    }
}
