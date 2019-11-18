use priority_queue::PriorityQueue;
use crate::navigation::successors;
use crate::navigation::distance;
use crate::components::{Id, Physical};
use crate::game_state::Map;
use std::cmp::Reverse;

pub struct InputData {
    pub id: Id, 
    pub obj: Physical, 
    pub map: Map,
}

pub fn find_path(origin: (i32, i32), end: (i32, i32), d: InputData) -> Option<(Vec<(i32, i32)>, u32)> {

    println!(">>find_path origin {:?} end {:?}", origin, end);

    let mut path = Vec::new();
    let mut curr = origin;

    let mut came_from = vec![(0, 0); (d.map.width * d.map.height) as usize];
    let mut cost_to = vec![-1; (d.map.width * d.map.height) as usize];

    let mut frontier = PriorityQueue::new();
    frontier.push(curr, 0);
    cost_to[(curr.0 + curr.1 * d.map.width as i32) as usize] = 0;

    while !frontier.is_empty() {
        curr = *frontier.peek().unwrap().0;

        println!(">>find_path curr {:?} on {:?} to {:?}", curr, origin, end);
        println!(">>find_path priority_queue {:?}", frontier);
        if curr == end {
            break;
        }

        let mut no_valid_succ = true;

        let neighbors = successors(&frontier.pop().unwrap().0, &d.id, &d.obj, &d.map);

        //println!(">>find_path neighbors are {:?}", neighbors);

        for succ in neighbors {
            let new_cost = cost_to[(curr.0 + curr.1 * d.map.width as i32) as usize] + succ.1 as i32;
            let succ_index = ((succ.0).0 + (succ.0).1 * d.map.width as i32) as usize;

            //println!("from ({}, {}) to ({}, {}) has cost {}", curr.0, curr.1, (succ.0).0, (succ.0).1, new_cost);

            if cost_to[succ_index] < 0 || new_cost < cost_to[succ_index] {
                cost_to[succ_index] = new_cost as i32;

                //println!("adding in {:?} with priority {:?}", succ.0, new_cost + distance(&end, &succ) as i32);
                frontier.push(succ.0, -(new_cost + distance(&end, &succ) as i32));
                came_from[succ_index] = curr;
                no_valid_succ = false;
            }
        }    

        frontier.pop();
        /*if no_valid_succ {
            println!("warning no valid successors found");
            return None;
        }*/
    }

    let mut curr = end;
    let mut cost = 0;
    while curr != origin {
        let index = (curr.0 + curr.1 * d.map.width as i32) as usize;
        cost += cost_to[index];
        path.insert(0, curr);
        curr = came_from[index];
    }    

    println!(">>find_path path is {:#?}", path);

    Some((path, cost as u32))
}
