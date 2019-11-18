use crate::components::{Id, Physical};
use crate::game_state::Map;

pub fn successors(pos: &(i32, i32), id: &Id, obj: &Physical, map: &Map) -> Vec<((i32, i32), u32)> {
    let mut out = Vec::new();
    //println!("running successors");
    for iy in -1..2 {
        for ix in -1..2 {
            let mut traversable = true;
            if ix == 0 && iy == 0 {
                continue;
            }
            let px = ix + pos.0;
            let py = iy + pos.1;
            let vol = obj.get_taken_space((px as f32, py as f32));

            let mut some_in_bounds = false;

            for (x, y) in vol {
                //println!("vol {:?}", (x, y));
                if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
                    continue;
                }
                
                some_in_bounds = true;

                let index = (x + y * map.width as i32) as usize;
                let (obstruct, owner) = map.storage[index];

                //println!("successor ({}, {}) has obstruction {} owned by {:?}", x, y, obstruct, owner);

                if obstruct > 0.0 {
                    //println!("successor ({}, {}) has obstruction {} owned by {:?}", x, y, obstruct, owner);
                    if owner != *id {
                        traversable = false;

                        break;
                    }
                }
            }
            
            if px < 0 || py < 0 || px >= map.width as i32 || py >= map.height as i32 {
                continue;
            }
            if traversable && some_in_bounds {
                out.push(((px as i32, py as i32), 1));
            }

            
        }
    }
    /*if !(pos.0 < 0 || pos.1 < 0 || pos.0 >= map.width as i32 || pos.1 >= map.height as i32) {
        let index = (pos.0 + pos.1 * map.width as i32) as usize;
        //map.storage[index] = (1.0, Id::nil());
    }*/
    //println!("successors of {:?} are {:?}", *pos, out);
    out
}