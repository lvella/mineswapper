use std::collections::{VecDeque, HashMap, HashSet};
use std::iter::FromIterator;
use super::neighbor_iter::NeighborIterable;

type Key = (u8, u8);

#[derive(Copy, Clone)]
enum CellState {
    // TODO: ensure that two flavors of unconstrained is really necessary
    UnknownUnconstrained,
    UnknownConstrained,
    Mine,
    Empty,
    Clue(u8),
}

struct Solution {
    grid: Vec<Vec<CellState>>,
    width: u8,
    height: u8,
    unconstrained_count: u16
}

impl Solution {
    pub fn new(width: u8, height: u8) -> Self
    {
        Self{
            grid: vec![vec![CellState::UnknownUnconstrained; width as usize]; height as usize],
            unconstrained_count: width as u16 * height as u16,
            width, height
        }
    }

    pub fn add_clue(&mut self, (row, col): Key, mut clue: u8)
    {
        let state = &mut self.grid[row as usize][col as usize];

        // Sanity check:
        match state {
            CellState::Clue(_) => panic!("Can't add clue to a revealed square!"),
            CellState::Mine => panic!("Can't add clue to a hidden mine!"),
            _ => {}
        }

        // Mark neighbors as constrained and check for known mines:
        for (row, col) in self.neighbors_of(row, col) {
            let cell = &mut self.grid[row as usize][col as usize];
            match cell {
                CellState::UnknownUnconstrained => {
                    *cell = CellState::UnknownConstrained;
                    self.unconstrained_count -= 1;
                },
                CellState::Mine => {
                    clue -= 1;
                },
                _ => {}
            }
        }

        // Set the state of the new clue cell:
        self.grid[row as usize][col as usize] = CellState::Clue(clue);

        // Update the solution, starting from the newly revealed clue

        // TODO: to be continued...
        // below is all wrong. If clue == 0, must CheckIfClueFindEmpties,
        // If clue > 0, must CheckIfClueFindMines
        // Not sure about the neighbors.
        //if clue != 0 {
            // All the Clues > 0 neighbors, plus itself:
        //    let affected_clues: Vec<Key> = self.neighbors_of(row, col)
        //        .filter(|(row, col)| match self.get(*row, *col) {
        //            CellState::Clue(val) if *val > 0u8 => true,
        //            _ => false
        //        }).chain([(row, col)]).collect();

        //    self.breadth_first_update(affected_clues);
        //}
    }

    fn breadth_first_update(&mut self, seed: impl IntoIterator<Item=Key>)
    {
        enum Action {
            CheckIfClueFindMines,
            ToMine,
            CheckIfClueFindEmpties,
            ToEmpty
        }

        let mut is_visited: HashSet<Key> = HashSet::from_iter(seed);
        let mut queue: VecDeque<(Key, Action)> = is_visited.iter()
            .map(|key| (*key, Action::CheckIfClueFindMines)).collect();

        while let Some(((row, col), action)) = queue.pop_front() {
            let mut try_enqueue = |key, action| {
                if !is_visited.contains(&key) {
                    queue.push_front((key, action));
                }
            };

            match action {
                Action::CheckIfClueFindMines => {
                    let saturated = if let CellState::Clue(clue) = self.get(row, col) {
                        let mut unk_count = 0u8;
                        for (row, col) in self.neighbors_of(row, col) {
                            unk_count += match self.get(row, col) {
                                CellState::UnknownConstrained => 1,
                                CellState::UnknownUnconstrained =>
                                    panic!("Can't have unconstrained next to a clue!"),
                                _ => 0
                            }
                        }

                        assert!(unk_count <= *clue);
                        unk_count == *clue
                    } else {
                        panic!("Must be a clue!");
                    };

                    if saturated {
                        *self.get_mut(row, col) = CellState::Clue(0);

                        for (row, col) in self.neighbors_of(row, col) {
                            if let CellState::UnknownConstrained = self.get(row, col) {
                                try_enqueue((row, col), Action::ToMine);
                            }
                        }
                    }
                },

                Action::ToMine => {
                    assert!(matches!(self.get(row, col), CellState::UnknownConstrained));
                    *self.get_mut(row, col) = CellState::Mine;

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.get_mut(row, col) {
                            CellState::Clue(val) if *val > 0 => {
                                *val -= 1;
                                // TODO: to be continued...
                                // enqueue Clue(0)
                            },
                            _ => ()
                        }
                    }
                },

                // TODO: to be continued...
            }
        }
    }

    fn get_mut(&mut self, row: u8, col: u8) -> &mut CellState
    {
        &mut self.grid[usize::from(row)][usize::from(col)]
    }

    fn get(&self, row: u8, col: u8) -> &CellState
    {
        &self.grid[usize::from(row)][usize::from(col)]
    }
}

impl NeighborIterable for Solution {
    fn width(&self) -> u8
    {
        self.width
    }
    fn height(&self) -> u8
    {
        self.height
    }
}

#[derive(Default)]
struct BipartiteGraph
{
    clues: HashMap<Key, (u8, HashSet<Key>)>,
    unknowns: HashMap<Key, HashSet<Key>>
}
