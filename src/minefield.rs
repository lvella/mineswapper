use std::time::Instant;
use rand::seq;
use super::neighbor_iter::NeighborIterable;
use super::solver::PartialSolution;
use super::grid;

#[derive(Copy, Clone)]
pub enum UserMarking
{
    None,
    Flag,
    QuestionMark,
}

#[derive(Copy, Clone)]
pub enum Content
{
    Empty,
    Mine
}

#[derive(Copy, Clone)]
pub enum Tile {
    Hidden(Content, UserMarking),
    Revealed(u8)
}

#[derive(Default)]
pub struct MinefieldCounters {
    pub flag_count: u16,
    pub revealed_count: u16,
}

impl grid::GridCounters<Tile> for MinefieldCounters {
    fn notify_change(&mut self, from: &Tile, to: &Tile)
    {
        if let Tile::Hidden(_, UserMarking::Flag) = *from {
            self.flag_count -= 1;
        }

        match *to {
            Tile::Hidden(_, UserMarking::Flag) => {
                self.flag_count += 1;
            },
            Tile::Revealed(_) => {
                assert!(!matches!(*from, Tile::Revealed(_)));
                self.revealed_count += 1;
            },
            _ => ()
        }
    }
}

pub struct Minefield {
    pub grid: grid::Grid<Tile, u8, MinefieldCounters>,
    pub mine_count: u16,
    sol: PartialSolution,
}

impl Minefield {
    pub fn create_random(width: u8, height: u8, mine_count: u16, rng: &mut impl rand::Rng) -> Minefield {

        let swidth = usize::from(width);
        let total_size = swidth * usize::from(height);
        let smine_count = usize::from(mine_count);

        if smine_count > total_size {
            panic!("More mines than it fits in the field!");
        }

        let mut flattened = vec![
            Tile::Hidden(Content::Mine, UserMarking::None); smine_count
        ];
        flattened.resize(total_size, Tile::Hidden(Content::Empty, UserMarking::None));

        seq::SliceRandom::shuffle(&mut flattened[..], rng);

        let sol = PartialSolution::new(width, height, mine_count);
        //sol.print();

        Minefield {
            grid: grid::Grid::from_vec(width, height, flattened).unwrap(), mine_count, sol
        }
    }

    pub fn reveal(&mut self, rng: &mut impl rand::Rng, row: u8, col: u8) -> bool
    {
        let cells = self.find_revealed_cells(row, col, true);
        let was_something_revealed = cells.len() > 0;

        let survived = {
            let had_mine = cells.iter().any(|&(_,_,mine)| mine);

            !had_mine || self.try_reacomodate(rng, cells.iter()
                .map(|&(row, col, _)| (row, col)))
        };

        // Independently of surviving, reveal what is revealable:
        for (row, col, _) in cells {
            self.recursive_reveal(row, col);
        }

        // Update the solver only if something changed:
        if survived && was_something_revealed {
            let begin = std::time::Instant::now();
            self.sol.find_graph_solutions();
            let delta = std::time::Instant::now() - begin;
            println!("Search time: {:0.09}", delta.as_secs_f64());

            //self.sol.print();
        }

        survived
    }

    pub fn switch_mark(&mut self, row: u8, col: u8)
    {
        if let Tile::Hidden(c, mark) = *self.grid.get(row, col) {
            self.grid.set(row, col, Tile::Hidden(c, match mark {
                UserMarking::None => {
                    UserMarking::Flag
                },
                UserMarking::Flag => {
                    UserMarking::QuestionMark
                },
                UserMarking::QuestionMark => UserMarking::None
            }));
        }
    }

    pub fn is_all_revealed(&self) -> bool
    {
        self.grid.counters.revealed_count + self.mine_count == self.width() as u16 * self.height() as u16
    }

    fn find_revealed_cells(&self, row: u8, col: u8, process_revealed: bool)
        -> Vec<(u8, u8, bool)>
    {
        match self.grid.get(row, col) {
            Tile::Hidden(_, UserMarking::Flag) => Vec::new(),
            Tile::Hidden(Content::Mine, _) => vec![(row, col, true)],
            Tile::Hidden(Content::Empty, _) => vec![(row, col, false)],
            Tile::Revealed(count) => {
                // Only reveal neighbors if there is the exact number
                // of flags around the clue
                if process_revealed && *count == self.neighbors_of(row, col).fold(0,
                    |sum, (row, col)| sum + match self.grid.get(row, col) {
                        Tile::Hidden(_, UserMarking::Flag) => 1,
                        _ => 0
                    }
                ) {
                    // Reveal unflagged neighbos
                    self.neighbors_of(row, col).fold(Vec::new(),
                        |mut acum, (row, col)| {
                            acum.append(&mut self.find_revealed_cells(row, col, false));
                            acum
                        })
                } else {
                    Vec::new()
                }
            }
        }
    }

    fn try_reacomodate(&mut self, rng: &mut impl rand::Rng, revealed: impl IntoIterator<Item = (u8, u8)>)
        -> bool
    {
        let begin = Instant::now();

        let grid = &mut self.grid;

        let ret = self.sol.find_acomodating_solution(rng, revealed, |row, col, is_mine| {
            match *grid.get(row, col) {
                Tile::Hidden(_, m) => {
                    grid.set(row, col, Tile::Hidden(if is_mine {
                            Content::Mine
                        } else {
                            Content::Empty
                        },
                    m));
                },
                _ => panic!("Can not reaccommodate revealed tiles")
            };
        });

        let elapsed = Instant::now() - begin;
        println!("Reconfiguration time: {:0.06}", elapsed.as_secs_f64());

        ret
    }

    fn recursive_reveal(&mut self, row: u8, col: u8)
    {
        match *self.grid.get(row, col) {
            Tile::Hidden(Content::Empty, _) => {
                let bomb_count = self.count_neighbor_bombs(row, col);

                self.grid.set(row, col, Tile::Revealed(bomb_count));
                self.sol.add_clue((row, col), bomb_count);

                if bomb_count == 0 {
                    for (row, col) in self.neighbors_of(row, col) {
                        self.recursive_reveal(row, col);
                    }
                };
            },
            // TODO: reveal mine to display to the player the reason of losing
            _ => ()
        }
    }

    fn count_neighbor_bombs(&self, row: u8, col: u8) -> u8
    {
        self.neighbors_of(row, col).fold(0, |accum, (row, col)| {
            accum + match self.grid.get(row, col) {
                Tile::Hidden(Content::Mine, _) => 1,
                _ => 0
            }
        })
    }
}

impl NeighborIterable for Minefield {
    fn width(&self) -> u8
    {
        self.grid.width()
    }

    fn height(&self) -> u8
    {
        self.grid.height()
    }
}
