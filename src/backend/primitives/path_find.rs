#[cfg(test)]
use super::super::time;
#[cfg(test)]
use super::super::Time;
#[cfg(test)]
use super::vec2d::Vec2d;
use super::Point;
#[cfg(test)]
use super::Size;
use fnv::FnvHashMap;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::{Add, AddAssign};

/// Dijkstra's path finding algorithm, see http://www.roguebasin.com/index.php/Pathfinding
/// and https://en.wikipedia.org/wiki/Pathfinding.
pub struct PathFind<C, S>
where
    // It'd be a lot nicer to require Add instead of AddAssign but it's tough to make the
    // compiler happy doing that.
    C: Copy + Ord + Add + AddAssign + Default, // for movement C will be Time
    S: Fn(Point, &mut Vec<(Point, C)>),        // push neighbors of the point onto the vector
{
    start: Point,
    target: Point,
    successors: S,
    path: Vec<Point>,
    total_cost: C,
}

// In a lot of ways Dijkstra Maps would be better (see http://www.roguebasin.com/index.php/The_Incredible_Power_of_Dijkstra_Maps
// and http://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized). However they work best
// with relatively small compact maps and we're currently being very general with the sort
// of maps we support (that's why we're using a hashmap for levels instead of Vec2d).
impl<C, S> PathFind<C, S>
where
    C: Copy + Ord + Add + AddAssign + Default,
    S: Fn(Point, &mut Vec<(Point, C)>),
{
    /// Successors should push the neighbors of the provided point onto the provided
    /// vector along with the time it would take to move to that neighbor.
    pub fn new(start: Point, target: Point, successors: S) -> PathFind<C, S> {
        let mut find = PathFind {
            start,
            target,
            successors,
            path: Vec::new(),
            total_cost: C::default(),
        };
        find.compute();
        find
    }

    /// Returns the distance of the shortest path from start to target (or None if a path
    /// could not be found).
    pub fn distance(&mut self) -> Option<C> {
        // TODO: need to somehow apply a penalty for stuff like closed doors
        if self.path.is_empty() {
            None
        } else {
            Some(self.total_cost)
        }
    }

    /// Returns the next point on the shortest path from start to target (or None if a
    /// path could not be found).
    pub fn next(&mut self) -> Option<Point> {
        if self.path.len() > 1 {
            None
        } else {
            Some(self.path[1]) // first entry is start
        }
    }

    // #[cfg(test)]
    pub fn path(&mut self) -> &Vec<Point> {
        &self.path
    }
}

impl<C, S> PathFind<C, S>
where
    C: Copy + Ord + Add + AddAssign + Default,
    S: Fn(Point, &mut Vec<(Point, C)>),
{
    fn compute(&mut self) {
        let mut queue = BinaryHeap::new();
        let mut dist = FnvHashMap::default(); // loc => cost to reach loc from start, predecessor of loc

        // We're at start with a zero cost.
        queue.push(State {
            cost: C::default(),
            loc: self.start,
        });
        dist.insert(
            self.start,
            State {
                cost: C::default(),
                loc: self.start, // predecessor of start is start...
            },
        );

        // Examine the frontier with lower cost nodes first (min-heap)
        let mut neighbors = Vec::new();
        while let Some(State { cost, loc }) = queue.pop() {
            if loc == self.target {
                self.total_cost = dist.get(&self.target).unwrap().cost;
                self.build_path(dist);
                return;
            }

            // Important as we may have already found a better way
            if cost > dist.get(&loc).unwrap().cost {
                continue;
            }

            // For each node we can reach, see if we can find a way with a lower cost
            // going through this node
            neighbors.clear();
            (self.successors)(loc, &mut neighbors);
            for (next_loc, edge_cost) in &neighbors {
                let mut new_cost = cost; // can't use + or compiler gets upset
                new_cost += *edge_cost;
                let next = State {
                    cost: new_cost,
                    loc: *next_loc,
                };

                // If so, add it to the frontier and continue
                if dist.get(&next_loc).map_or(true, |s| next.cost < s.cost) {
                    queue.push(next);
                    // Relaxation, we have now found a better way
                    dist.insert(next.loc, State { cost: next.cost, loc });
                }
            }
        }
    }

    fn build_path(&mut self, dist: FnvHashMap<Point, State<C>>) {
        let mut loc = self.target;
        loop {
            self.path.push(loc);
            if loc == self.start {
                break;
            }
            loc = dist.get(&loc).unwrap().loc;
        }
        self.path.reverse();
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State<C>
where
    C: Copy + Ord + Add + AddAssign + Default,
{
    cost: C,
    loc: Point,
}

impl<C> Ord for State<C>
where
    C: Copy + Ord + Add + AddAssign + Default,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // Order is flipped so that we get a min priority queue. We fall back onto comparing
        // location to ensure that this Op is compatible with the Eq Op.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| (self.loc.x + self.loc.y).cmp(&(other.loc.x + other.loc.y)))
    }
}

impl<C> PartialOrd for State<C>
where
    C: Copy + Ord + Add + AddAssign + Default,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_solution() {
        #[rustfmt::skip]
        let rows = vec![
          // 01234567890123
            "..............", // 0
            "..............", // 1
            "......######..", // 2
            "......#....#..", // 3
            "......#....#..", // 4
            ".S....#..T.#..", // 5
            "......######..", // 6
            "..............", // 7
            "..............", // 8
        ];
        let (start, target, map) = build_map(rows);
        let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(&map, loc, neighbors);
        let mut find = PathFind::new(start, target, callback);
        let actual = find.path();

        let expected = Vec::new();
        assert_eq!(*actual, expected);
    }

    #[test]
    fn test_neighbor() {
        #[rustfmt::skip]
        let rows = vec![
          // 0123
            "....", // 0
            "....", // 1
            "....", // 2
            "....", // 3
            "....", // 4
            ".ST.", // 5
            "....", // 6
            "....", // 7
            "....", // 8
        ];
        let (start, target, map) = build_map(rows);
        let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(&map, loc, neighbors);
        let mut find = PathFind::new(start, target, callback);
        let actual = find.path();

        let expected = vec![Point::new(1, 5), Point::new(2, 5)];
        assert_eq!(*actual, expected);
    }

    #[test]
    fn test_coincident() {
        #[rustfmt::skip]
        let rows = vec![
          // 0123
            "....", // 0
            "....", // 1
            "....", // 2
            "....", // 3
            "....", // 4
            ".ST.", // 5
            "....", // 6
            "....", // 7
            "....", // 8
        ];
        let (start, _, map) = build_map(rows);
        let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(&map, loc, neighbors);
        let mut find = PathFind::new(start, start, callback);
        let actual = find.path();

        let expected = vec![Point::new(1, 5)];
        assert_eq!(*actual, expected);
    }

    #[test]
    fn test_column() {
        #[rustfmt::skip]
        let rows = vec![
          // 012345678901
            "............", // 0
            "............", // 1
            "......#.....", // 2
            "......#.....", // 3
            "......#.....", // 4
            ".S....#..T..", // 5
            "......#.....", // 6
            "............", // 7
            "............", // 8
        ];
        let (start, target, map) = build_map(rows);
        let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(&map, loc, neighbors);
        let mut find = PathFind::new(start, target, callback);
        let actual = find.path();

        let expected = vec![
            Point::new(1, 5),
            Point::new(2, 5),
            Point::new(3, 5),
            Point::new(4, 5),
            Point::new(5, 6),
            Point::new(6, 7),
            Point::new(7, 7),
            Point::new(8, 6),
            Point::new(9, 5),
        ];
        assert_eq!(*actual, expected);
    }

    #[test]
    fn test_bad_terrain() {
        #[rustfmt::skip]
        let rows = vec![
          // 012345678901
            "............", // 0
            "............", // 1
            "......~.....", // 2
            "......~.....", // 3
            "......~~~~..", // 4
            ".S....~~T~..", // 5
            "......~~~~..", // 6
            "............", // 7
            "............", // 8
        ];
        let (start, target, map) = build_map(rows);
        let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(&map, loc, neighbors);
        let mut find = PathFind::new(start, target, callback);
        let actual = find.path();

        let expected = vec![
            Point::new(1, 5),
            Point::new(2, 5),
            Point::new(3, 5),
            Point::new(4, 5),
            Point::new(5, 6),
            Point::new(6, 7),
            Point::new(7, 6),
            Point::new(8, 5),
        ];
        assert_eq!(*actual, expected);
    }

    #[test]
    fn test_hole() {
        #[rustfmt::skip]
        let rows = vec![
          // 01234567890123
            "..............", // 0
            "..............", // 1
            "......######..", // 2
            "......#.......", // 3
            "......#....#..", // 4
            ".S....#..T.#..", // 5
            "......######..", // 6
            "..............", // 7
            "..............", // 8
        ];
        let (start, target, map) = build_map(rows);
        let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(&map, loc, neighbors);
        let mut find = PathFind::new(start, target, callback);
        let actual = find.path();

        let expected = vec![
            Point::new(1, 5),
            Point::new(2, 5),
            Point::new(3, 4),
            Point::new(4, 3),
            Point::new(5, 2),
            Point::new(6, 1),
            Point::new(7, 1),
            Point::new(8, 1),
            Point::new(9, 1),
            Point::new(10, 1),
            Point::new(11, 1),
            Point::new(12, 2),
            Point::new(11, 3),
            Point::new(10, 4),
            Point::new(9, 5),
        ];
        assert_eq!(*actual, expected);
    }

    fn build_map(rows: Vec<&'static str>) -> (Point, Point, Vec2d<char>) {
        let size = Size::new(rows[0].len() as i32, rows.len() as i32);
        let mut start = Point::origin();
        let mut target = Point::origin();
        let mut map = Vec2d::new(size, '.');
        for (v, row) in rows.iter().enumerate() {
            for (h, ch) in (*row).chars().enumerate() {
                let pt = Point::new(h as i32, v as i32);
                if ch == 'S' {
                    start = pt;
                } else if ch == 'T' {
                    target = pt;
                } else {
                    map.set(pt, ch);
                }
            }
        }
        (start, target, map)
    }

    fn successors(map: &Vec2d<char>, loc: Point, neighbors: &mut Vec<(Point, Time)>) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        let size = map.size();
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if new_loc.x >= 0 && new_loc.y >= 0 && new_loc.x < size.width && new_loc.y < size.height {
                if *map.get(new_loc) == '~' {
                    neighbors.push((new_loc, time::CARDINAL_MOVE * 5));
                } else if *map.get(new_loc) != '#' {
                    if loc.diagnol(&new_loc) {
                        neighbors.push((new_loc, time::DIAGNOL_MOVE));
                    } else {
                        neighbors.push((new_loc, time::CARDINAL_MOVE));
                    }
                }
            }
        }
    }
}
