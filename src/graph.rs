use std::collections::{VecDeque, HashMap, HashSet};
use std::iter::FromIterator;
use arrayvec::ArrayVec;
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

#[derive(Copy, Clone)]
enum UpdateAction {
    CheckIfClueFindMines,
    ToMine,
    CheckIfClueFindEmpties,
    ToEmpty
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

        // Check if we previously knew if the cell was empty,
        // and possibly update the state before anything else.
        match state {
            CellState::Clue(_) => panic!("Can't add clue to a revealed square!"),
            CellState::Mine => panic!("Can't add clue to a hidden mine!"),
            CellState::UnknownConstrained => {
                // We didn't knew this was empty, so we
                // must update the neighboring cells.
                self.breadth_first_update(UpdateAction::ToEmpty, &[(row, col)]);
            },
            _ => {}
        }

        let mut unknowns = ArrayVec::<Key, 8>::new();

        // Mark neighbors as constrained and check for known mines:
        for (row, col) in self.neighbors_of(row, col) {
            let cell = &mut self.grid[row as usize][col as usize];
            match *cell {
                CellState::UnknownUnconstrained => {
                    *cell = CellState::UnknownConstrained;
                    self.unconstrained_count -= 1;
                    unknowns.push((row, col));
                },
                CellState::UnknownConstrained => {
                    unknowns.push((row, col));
                },
                CellState::Mine => {
                    clue -= 1;
                },
                _ => {}
            }
        }

        // Set the state of the new clue cell:
        self.grid[row as usize][col as usize] = CellState::Clue(clue);

        // Update the solution, changing unknown neighbors to either Empty or Mines, as appropriate.
        if unknowns.len() > 0 {
            let slice = unknowns.as_slice();
            if clue == 0 {
                self.breadth_first_update(UpdateAction::ToEmpty, slice);
            } else if clue == unknowns.len() as u8 {
                self.breadth_first_update(UpdateAction::ToMine, slice);
            }
        }
    }


    fn breadth_first_update(&mut self, action: UpdateAction, seed: &[Key])
    {
        let mut is_queued: HashSet<Key> = HashSet::from_iter(seed.iter().copied());
        let mut queue: VecDeque<(Key, UpdateAction)> = is_queued.iter()
            .map(|key| (*key, action)).collect();

        while let Some(((row, col), action)) = queue.pop_front() {
            is_queued.remove(&(row, col));

            let mut try_enqueue = |key, action| {
                if is_queued.insert(key) {
                    queue.push_front((key, action));
                }
            };

            match action {
                UpdateAction::CheckIfClueFindMines => {
                    let clue = if let CellState::Clue(clue) = self.get(row, col) {
                        *clue
                    } else {
                        panic!("Must be a clue!");
                    };

                    let mut unknowns = ArrayVec::<Key, 8>::new();

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.get(row, col) {
                            CellState::UnknownConstrained => unknowns.push((row, col)),
                            CellState::UnknownUnconstrained =>
                                panic!("Can't have unconstrained next to a clue!"),
                            _ => ()
                        }
                    }

                    assert!(unknowns.len() as u8 >= clue);

                    if unknowns.len() as u8 == clue {
                        *self.get_mut(row, col) = CellState::Clue(0);

                        for (row, col) in unknowns {
                            try_enqueue((row, col), UpdateAction::ToMine);
                        }
                    }
                },

                UpdateAction::ToMine => {
                    assert!(matches!(self.get(row, col), CellState::UnknownConstrained));
                    *self.get_mut(row, col) = CellState::Mine;

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.get_mut(row, col) {
                            CellState::Clue(val) if *val > 0 => {
                                *val -= 1;
                                if *val == 0 {
                                    // A clue can only get to zero once,
                                    // so it can not be inserted twice:
                                    assert!(is_queued.insert((row, col)));
                                    queue.push_back(((row, col), UpdateAction::CheckIfClueFindEmpties));
                                }
                            },
                            _ => ()
                        }
                    }
                },

                UpdateAction::CheckIfClueFindEmpties => {
                    // You can only get empties from a 0 clue:
                    assert!(matches!(self.get(row, col), CellState::Clue(0)));

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.get(row, col) {
                            CellState::UnknownConstrained =>
                                try_enqueue((row, col), UpdateAction::ToEmpty),
                            CellState::UnknownUnconstrained =>
                                panic!("Can't have unconstrained next to a clue!"),
                            _ => ()
                        }
                    }
                },

                UpdateAction::ToEmpty => {
                    // Only constrained can be found to be empty:
                    assert!(matches!(self.get(row, col), CellState::UnknownConstrained));
                    *self.get_mut(row, col) = CellState::Empty;

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.get(row, col) {
                            CellState::Clue(val) if *val > 0 => {

                            },
                            _ => ()
                        }
                    }
                }
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
