use bitvec::prelude as bv;
use std::collections::VecDeque;

struct Clue {
    mine_count: u8,
    adjacency: Vec<u16>
}

struct Topology {
    unknown_count: u16,
    clues: Vec<Clue>
}

fn find_solutions(topology: &Topology) -> VecDeque::<bv::BitVec>
{
    // Create a reverse map of unknows to the clues:
    let unknowns_to_clues = {
        let mut unknowns_to_clues = vec![Vec::<u16>::new(); topology.unknown_count as usize];
        for (i, clue) in topology.clues.iter().enumerate() {
            for unknown in &clue.adjacency {
                unknowns_to_clues[*unknown as usize].push(i as u16);
            }
        }

        unknowns_to_clues
    };

    // Find all solutions
    let mut solutions = VecDeque::<bv::BitVec>::new();
    solutions.push_back(bv::BitVec::new());

    loop {
        if let Some(mut sol) = solutions.pop_front() {
            if sol.len() >= topology.unknown_count as usize {
                // There should be only complete solutions remaining, return them.
                solutions.push_front(sol);
                break;
            }

            let to_clues = &*unknowns_to_clues[sol.len()];

            sol.push(false);
            if is_last_possible(&topology, to_clues, &sol) {
                solutions.push_back(sol.clone());
            }
            sol.pop();

            sol.push(true);
            if is_last_possible(&topology, to_clues, &sol) {
                solutions.push_back(sol);
            }
        } else {
            // Since the list is empty, solution is impossible.
            break;
        }
    }

    solutions
}

fn is_last_possible(topology: &Topology, to_clues: &[u16], sol: &bv::BitVec) -> bool
{
    for clue_idx in to_clues {
        let mut mine_count = 0;
        let mut unknown_count = 0;
        let clue = &topology.clues[*clue_idx as usize];
        for unk_idx in &clue.adjacency {
            if let Some(is_mine) = sol.get(*unk_idx as usize) {
                if *is_mine {
                    mine_count += 1;
                    if mine_count > clue.mine_count {
                        // More mines than needed, impossible
                        return false;
                    }
                }
            } else {
                unknown_count += 1;
            }
        }
        if unknown_count + mine_count < clue.mine_count {
            // Not enough mines to fulfill the clue, impossible
            return false;
        }
    }
    true
}
