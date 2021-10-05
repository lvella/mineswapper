use rand::{thread_rng, seq};
use super::neighbor_iter::NeighborIterable;

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

pub struct Minefield {
    pub grid: Vec<Vec<Tile>>,
    pub width: u8,
    pub height: u8,
    pub mine_count: u16,
    pub flag_count: u16,
    pub revealed_count: u16
}

impl Minefield {
    pub fn create_random(width: u8, height: u8, mine_count: u16) -> Minefield {
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

        seq::SliceRandom::shuffle(&mut flattened[..], &mut thread_rng());

        Minefield {
            grid: flattened.chunks(swidth).map(|x| x.to_vec()).collect(),
            width, height, mine_count, flag_count: 0, revealed_count: 0
        }
    }

    pub fn reveal(&mut self, row: u8, col: u8) -> bool
    {
        match self.get(row, col) {
            Tile::Hidden(_, UserMarking::Flag) => true,
            Tile::Hidden(Content::Mine, _) => false,
            Tile::Hidden(Content::Empty, _) => {
                self.recursive_reveal(row, col);
                true
            }

            Tile::Revealed(count) => {
                // Only reveal neighbors if there is the exact number
                // of flags around the clue
                if *count == self.neighbors_of(row, col).fold(0,
                    |sum, (row, col)| sum + match self.get(row, col) {
                        Tile::Hidden(_, UserMarking::Flag) => 1,
                        _ => 0
                    }
                ) {
                    // Reveal unflagged neighbor clues
                    self.neighbors_of(row, col).fold(true,
                        |survived, (row, col)| match self.get(row, col) {
                            Tile::Hidden(_, _) => self.reveal(row, col),
                            Tile::Revealed(_) => true
                    } && survived)
                } else {
                    true
                }
            }
        }
    }

    pub fn switch_mark(&mut self, row: u8, col: u8)
    {
        if let Tile::Hidden(_, mark) = self.get_mut(row, col) {
            let mut flag_change = 0i32;
            *mark = match mark {
                UserMarking::None => {
                    flag_change = 1;
                    UserMarking::Flag
                },
                UserMarking::Flag => {
                    flag_change = -1;
                    UserMarking::QuestionMark
                },
                UserMarking::QuestionMark => UserMarking::None
            };

            self.flag_count = (self.flag_count as i32 + flag_change) as u16;
        }
    }

    pub fn is_all_revealed(&self) -> bool
    {
        self.revealed_count + self.mine_count == self.width as u16 * self.height as u16
    }

    fn get_mut(&mut self, row: u8, col: u8) -> &mut Tile
    {
        &mut self.grid[usize::from(row)][usize::from(col)]
    }

    fn get(&self, row: u8, col: u8) -> &Tile
    {
        &self.grid[usize::from(row)][usize::from(col)]
    }

    fn recursive_reveal(&mut self, row: u8, col: u8)
    {
        match *self.get(row, col) {
            Tile::Hidden(Content::Empty, _) => {
                let bomb_count = self.count_neighbor_bombs(row, col);
                (*self.get_mut(row, col)) = Tile::Revealed(bomb_count);
                self.revealed_count += 1;
                if bomb_count == 0 {
                    for (row, col) in self.neighbors_of(row, col) {
                        self.recursive_reveal(row, col);
                    }
                };
            },
            Tile::Hidden(Content::Mine, _) => panic!("A mine should never be revealed!"),
            _ => ()
        }
    }

    fn count_neighbor_bombs(&self, row: u8, col: u8) -> u8
    {
        self.neighbors_of(row, col).fold(0, |accum, (row, col)| {
            accum + match self.get(row, col) {
                Tile::Hidden(Content::Mine, _) => 1,
                _ => 0
            }
        })
    }
}

impl NeighborIterable for Minefield {
    fn width(&self) -> u8
    {
        self.width
    }

    fn height(&self) -> u8
    {
        self.height
    }
}
