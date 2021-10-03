use rand::{thread_rng, seq};
use crate::search;

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
    pub mine_count: u16
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
            width, height, mine_count
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

            // The following is too harsh, because it will try to reveal all the hidden
            // squares around a revealed clue. For others minesweeper implementations,
            // it would only work if there is the exact number of flags around a
            // revealed clue, sometimes saving the player.
            // TODO: maybe implement the forgiving behavior of other implementations
            Tile::Revealed(_) => self.neighbor_coords(row, col).iter().fold(true,
                |value, (row, col)| match self.get(*row, *col) {
                    Tile::Hidden(_, _) => self.reveal(*row, *col),
                    Tile::Revealed(_) => true
                } && value
            )
        }
    }

    pub fn switch_mark(&mut self, row: u8, col: u8)
    {
        if let Tile::Hidden(_, mark) = self.get_mut(row, col) {
            *mark = match mark {
                UserMarking::None => UserMarking::Flag,
                UserMarking::Flag => UserMarking::QuestionMark,
                UserMarking::QuestionMark => UserMarking::None
            }
        }
    }

    fn get_mut(&mut self, row: u8, col: u8) -> &mut Tile
    {
        &mut self.grid[usize::from(row)][usize::from(col)]
    }

    fn get(&self, row: u8, col: u8) -> &Tile
    {
        &self.grid[usize::from(row)][usize::from(col)]
    }

    fn neighbor_coords(&self, row: u8, col: u8) -> Vec<(u8, u8)>
    {
        const DELTAS: [(i16, i16); 8] = [
            (-1, -1), (-1, 0), (-1, 1),
            ( 0, -1),          ( 0, 1),
            ( 1, -1), ( 1, 0), ( 1, 1)
        ];
        DELTAS.iter().filter_map(|(dr, dc)| {
            let row = dr + i16::from(row);
            if row < 0 || row >= i16::from(self.height) {
                return None;
            }

            let col = dc + i16::from(col);
            if col < 0 || col >= i16::from(self.width) {
                return None;
            }

            Some((row as u8, col as u8))
        }).collect()
    }

    fn recursive_reveal(&mut self, row: u8, col: u8)
    {
        match *self.get(row, col) {
            Tile::Hidden(Content::Empty, _) => {
                let bomb_count = self.count_neighbor_bombs(row, col);
                (*self.get_mut(row, col)) = Tile::Revealed(bomb_count);
                if bomb_count == 0 {
                    for (row, col) in self.neighbor_coords(row, col) {
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
        self.neighbor_coords(row, col).iter().fold(0, |accum, (row, col)| {
            accum + match self.get(*row, *col) {
                Tile::Hidden(Content::Mine, _) => 1,
                _ => 0
            }
        })
    }
}
