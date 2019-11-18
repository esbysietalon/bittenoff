fn absdiff(a: i32, b: i32) -> u32 {
    (a - b).abs() as u32
}

pub fn distance(o: &(i32, i32), e: &((i32, i32), u32)) -> u32 {
    let dx = absdiff(o.0, (e.0).0);
    let dy = absdiff(o.1, (e.0).1);
    (e.1 * (dx + dy)) as u32
}
pub fn heuristic(o: &(i32, i32), e: &(i32, i32)) -> u32 {
    let dx = absdiff(o.0, e.0);
    let dy = absdiff(o.1, e.1);
    ((dx + dy)) as u32
}