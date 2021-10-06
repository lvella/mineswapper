mod minefield;
mod neighbor_iter;
mod solver;
mod search;
mod right_clickable;

use minefield::Minefield;
use std::time::{Instant, Duration};
use iced::{executor, Application, Command, Clipboard, Element, Column, Row, Button, button, Text, Space, Svg};
use itertools::izip;
use right_clickable::RightClickable;
use strum_macros;

thread_local!(static FLAG: iced::svg::Handle = iced::svg::Handle::from_path("resources/flag.svg"));

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum_macros::EnumIter)]
enum DifficultyLevels {
    Beginner,
    Intermediate,
    Expert
}

impl DifficultyLevels {
    fn rows(self) -> u8
    {
        match self {
            Self::Beginner => 9,
            Self::Intermediate => 16,
            Self::Expert => 16
        }
    }

    fn cols(self) -> u8
    {
        match self {
            Self::Beginner => 9,
            Self::Intermediate => 16,
            Self::Expert => 30
        }
    }

    fn mines(self) -> u16
    {
        match self {
            Self::Beginner => 10,
            Self::Intermediate => 40,
            Self::Expert => 99
        }
    }
}

#[derive(Copy, Clone)]
struct Settings {
    rows_slider: iced::slider::State,
    cols_slider: iced::slider::State,
    mines_slider: iced::slider::State,
    width: u8,
    height: u8,
    mine_count: u16
}

impl Settings {
    fn new(width: u8, height: u8, mine_count: u16) -> Self
    {
        let mut new = Settings {
            rows_slider: iced::slider::State::new(),
            cols_slider: iced::slider::State::new(),
            mines_slider: iced::slider::State::new(),
            width: 0,
            height: 0,
            mine_count: 0,
        };
        new.update(width, height, mine_count);

        new
    }

    fn max_mines(self) -> u16
    {
        self.height as u16 * self.width as u16 - 1
    }

    fn update(&mut self, width: u8, height: u8, mine_count: u16)
    {
        self.width = width;
        self.height = height;

        self.mine_count = std::cmp::min(mine_count, self.max_mines());
    }

    fn view(&mut self) -> Element<Message>
    {
        let selected = {
            let mut selected = None;
            for level in <DifficultyLevels as strum::IntoEnumIterator>::iter() {
                if level.rows() == self.height
                    && level.cols() == self.width
                    && level.mines() == self.mine_count {
                        selected = Some(level);
                        break;
                }
            }
            selected
        };

        let preset = |level: DifficultyLevels| Message::DefineSettings{
            width: level.cols(),
            height: level.rows(),
            mine_count: level.mines(),
            apply: true
        };

        let presets = iced::Column::new()
            .push(iced::Radio::new(DifficultyLevels::Beginner,
                "Beginner", selected, preset))
            .push(iced::Radio::new(DifficultyLevels::Intermediate,
                "Itermediate", selected, preset))
            .push(iced::Radio::new(DifficultyLevels::Expert,
                "Expert", selected, preset));

        let labels = iced::Column::new()
                .push(Text::new("Rows:"))
                .push(Text::new("Columns:"))
                .push(Text::new("Mines:"));

        let width = self.width;
        let height = self.height;
        let mine_count = self.mine_count;

        let max_mines = self.max_mines();

        let sliders = iced::Column::new()
            .push(iced::Slider::new(&mut self.rows_slider, 2..=255,
                    self.height,
                    move |height| Message::DefineSettings{width, height, mine_count, apply: false})
                    .on_release(Message::ApplySettings))
            .push(iced::Slider::new(&mut self.cols_slider, 2..=255,
                    self.width as u8,
                    move |width| Message::DefineSettings{width, height, mine_count, apply: false})
                    .on_release(Message::ApplySettings))
            .push(iced::Slider::new(&mut self.mines_slider, 1..=max_mines,
                    self.mine_count,
                    move |mine_count| Message::DefineSettings{width, height, mine_count, apply: false})
                    .on_release(Message::ApplySettings))
            .width(iced::Length::Fill);

        let descriptions = iced::Column::new()
                .push(Text::new(format!("{} rows", height)))
                .push(Text::new(format!("{} columns", width)))
                .push(Text::new(format!("{} mines in {} cells, {:3.1} %",
                    mine_count, max_mines + 1,
                    (100 * mine_count) as f32 / (max_mines + 1) as f32)));

        iced::Row::new()
            .push(presets)
            .push(labels)
            .push(sliders)
            .push(descriptions)
            .spacing(10)
            .into()
    }
}

#[derive(Copy, Clone)]
struct RunningView
{
    restart_button: button::State,
    start_time: Instant
}

impl RunningView {
    fn new() -> Self
    {
        RunningView{
            start_time: Instant::now(),
            restart_button: button::State::new()
        }
    }

    fn view(&mut self, minefield: & minefield::Minefield) -> Element<Message>
    {
        let delta = Instant::now() - self.start_time;

        let info = iced::Column::new()
            .spacing(10)
            .width(iced::Length::Fill)
            .push(iced::Row::new()
                .push(Svg::new(FLAG.with(|f| f.clone())).height(iced::Length::Units(25)))
                .push(iced::Text::new(format!(": {}/{}", minefield.flag_count, minefield.mine_count))))
            .push(iced::Text::new(
                format!("Ellapsed time: {} seconds", delta.as_secs())));

        let button = iced::Button::new(&mut self.restart_button, iced::Text::new("Restart"))
            .on_press(Message::Restart);

        iced::Row::new()
            .push(info)
            .push(button)
            .into()
    }
}

#[derive(Copy, Clone)]
struct EndGameView {
    restart_button: button::State,
    game_duration: Duration,
    won: bool
}

impl EndGameView {
    fn new(start_time: Instant, won: bool) -> Self
    {
        let game_duration = Instant::now() - start_time;

        Self{
            restart_button: button::State::new(),
            game_duration, won
        }
    }

    fn view(&mut self, minefield: & minefield::Minefield) -> Element<Message>
    {
        let info = iced::Column::new()
            .spacing(10)
            .width(iced::Length::Fill)
            .push(iced::Row::new()
                .push(Svg::new(FLAG.with(|f| f.clone())).height(iced::Length::Units(25)))
                .push(iced::Text::new(format!(": {}/{}", minefield.flag_count, minefield.mine_count))))
            .push(iced::Text::new(
                format!("Game time: {:0.06} seconds", self.game_duration.as_secs_f64())))
            .push(iced::Text::new(
                if self.won {
                    "😄 You won! Congratulations!"
                } else {
                    "😖 You lost! Try again..."
                }
            ).size(40));

        let button = iced::Button::new(&mut self.restart_button, iced::Text::new("Restart"))
            .on_press(Message::Restart);

        iced::Row::new()
            .push(info)
            .push(button)
            .into()
    }
}

#[derive(Copy, Clone)]
enum GameState {
    BeforeStarted(Settings),
    Running(RunningView),
    Finished(EndGameView)
}

#[derive(Debug, Clone)]
enum Message {
    DefineSettings{width: u8, height: u8, mine_count: u16, apply: bool},
    ApplySettings,
    Restart,
    Tick,
    Reveal(u8, u8),
    Mark(u8, u8)
}

struct RevealedStyle;

impl RevealedStyle {
    const WHITE_BG: Option<iced::Background> = Some(iced::Background::Color(iced::Color::WHITE));
}

impl button::StyleSheet for RevealedStyle {
    fn active(&self) -> button::Style {
        button::Style {
            background: Self::WHITE_BG,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Self::WHITE_BG,
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            background: Self::WHITE_BG,
            ..self.hovered()
        }
    }
}

fn number_color(clue: u8) -> iced_native::Color
{
    use iced_native::Color;

    match clue {
        0 => Color::WHITE,
        1 => Color::from_rgb8(0x00, 0x00, 0xff),
        2 => Color::from_rgb8(0x00, 0x80, 0x00),
        3 => Color::from_rgb8(0xff, 0x00, 0x00),
        4 => Color::from_rgb8(0x00, 0x00, 0x80),
        5 => Color::from_rgb8(0x80, 0x00, 0x00),
        6 => Color::from_rgb8(0x00, 0x80, 0x80),
        7 => Color::BLACK,
        8 => Color::from_rgb8(0x80, 0x80, 0x80),
        _ => {
            panic!("Invalid clue value: {}", clue);
        }
    }
}

fn create_button<'a>(state: &'a mut button::State, tile: &minefield::Tile) -> Button<'a, Message>
{
    match tile {
        minefield::Tile::Hidden(_, minefield::UserMarking::None) =>
            Button::new(state, Space::new(iced::Length::Fill, iced::Length::Fill)),
        minefield::Tile::Hidden(_, minefield::UserMarking::Flag) =>
            Button::new(state, Svg::new(FLAG.with(|f| f.clone()))),
        minefield::Tile::Hidden(_, minefield::UserMarking::QuestionMark) =>
            Button::new(state, Text::new("?").horizontal_alignment(iced::HorizontalAlignment::Center)),
        minefield::Tile::Revealed(clue) => Button::new(state, Text::new(clue.to_string())
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .color(number_color(*clue))
        ).style(RevealedStyle),
    }
}

struct Minesweeper {
    minefield: Minefield,
    button_grid: Vec<Vec<button::State>>,
    state: GameState
}

impl Minesweeper {
    fn new(settings: Settings) -> Self
    {
        let size = usize::from(settings.width) * usize::from(settings.height);
        Self {
            minefield: Minefield::create_random(settings.width, settings.height, settings.mine_count),
            button_grid: vec![button::State::new(); size]
                .chunks(usize::from(settings.width)).map(|x| x.to_vec()).collect(),
            state: GameState::BeforeStarted(settings)
        }
    }
}

impl Application for Minesweeper {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        const DEFAULT: DifficultyLevels = DifficultyLevels::Expert;
        (
            Self::new(Settings::new(DEFAULT.cols(), DEFAULT.rows(), DEFAULT.mines())),
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Non-deterministic Minesweeper")
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::DefineSettings{width, height, mine_count, apply} => {
                if let GameState::BeforeStarted(settings) = &mut self.state {
                    settings.update(width, height, mine_count);
                    if apply {
                        *self = Minesweeper::new(*settings);
                    }
                } else {
                    panic!("We should only get settings message before started!");
                }
            },
            Message::ApplySettings => {
                if let GameState::BeforeStarted(settings) = &self.state {
                    *self = Minesweeper::new(*settings);
                } else {
                    panic!("We should only get settings message before started!");
                }
            },
            Message::Restart => {
                *self = Self::new(Settings::new(self.minefield.width, self.minefield.height, self.minefield.mine_count));
            }
            Message::Reveal(row, col) => {
                if let GameState::BeforeStarted(_) = self.state {
                    self.state = GameState::Running(RunningView::new());
                }

                if let GameState::Running(running) = self.state {
                    let has_lost = !self.minefield.reveal(row, col);
                    let has_won = !has_lost && self.minefield.is_all_revealed();

                    if has_lost || has_won {
                        self.state = GameState::Finished(EndGameView::new(running.start_time, has_won));
                    }
                }
            },
            Message::Mark(row, col) => {
                match self.state {
                    GameState::BeforeStarted(_) | GameState::Running(_) =>
                        self.minefield.switch_mark(row, col),
                    _ => {}
                }
            },
            _ => {}
        };
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::Tick)
    }

    fn view(&mut self) -> Element<Self::Message> {
        // Minefield
        let mut mf = Column::new();
        for (row, states, tiles) in izip!(0u16.., self.button_grid.iter_mut(), self.minefield.grid.iter()) {
            let mut view_row = Row::new();
            for (col, state, tile) in izip!(0u16.., states.iter_mut(), tiles.iter()) {
                view_row = view_row.push(RightClickable::new(create_button(state, tile)
                    .width(iced::Length::Units(30))
                    .height(iced::Length::Units(30))
                    .on_press(Message::Reveal(row as u8, col as u8)))
                    .on_right_click(Message::Mark(row as u8, col as u8)));
            }
            mf = mf.push(view_row);
        }

        // Controls
        let controls = iced::Container::new(match &mut self.state {
            GameState::BeforeStarted(controls) => controls.view(),
            GameState::Running(running) => running.view(&self.minefield),
            GameState::Finished(end_game) => end_game.view(&self.minefield),
        }).height(iced::Length::Units(150)).padding(20);

        // Aligner
        let aligner = iced::Container::new(mf)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Start);

        // Main container
        Column::new()
            .push(controls)
            .push(aligner)
            .into()
    }
}

fn main() -> iced::Result {
    Minesweeper::run(iced::Settings::default())
}
