use rand::{thread_rng, seq};
use std::{io, fmt};
use std::io::Write;

#[derive(Copy, Clone)]
enum Tile {
    Mine,
    Empty,
    Revealed
}


impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str( match self {
            Tile::Mine => "*",
            Tile::Empty => ".",
            Tile::Revealed => " ",
        })
    }
}

struct Minefield {
    grid: Vec<Vec<Tile>>,
}

impl Minefield {
    fn create_random(width: u16, height: u16, mine_count: u16) -> Minefield {
        let width = usize::from(width);
        let total_size = usize::from(width) * usize::from(height);
        let mine_count = usize::from(mine_count);

        if mine_count > total_size {
            panic!("More mines than it fits in the field!");
        }

        let mut flattened = vec![Tile::Mine; mine_count];
        flattened.resize(total_size, Tile::Empty);

        seq::SliceRandom::shuffle(&mut flattened[..], &mut thread_rng());

        Minefield {
            grid: flattened[..].chunks(width).map(|x| x.to_vec()).collect(),
        }
    }
}


fn main() {
    let minefield = Minefield::create_random(30, 16, 99);
    for row in minefield.grid {
        for tile in row {
            print!("{}", tile);
        }
        print!("\n");
    }
    io::stdout().flush().unwrap();
}
