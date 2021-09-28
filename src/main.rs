mod minefield;
mod search;

use minefield::Minefield;
use std::time::{Instant, Duration};
use iced::{executor, Application, Command, Clipboard, Element, Column, Row, Button, button, Text, Space, Svg};
use itertools::izip;

thread_local!(static FLAG: iced::svg::Handle = iced::svg::Handle::from_path("resources/flag.svg"));

enum GameState {
    BeforeStarted,
    Running{start_time: Instant},
    EndGame{game_duration: Duration}
}

#[derive(Debug, Clone)]
enum Message {
    Reveal(u8, u8),
    Mark(u8, u8)
}

struct Minesweeper {
    minefield: Minefield,
    button_grid: Vec<Vec<button::State>>,
    state: GameState
}

fn create_button<'a>(state: &'a mut button::State, tile: &minefield::Tile) -> Button<'a, Message>
{
    match tile {
        minefield::Tile::Hidden(_, minefield::UserMarking::None) =>
            Button::new(state, Space::new(iced::Length::Fill, iced::Length::Fill)),
        minefield::Tile::Hidden(_, minefield::UserMarking::Flag) =>
            Button::new(state, Svg::new(FLAG.with(|f| f.clone()))),
        minefield::Tile::Hidden(_, minefield::UserMarking::QuestionMark) =>
            Button::new(state, Text::new("?")),
        minefield::Tile::Revealed(clue) => Button::new(state, Text::new(clue.to_string())),
    }
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

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::Reveal(row, col) => {
                if ! self.minefield.reveal(row, col) {
                    println!("BOOOM!");
                }
            },
            Message::Mark(row, col) => self.minefield.switch_mark(row, col)
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut grid = Column::new();
        for (row, states, tiles) in izip!(0.., self.button_grid.iter_mut(), self.minefield.grid.iter()) {
            let mut view_row = Row::new();
            for (col, state, tile) in izip!(0.., states.iter_mut(), tiles.iter()) {
                view_row = view_row.push(create_button(state, tile)
                    .width(iced::Length::Units(30))
                    .height(iced::Length::Units(30))
                    .on_press(Message::Mark(row, col)));
            }
            grid = grid.push(view_row);
        }
        grid.into()
    }
}

fn main() -> iced::Result {
    Minesweeper::run(iced::Settings::default())
}
