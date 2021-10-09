use std::collections::{VecDeque, HashSet, HashMap};
use std::iter::FromIterator;
use arrayvec::ArrayVec;
use bitvec::prelude as bv;
use super::neighbor_iter::NeighborIterable;
use super::search;

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

pub struct PartialSolution {
    grid: Vec<Vec<CellState>>,
    width: u8,
    height: u8,
    unconstrained_count: u16
}

#[derive(Copy, Clone, Debug)]
enum UpdateAction {
    CheckIfClueFindMines,
    ToMine,
    CheckIfClueFindEmpties,
    ToEmpty
}

impl PartialSolution {
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
                    queue.push_back((key, action));
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
                                    for e in &queue {
                                        println!("### {:?}", e);
                                    }
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
                                try_enqueue((row, col), UpdateAction::CheckIfClueFindMines)
                            },
                            _ => ()
                        }
                    }
                }
            }
        }
    }

    pub fn find_solutions(&self)
    {
        let mut visited = vec![bv::bitvec![0; self.width as usize]; self.height as usize];

        for (i, row) in self.grid.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let visited = visited[i].get_mut(j).unwrap();
                if *visited {
                    continue;
                }
                visited.set(true);

                match cell {
                    CellState::Clue(val) if *val > 0 => {
                        // TODO: to be continued...
                    },
                    _ => ()
                }
            }
        }
    }

    fn extract_graph_starting_from(&self, row: u8, col: u8, visited: &mut [bv::BitVec]) ->
        (HashMap::<Key, u16>, search::Topology)
    {
        // Start search
        let mut queue = VecDeque::new();
        queue.push_back((row, col));

        // Maps of keys to the local 0-based indices of unknowns:
        let mut unk_map = HashMap::<Key, u16>::new();

        // List of clues part of this graph:
        let mut clues = Vec::<search::Clue>::new();

        // Breadth-first search
        while let Some((row, col)) = queue.pop_front() {
            let mut try_enqueue = |row, col| {
                let visited = visited[row as usize].get_mut(col as usize).unwrap();
                if *visited {
                    visited.set(true);
                    queue.push_back((row, col));
                }
            };

            match self.get(row, col) {
                CellState::Clue(val) => {
                    assert!(*val > 0u8);

                    let mut adjacency = Vec::new();
                    for (row, col) in self.neighbors_of(row, col) {
                        if let CellState::UnknownConstrained = self.get(row, col) {
                            let len = unk_map.len() as u16;
                            let unk_id = *unk_map.entry((row, col)).or_insert(len);
                            adjacency.push(unk_id);

                            try_enqueue(row, col);
                        }
                    }

                    clues.push(search::Clue{mine_count: *val, adjacency});
                },
                CellState::UnknownConstrained => {
                    for (row, col) in self.neighbors_of(row, col) {
                        if let CellState::Clue(val) = self.get(row, col) {
                            if *val > 0u8 {
                                try_enqueue(row, col);
                            }
                        }
                    }
                },
                _ => panic!("Only constrained unknowns and clues must be part of graph")
            }
        }

        let unknown_count = unk_map.len() as u16;
        (unk_map, search::Topology{unknown_count, clues})
    }

    fn get_mut(&mut self, row: u8, col: u8) -> &mut CellState
    {
        &mut self.grid[usize::from(row)][usize::from(col)]
    }

    fn get(&self, row: u8, col: u8) -> &CellState
    {
        &self.grid[usize::from(row)][usize::from(col)]
    }

    pub fn print(&self) {
        print!("\n");
        for row in &self.grid {
            for cell in row {
                print!("{}", cell);
            }
            print!("\n");
        }
        print!("\n");
    }
}

impl std::fmt::Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let val = match self {
            CellState::Clue(val) if *val > 0u8 => val.to_string(),
            _ => String::from(match self {
                CellState::Empty => "E",
                CellState::Mine => "M",
                CellState::UnknownConstrained => "C",
                CellState::UnknownUnconstrained => "U",
                CellState::Clue(_) => " ",
            })
        };
        f.write_str(val.as_str())
    }
}

impl NeighborIterable for PartialSolution {
    fn width(&self) -> u8
    {
        self.width
    }
    fn height(&self) -> u8
    {
        self.height
    }
}
