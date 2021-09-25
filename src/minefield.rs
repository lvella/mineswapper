use rand::{thread_rng, seq};

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
}

impl Minefield {
    pub fn create_random(width: u8, height: u8, mine_count: u16) -> Minefield {
        let width = usize::from(width);
        let total_size = usize::from(width) * usize::from(height);
        let mine_count = usize::from(mine_count);

        if mine_count > total_size {
            panic!("More mines than it fits in the field!");
        }

        let mut flattened = vec![
            Tile::Hidden(Content::Mine, UserMarking::None); mine_count
        ];
        flattened.resize(total_size, Tile::Hidden(Content::Empty, UserMarking::None));

        seq::SliceRandom::shuffle(&mut flattened[..], &mut thread_rng());

        Minefield {
            grid: flattened.chunks(width).map(|x| x.to_vec()).collect(),
        }
    }
}

