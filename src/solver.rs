use super::grid;
use super::neighbor_iter::NeighborIterable;
use super::search;
use arrayvec::ArrayVec;
use bitvec::prelude as bv;
use itertools::izip;
use rand;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;

type Key = (u8, u8);

#[derive(Copy, Clone)]
enum CellState {
    UnknownUnconstrained,
    UnknownConstrained,
    Mine,
    Empty,
    Clue(u8),
}

struct GraphSolution {
    tile_map: HashMap<Key, u16>,
    alternatives: VecDeque<bv::BitVec>,
}

struct Counters {
    unconstrained_cells: u16,
    hidden_mines: u16,
}

impl grid::GridCounters<CellState> for Counters {
    fn notify_change(&mut self, from: &CellState, to: &CellState) {
        match *from {
            CellState::UnknownUnconstrained => {
                self.unconstrained_cells -= 1;
            }
            CellState::Mine => {
                // These states can never change
                assert!(matches!(*from, CellState::Mine));
                return;
            }
            _ => (),
        }

        match *to {
            CellState::Mine => self.hidden_mines -= 1,
            CellState::UnknownUnconstrained => panic!("Cell state can not be set to unconstrained"),
            _ => (),
        }
    }
}

pub struct PartialSolution {
    grid: grid::Grid<CellState, u8, Counters>,
    graphs_solutions: Vec<GraphSolution>,
}

#[derive(Copy, Clone, Debug)]
enum UpdateAction {
    CheckIfClueFindMines,
    ToMine,
    CheckIfClueFindEmpties,
    ToEmpty,
}

struct CartesianProduct<T> {
    curr: Vec<usize>,
    basis: Vec<Vec<T>>,
}

impl<T> CartesianProduct<T> {
    fn new(basis: impl IntoIterator<Item = impl IntoIterator<Item = T>>) -> Self {
        let basis: Vec<Vec<T>> = basis.into_iter().map(|v| v.into_iter().collect()).collect();
        Self {
            curr: vec![0; basis.len()],
            basis,
        }
    }
}

impl<T> Iterator for CartesianProduct<T>
where
    T: Copy,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.len() == 0 {
            return None;
        }
        let ret = Some(
            self.curr
                .iter()
                .enumerate()
                .map(|(i, val)| self.basis[i][*val])
                .collect(),
        );

        for (val, v) in izip!(self.curr.iter_mut(), self.basis.iter()) {
            *val += 1;
            if *val != v.len() {
                return ret;
            }

            *val = 0;
        }

        // All combinations have been generated, so stop execution in next iteration.
        self.curr.clear();

        ret
    }
}

impl PartialSolution {
    pub fn new(width: u8, height: u8, mine_count: u16) -> Self {
        let counters = Counters {
            unconstrained_cells: width as u16 * height as u16,
            hidden_mines: mine_count,
        };

        Self {
            grid: grid::Grid::new(width, height, counters, CellState::UnknownUnconstrained),
            graphs_solutions: Vec::new(),
        }
    }

    pub fn add_clue(&mut self, (row, col): Key, mut clue: u8) {
        let state = self.grid.get(row, col);

        // Check if we previously knew if the cell was empty,
        // and possibly update the state before anything else.
        match state {
            CellState::Clue(_) => panic!("Can't add clue to a revealed square!"),
            CellState::Mine => panic!("Can't add clue to a hidden mine!"),
            CellState::UnknownConstrained => {
                // We didn't knew this was empty, so we
                // must update the neighboring cells.
                self.breadth_first_update(UpdateAction::ToEmpty, &[(row, col)]);
            }
            _ => {}
        }

        let mut unknowns = ArrayVec::<Key, 8>::new();

        // Mark neighbors as constrained and check for known mines:
        for (row, col) in self.neighbors_of(row, col) {
            let cell = self.grid.get(row, col);
            match cell {
                CellState::UnknownUnconstrained => {
                    self.grid.set(row, col, CellState::UnknownConstrained);
                    unknowns.push((row, col));
                }
                CellState::UnknownConstrained => {
                    unknowns.push((row, col));
                }
                CellState::Mine => {
                    clue -= 1;
                }
                _ => {}
            }
        }

        // Set the state of the new clue cell:
        self.grid.set(row, col, CellState::Clue(clue));

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

    fn breadth_first_update(&mut self, action: UpdateAction, seed: &[Key]) {
        let mut is_queued: HashSet<Key> = HashSet::from_iter(seed.iter().copied());
        let mut queue: VecDeque<(Key, UpdateAction)> =
            is_queued.iter().map(|key| (*key, action)).collect();

        while let Some(((row, col), action)) = queue.pop_front() {
            is_queued.remove(&(row, col));

            let mut try_enqueue = |key, action| {
                if is_queued.insert(key) {
                    queue.push_back((key, action));
                }
            };

            match action {
                UpdateAction::CheckIfClueFindMines => {
                    let clue = if let CellState::Clue(clue) = self.grid.get(row, col) {
                        *clue
                    } else {
                        panic!("Must be a clue!");
                    };

                    let mut unknowns = ArrayVec::<Key, 8>::new();

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.grid.get(row, col) {
                            CellState::UnknownConstrained => unknowns.push((row, col)),
                            CellState::UnknownUnconstrained => {
                                panic!("Can't have unconstrained next to a clue!")
                            }
                            _ => (),
                        }
                    }

                    assert!(unknowns.len() as u8 >= clue);

                    if unknowns.len() as u8 == clue {
                        self.grid.set(row, col, CellState::Clue(0));

                        for (row, col) in unknowns {
                            try_enqueue((row, col), UpdateAction::ToMine);
                        }
                    }
                }

                UpdateAction::ToMine => {
                    assert!(matches!(
                        self.grid.get(row, col),
                        CellState::UnknownConstrained
                    ));
                    self.grid.set(row, col, CellState::Mine);

                    for (row, col) in self.neighbors_of(row, col) {
                        match *self.grid.get(row, col) {
                            CellState::Clue(mut val) if val > 0 => {
                                val -= 1;
                                if val == 0 {
                                    // A clue can only get to zero once,
                                    // so it can not be inserted twice:
                                    assert!(is_queued.insert((row, col)));
                                    queue.push_back((
                                        (row, col),
                                        UpdateAction::CheckIfClueFindEmpties,
                                    ));
                                }
                                self.grid.set(row, col, CellState::Clue(val));
                            }
                            _ => (),
                        }
                    }
                }

                UpdateAction::CheckIfClueFindEmpties => {
                    // You can only get empties from a 0 clue:
                    assert!(matches!(self.grid.get(row, col), CellState::Clue(0)));

                    for (row, col) in self.neighbors_of(row, col) {
                        match self.grid.get(row, col) {
                            CellState::UnknownConstrained => {
                                try_enqueue((row, col), UpdateAction::ToEmpty)
                            }
                            CellState::UnknownUnconstrained => {
                                panic!("Can't have unconstrained next to a clue!")
                            }
                            _ => (),
                        }
                    }
                }

                UpdateAction::ToEmpty => {
                    // Only constrained can be found to be empty:
                    assert!(matches!(
                        self.grid.get(row, col),
                        CellState::UnknownConstrained
                    ));
                    self.grid.set(row, col, CellState::Empty);

                    for (row, col) in self.neighbors_of(row, col) {
                        match *self.grid.get(row, col) {
                            CellState::Clue(val) if val > 0 => {
                                try_enqueue((row, col), UpdateAction::CheckIfClueFindMines)
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    pub fn find_graph_solutions(&mut self) {
        let mut visited =
            vec![bv::bitvec![0; self.grid.width() as usize]; self.grid.height() as usize];

        let mut graphs_solutions = Vec::new();
        for (i, row) in self.grid.rows().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let mut cell_visited = visited[i].get_mut(j).unwrap();
                if *cell_visited {
                    continue;
                }

                match cell {
                    CellState::Clue(val) if *val > 0 => {
                        cell_visited.set(true);
                        drop(cell_visited);

                        let (tile_map, topology) =
                            self.extract_graph_starting_from(i as u8, j as u8, &mut visited[..]);
                        let alternatives = search::find_solutions(&topology);
                        graphs_solutions.push(GraphSolution {
                            tile_map,
                            alternatives,
                        });
                    }
                    _ => (),
                }
            }
        }

        self.graphs_solutions = graphs_solutions;
    }

    fn extract_graph_starting_from(
        &self,
        row: u8,
        col: u8,
        visited: &mut [bv::BitVec],
    ) -> (HashMap<Key, u16>, search::Topology) {
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
                let mut visited = visited[row as usize].get_mut(col as usize).unwrap();
                if !*visited {
                    visited.set(true);
                    queue.push_back((row, col));
                }
            };

            match self.grid.get(row, col) {
                CellState::Clue(val) => {
                    assert!(*val > 0u8);

                    let mut adjacency = Vec::new();
                    for (row, col) in self.neighbors_of(row, col) {
                        if let CellState::UnknownConstrained = self.grid.get(row, col) {
                            let len = unk_map.len() as u16;
                            let unk_id = *unk_map.entry((row, col)).or_insert(len);
                            adjacency.push(unk_id);

                            try_enqueue(row, col);
                        }
                    }

                    clues.push(search::Clue {
                        mine_count: *val,
                        adjacency,
                    });
                }
                CellState::UnknownConstrained => {
                    for (row, col) in self.neighbors_of(row, col) {
                        if let CellState::Clue(val) = self.grid.get(row, col) {
                            if *val > 0u8 {
                                try_enqueue(row, col);
                            }
                        }
                    }
                }
                _ => panic!("Only constrained unknowns and clues must be part of a graph"),
            }
        }

        let unknown_count = unk_map.len() as u16;
        (
            unk_map,
            search::Topology {
                unknown_count,
                clues,
            },
        )
    }

    /// Tries to find a valid configuration where all cells in "reveal" are empty.
    ///
    /// self is modified assuming this function will succeed, so in case of return false,
    /// this partial solution should no longer be used.
    pub fn find_acomodating_solution(
        &mut self,
        rng: &mut impl rand::Rng,
        revealed: impl IntoIterator<Item = (u8, u8)>,
        mut reconfigure_tile: impl FnMut(u8, u8, bool),
    ) -> bool {
        let mut unconstrained_revealed = Vec::new();

        for key in revealed {
            let cell = *self.grid.get(key.0, key.1);
            match cell {
                CellState::UnknownConstrained => {
                    // Linear search through all the graphs (because there can't be many)
                    // for the one containing the key.
                    for sol in self.graphs_solutions.iter_mut() {
                        if let Some(idx) = sol.tile_map.get(&key) {
                            // Delete every alternative who has a mine at idx:
                            sol.alternatives.retain(|alt| !alt[*idx as usize]);
                            if sol.alternatives.len() == 0 {
                                return false;
                            }

                            // Each key can be in at most 1 graph, so there is no
                            // need to search the others
                            break;
                        }
                    }
                }
                CellState::UnknownUnconstrained => {
                    unconstrained_revealed.push(key);
                    self.grid.set(key.0, key.1, CellState::Empty);
                }
                CellState::Mine => {
                    return false;
                }
                CellState::Clue(_) => panic!("Tried to reveal an already revealed cell."),
                CellState::Empty => (),
            }
        }

        // Count the number of mines in each solution for each graph:
        let mine_counts: Vec<HashMap<u16, Vec<&bv::BitVec>>> = self
            .graphs_solutions
            .iter()
            .map(|sol| {
                let mut counts: HashMap<u16, Vec<&bv::BitVec>> = HashMap::new();
                for alt in sol.alternatives.iter() {
                    let count = alt.count_ones() as u16;
                    counts.entry(count).or_default().push(alt);
                }
                counts
            })
            .collect();

        let combinations = Vec::from_iter(
            CartesianProduct::new(mine_counts.iter().map(|x| x.keys().copied())).filter(|comb| {
                let total = comb.iter().copied().sum();

                // Do we have enough remaining mines to satisfy this
                // combination of solutions?
                if self.grid.counters.hidden_mines < total {
                    return false;
                }

                // Do we have enough unconstrained squares to fit all
                // the mines left over from this solution?
                if self.grid.counters.unconstrained_cells < self.grid.counters.hidden_mines - total
                {
                    return false;
                }

                true
            }),
        );

        // TODO: calculate the probability of each combination actually happening,
        // so that we have the weights to randomly select one solution.
        // For now, just sample uniformly from the combinations, and then sample
        // uniformly from graph solutions that makes up the combination:
        use rand::seq::SliceRandom;
        let mut replaced_mines = 0u16;
        if let Some(combination) = combinations.as_slice().choose(rng) {
            // Reconfigure constrained tiles
            for (mine_count, sols_per_count, graph) in
                izip!(combination, &mine_counts, &self.graphs_solutions)
            {
                replaced_mines += mine_count;

                let sol = *sols_per_count
                    .get(mine_count)
                    .unwrap()
                    .as_slice()
                    .choose(rng)
                    .unwrap();

                for ((row, col), idx) in graph.tile_map.iter() {
                    reconfigure_tile(*row, *col, sol[*idx as usize]);
                }
            }
        }

        // Clear just revealed unconstrained tiles from mines:
        for (row, col) in unconstrained_revealed {
            reconfigure_tile(row, col, false);
        }

        // Reconfigure unconstrained tiles:
        assert!(self.grid.counters.hidden_mines >= replaced_mines);
        let remaining_mines = self.grid.counters.hidden_mines - replaced_mines;

        assert!(self.grid.counters.unconstrained_cells >= remaining_mines);
        let mut shuffled_mines = vec![true; remaining_mines as usize];
        shuffled_mines.resize(self.grid.counters.unconstrained_cells as usize, false);
        shuffled_mines.shuffle(rng);

        for (i, row) in self.grid.rows().enumerate() {
            for (k, cell) in row.iter().enumerate() {
                if let CellState::UnknownUnconstrained = cell {
                    reconfigure_tile(i as u8, k as u8, shuffled_mines.pop().unwrap());
                }
            }
        }
        assert!(shuffled_mines.len() == 0);

        true
    }

    pub fn print(&self) {
        let mut map = HashMap::new();
        for (i, gs) in self.graphs_solutions.iter().enumerate() {
            for key in gs.tile_map.keys() {
                assert!(!map.contains_key(key));
                map.insert(key, i);
            }
        }

        print!("\n");
        for (i, row) in self.grid.rows().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if let CellState::UnknownConstrained = cell {
                    print!("\u{20dd}{}", map.get(&(i as u8, j as u8)).unwrap());
                } else {
                    print!("{}", cell);
                }
            }
            print!("\n");
        }
        print!("\n");

        //    for (i, gs) in self.graphs_solutions.iter().enumerate() {
        //        println!("{}:", i);
        //        for s in &gs.alternatives {
        //            println!("  {}", s);
        //        }
        //    }
        //    print!("\n");
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
            }),
        };
        f.write_str(val.as_str())
    }
}

impl NeighborIterable for PartialSolution {
    fn width(&self) -> u8 {
        self.grid.width()
    }
    fn height(&self) -> u8 {
        self.grid.height()
    }
}
