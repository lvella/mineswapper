mod grid;
mod minefield;
mod neighbor_iter;
mod right_clickable;
mod search;
mod solver;

use iced::{
    executor,
    widget::{self, svg},
    Application,
};
use iced_native::Theme;
use minefield::Minefield;
use right_clickable::RightClickable;
use std::time::{Duration, Instant};
use strum_macros;

thread_local!(
    static FLAG: svg::Handle =
        svg::Handle::from_memory(&include_bytes!("../resources/flag.svg.gz")[..]);

    static CROSSED_FLAG: svg::Handle =
        svg::Handle::from_memory(&include_bytes!("../resources/crossed_flag.svg.gz")[..]);
);

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum_macros::EnumIter)]
enum DifficultyLevels {
    Beginner,
    Intermediate,
    Expert,
}

impl DifficultyLevels {
    fn rows(self) -> u8 {
        match self {
            Self::Beginner => 9,
            Self::Intermediate => 16,
            Self::Expert => 16,
        }
    }

    fn cols(self) -> u8 {
        match self {
            Self::Beginner => 9,
            Self::Intermediate => 16,
            Self::Expert => 30,
        }
    }

    fn mines(self) -> u16 {
        match self {
            Self::Beginner => 10,
            Self::Intermediate => 40,
            Self::Expert => 99,
        }
    }
}

#[derive(Copy, Clone)]
struct Settings {
    width: u8,
    height: u8,
    mine_count: u16,
}

impl Settings {
    fn new(width: u8, height: u8, mine_count: u16) -> Self {
        let mut new = Settings {
            width: 0,
            height: 0,
            mine_count: 0,
        };
        new.update(width, height, mine_count);

        new
    }

    fn max_mines(self) -> u16 {
        self.height as u16 * self.width as u16 - 1
    }

    fn update(&mut self, width: u8, height: u8, mine_count: u16) {
        self.width = width;
        self.height = height;

        self.mine_count = std::cmp::min(mine_count, self.max_mines());
    }

    fn view(&self) -> iced::Element<Message> {
        let selected = {
            let mut selected = None;
            for level in <DifficultyLevels as strum::IntoEnumIterator>::iter() {
                if level.rows() == self.height
                    && level.cols() == self.width
                    && level.mines() == self.mine_count
                {
                    selected = Some(level);
                    break;
                }
            }
            selected
        };

        let preset = |level: DifficultyLevels| Message::DefineSettings {
            width: level.cols(),
            height: level.rows(),
            mine_count: level.mines(),
            apply: true,
        };

        let presets = widget::Column::new()
            .push(widget::Radio::new(
                "Beginner",
                DifficultyLevels::Beginner,
                selected,
                preset,
            ))
            .push(widget::Radio::new(
                "Itermediate",
                DifficultyLevels::Intermediate,
                selected,
                preset,
            ))
            .push(widget::Radio::new(
                "Expert",
                DifficultyLevels::Expert,
                selected,
                preset,
            ));

        let labels = widget::Column::new()
            .push(widget::Text::new("Rows:"))
            .push(widget::Text::new("Columns:"))
            .push(widget::Text::new("Mines:"));

        let width = self.width;
        let height = self.height;
        let mine_count = self.mine_count;

        let max_mines = self.max_mines();

        let sliders = widget::Column::new()
            .push(
                widget::Slider::new(2..=255, self.height, move |height| {
                    Message::DefineSettings {
                        width,
                        height,
                        mine_count,
                        apply: false,
                    }
                })
                .on_release(Message::ApplySettings),
            )
            .push(
                widget::Slider::new(2..=255, self.width as u8, move |width| {
                    Message::DefineSettings {
                        width,
                        height,
                        mine_count,
                        apply: false,
                    }
                })
                .on_release(Message::ApplySettings),
            )
            .push(
                widget::Slider::new(1..=max_mines, self.mine_count, move |mine_count| {
                    Message::DefineSettings {
                        width,
                        height,
                        mine_count,
                        apply: false,
                    }
                })
                .on_release(Message::ApplySettings),
            )
            .width(iced::Length::Fill);

        let descriptions = widget::Column::new()
            .push(widget::Text::new(format!("{} rows", height)))
            .push(widget::Text::new(format!("{} columns", width)))
            .push(widget::Text::new(format!(
                "{} mines in {} cells, {:3.1} %",
                mine_count,
                max_mines + 1,
                (100 * mine_count) as f32 / (max_mines + 1) as f32
            )));

        widget::Row::new()
            .push(presets)
            .push(labels)
            .push(sliders)
            .push(descriptions)
            .spacing(10)
            .into()
    }
}

#[derive(Copy, Clone)]
struct RunningView {
    start_time: Instant,
}

fn status_display<'a>(
    minefield: &minefield::Minefield,
    display_elements: impl Iterator<Item = iced::Element<'a, Message>>,
) -> iced::Element<'a, Message> {
    let mut info = widget::Column::new()
        .spacing(10)
        .width(iced::Length::Fill)
        .push(
            widget::Row::new()
                .width(iced::Length::Shrink)
                .align_items(iced_native::Alignment::Start)
                .push(widget::Svg::new(FLAG.with(|f| f.clone())).width(iced::Length::Fixed(25.0)))
                .push(widget::Text::new(format!(
                    ": {}/{}",
                    minefield.grid.counters.flag_count, minefield.mine_count
                ))),
        );

    for e in display_elements {
        info = info.push(e)
    }

    let button = widget::Button::new(widget::Text::new("Restart")).on_press(Message::Restart);

    widget::Row::new().push(info).push(button).into()
}

impl RunningView {
    fn new() -> Self {
        RunningView {
            start_time: Instant::now(),
        }
    }

    fn view(&self, minefield: &minefield::Minefield) -> iced::Element<Message> {
        let delta = Instant::now() - self.start_time;

        status_display(
            minefield,
            [widget::Text::new(format!("Ellapsed time: {} seconds", delta.as_secs())).into()]
                .into_iter(),
        )
    }
}

#[derive(Copy, Clone)]
struct EndGameView {
    game_duration: Duration,
    won: bool,
}

impl EndGameView {
    fn new(start_time: Instant, won: bool) -> Self {
        let game_duration = Instant::now() - start_time;

        Self { game_duration, won }
    }

    fn view(&self, minefield: &minefield::Minefield) -> iced::Element<Message> {
        status_display(
            minefield,
            [
                widget::Text::new(format!(
                    "Game time: {:0.06} seconds",
                    self.game_duration.as_secs_f64()
                ))
                .into(),
                widget::Text::new(if self.won {
                    "😄 You won! Congratulations!"
                } else {
                    "😖 You lost! Try again..."
                })
                .size(40)
                .into(),
            ]
            .into_iter(),
        )
    }
}

#[derive(Copy, Clone)]
enum GameState {
    BeforeStarted(Settings),
    Running(RunningView),
    Finished(EndGameView),
}

#[derive(Debug, Clone)]
enum Message {
    DefineSettings {
        width: u8,
        height: u8,
        mine_count: u16,
        apply: bool,
    },
    ApplySettings,
    Restart,
    Tick,
    Reveal(u8, u8),
    Mark(u8, u8),
}

struct RevealedStyle;

impl RevealedStyle {
    const WHITE_BG: Option<iced::Background> = Some(iced::Background::Color(iced::Color::WHITE));
}

impl widget::button::StyleSheet for RevealedStyle {
    fn active(&self, _: &Theme) -> widget::button::Appearance {
        widget::button::Appearance {
            background: Self::WHITE_BG,
            ..widget::button::Appearance::default()
        }
    }

    fn hovered(&self, _: &Theme) -> widget::button::Appearance {
        widget::button::Appearance {
            background: Self::WHITE_BG,
            ..widget::button::Appearance::default()
        }
    }

    fn pressed(&self, _: &Theme) -> widget::button::Appearance {
        widget::button::Appearance {
            background: Self::WHITE_BG,
            ..widget::button::Appearance::default()
        }
    }

    type Style = Theme;
}

fn number_color(clue: u8) -> iced_native::Color {
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

fn create_button(tile: &minefield::Tile, exposed: bool) -> widget::Button<Message> {
    if exposed {
        match tile {
            minefield::Tile::Hidden(minefield::Content::Mine, minefield::UserMarking::None) => {
                return widget::Button::new(
                    widget::Text::new("O")
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                );
            }
            minefield::Tile::Hidden(minefield::Content::Empty, minefield::UserMarking::Flag) => {
                return widget::Button::new(widget::Svg::new(CROSSED_FLAG.with(|f| f.clone())));
            }
            _ => (),
        }
    }

    match tile {
        minefield::Tile::Hidden(_, minefield::UserMarking::None) => {
            widget::Button::new(widget::Space::new(iced::Length::Fill, iced::Length::Fill))
        }
        minefield::Tile::Hidden(_, minefield::UserMarking::Flag) => {
            widget::Button::new(widget::Svg::new(FLAG.with(|f| f.clone())))
        }
        minefield::Tile::Hidden(_, minefield::UserMarking::QuestionMark) => widget::Button::new(
            widget::Text::new("?").horizontal_alignment(iced::alignment::Horizontal::Center),
        ),
        minefield::Tile::Revealed(clue) => widget::Button::new(
            widget::Text::new(clue.to_string())
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .style(number_color(*clue)),
        )
        .style(<Theme as widget::button::StyleSheet>::Style::Custom(
            Box::new(RevealedStyle),
        )),
    }
}

struct Minesweeper {
    minefield: Minefield,
    rng: rand_xoshiro::Xoshiro256StarStar,
    state: GameState,
}

impl Minesweeper {
    fn new(settings: Settings) -> Self {
        use hex::FromHex;
        use rand_core::SeedableRng;
        use std::env;

        let rng_seed = if let Some(Ok(seed)) = env::args_os().nth(1).and_then(|arg| {
            arg.to_str()
                .map(|valid_str| <[u8; 32]>::from_hex(valid_str))
        }) {
            println!("Using provided seed.");

            seed
        } else {
            let mut seed: [u8; 32] = Default::default();
            getrandom::getrandom(&mut seed).unwrap();
            println!("Using random seed: {}", hex::encode(seed));

            seed
        };

        let mut rng = rand_xoshiro::Xoshiro256StarStar::from_seed(rng_seed);

        Self {
            minefield: Minefield::create_random(
                settings.width,
                settings.height,
                settings.mine_count,
                &mut rng,
            ),
            rng,
            state: GameState::BeforeStarted(settings),
        }
    }
}

impl Application for Minesweeper {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        const DEFAULT: DifficultyLevels = DifficultyLevels::Expert;
        (
            Self::new(Settings::new(
                DEFAULT.cols(),
                DEFAULT.rows(),
                DEFAULT.mines(),
            )),
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Non-deterministic Minesweeper")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::DefineSettings {
                width,
                height,
                mine_count,
                apply,
            } => {
                if let GameState::BeforeStarted(settings) = &mut self.state {
                    settings.update(width, height, mine_count);
                    if apply {
                        *self = Minesweeper::new(*settings);
                    }
                } else {
                    panic!("We should only get settings message before started!");
                }
            }
            Message::ApplySettings => {
                if let GameState::BeforeStarted(settings) = &self.state {
                    *self = Minesweeper::new(*settings);
                } else {
                    panic!("We should only get settings message before started!");
                }
            }
            Message::Restart => {
                *self = Self::new(Settings::new(
                    self.minefield.grid.width(),
                    self.minefield.grid.height(),
                    self.minefield.mine_count,
                ));
            }
            Message::Reveal(row, col) => {
                if let GameState::BeforeStarted(_) = self.state {
                    self.state = GameState::Running(RunningView::new());
                }

                if let GameState::Running(running) = self.state {
                    let has_lost = !self.minefield.reveal(&mut self.rng, row, col);
                    let has_won = !has_lost && self.minefield.is_all_revealed();

                    if has_lost || has_won {
                        self.state =
                            GameState::Finished(EndGameView::new(running.start_time, has_won));
                    }
                }
            }
            Message::Mark(row, col) => match self.state {
                GameState::BeforeStarted(_) | GameState::Running(_) => {
                    self.minefield.switch_mark(row, col)
                }
                _ => {}
            },
            _ => {}
        };
        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick)
    }

    fn view(&self) -> iced::Element<Self::Message> {
        // Minefield
        let mut mf = widget::Column::new().spacing(1);
        for (row, tiles) in (0u16..).zip(self.minefield.grid.rows()) {
            let mut view_row = widget::Row::new().spacing(1);
            for (col, tile) in (0u16..).zip(tiles.iter()) {
                view_row = view_row.push(
                    RightClickable::new(
                        create_button(tile, matches!(self.state, GameState::Finished(_)))
                            .width(iced::Length::Fixed(29.0))
                            .height(iced::Length::Fixed(29.0))
                            .on_press(Message::Reveal(row as u8, col as u8)),
                    )
                    .on_right_click(Message::Mark(row as u8, col as u8)),
                );
            }
            mf = mf.push(view_row);
        }

        // Controls
        let controls = widget::Container::new(match &self.state {
            GameState::BeforeStarted(controls) => controls.view(),
            GameState::Running(running) => running.view(&self.minefield),
            GameState::Finished(end_game) => end_game.view(&self.minefield),
        })
        .height(iced::Length::Fixed(150.0))
        .padding(20);

        // Aligner
        let aligner = widget::Container::new(mf)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Top);

        // Main container
        widget::Column::new().push(controls).push(aligner).into()
    }

    type Theme = iced::Theme;
}

fn main() -> iced::Result {
    let settings = iced::Settings {
        ..Default::default()
    };
    Minesweeper::run(settings)
}
