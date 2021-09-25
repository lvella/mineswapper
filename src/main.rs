mod minefield;

use minefield::Minefield;
use std::time::{Instant, Duration};
use iced::{executor, Application, Command, Clipboard, Element, Column, Row, Button, button, Text};

enum GameState {
    BeforeStarted,
    Running{start_time: Instant},
    EndGame{game_duration: Duration}
}

#[derive(Debug, Clone)]
struct Message {}

struct Minesweeper {
    minefield: Minefield,
    button_grid: Vec<Vec<button::State>>,
    state: GameState
}

impl Application for Minesweeper {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        const WIDTH: u8 = 30;
        const HEIGHT: u8 = 16;
        let size = usize::from(WIDTH) * usize::from(HEIGHT);
        (
            Self {
                minefield: Minefield::create_random(WIDTH, HEIGHT, 99),
                button_grid: vec![button::State::new(); size]
                    .chunks(usize::from(WIDTH)).map(|x| x.to_vec()).collect(),
                state: GameState::BeforeStarted
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Non-deterministic Minesweeper")
    }

    fn update(&mut self, _message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut grid = Column::new();
        for row in self.button_grid.iter_mut() {
            let mut view_row = Row::new();
            for button_state in row.iter_mut() {
                view_row = view_row.push(Button::new(button_state, Text::new(""))
                    .width(iced::Length::Units(30))
                    .height(iced::Length::Units(30))
                    .on_press(Message{}));
            }
            grid = grid.push(view_row);
        }
        grid.into()
    }
}

fn main() -> iced::Result {
    Minesweeper::run(iced::Settings::default())
}
