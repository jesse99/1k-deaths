use super::point::Point;
use super::size::Size;
use std::fmt;

#[derive(Clone)]
pub struct Vec2d<T> {
    size: Size,
    elements: Vec<T>,
}

#[cfg(test)] // for now this is only used within unit tests
pub struct Vec2dIter<'a, T: 'a> {
    index: usize,
    vector: &'a Vec2d<T>,
}

impl<T: Clone> Vec2d<T> {
    // pub fn empty() -> Vec2d<T> {
    // 	Vec2d {
    // 		size: Size::zero(),
    // 		elements: Vec::new(),
    // 	}
    // }

    #[cfg(test)] // for now this is only used within unit tests
    pub fn new(size: Size, default: T) -> Vec2d<T> {
        let elements = vec![default; size.area() as usize];
        Vec2d { size, elements }
    }

    // #[cfg(test)] // for now this is only used within unit tests
    // pub fn size(&self) -> Size {
    //     self.size
    // }

    #[cfg(test)] // for now this is only used within unit tests
    pub fn set(&mut self, loc: Point, value: T) {
        let index = loc.x + loc.y * self.size.width;
        self.elements[index as usize] = value;
    }

    pub fn get(&self, loc: Point) -> &T {
        let index = loc.x + loc.y * self.size.width;
        &self.elements[index as usize]
    }

    // pub fn get_mut(&mut self, loc: Point) -> &mut T {
    // 	let index = loc.x + loc.y * self.size.width;
    // 	&mut self.elements[index as usize]
    // }

    #[cfg(test)] // for now this is only used within unit tests
    pub fn iter(&self) -> Vec2dIter<T> {
        Vec2dIter { index: 0, vector: self }
    }

    // More elegant to use a mutable iterator here but that requires an unsafe block, see https://users.rust-lang.org/t/implementing-an-iterator-of-mutable-references/8671
    // pub fn apply<F: Fn(Point, &mut T)>(&mut self, mutate: F) {
    // 	for i in 0..self.elements.len() {
    // 		let x = (i % self.size.width as usize) as i32;
    // 		let y = (i / self.size.width as usize) as i32;
    // 		let loc = Point::new(x, y);
    // 		let val = self.elements.get_mut(i);
    // 		mutate(loc, val.unwrap());
    // 	}
    // }
}

#[cfg(test)] // for now this is only used within unit tests
impl<'a, T> Iterator for Vec2dIter<'a, T> {
    type Item = (Point, &'a T);

    fn next(&mut self) -> Option<(Point, &'a T)> {
        if self.index < self.vector.elements.len() {
            let i = self.index;
            self.index += 1;

            let x = (i % self.vector.size.width as usize) as i32;
            let y = (i / self.vector.size.width as usize) as i32;
            let loc = Point::new(x, y);
            let val = self.vector.elements.get(i);
            Some((loc, val.unwrap()))
        } else {
            None
        }
    }
}

impl<T: Clone + fmt::Display> fmt::Display for Vec2d<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let loc = Point::new(x, y);
                write!(f, "{}", self.get(loc))?;
            }
            if y + 1 < self.size.height {
                writeln!(f)?;
            }
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let size = Size::new(2, 2);
        let mut v = Vec2d::new(size, '.');
        v.set(Point::new(1, 0), 'a');
        v.set(Point::new(0, 1), 'b');
        v.set(Point::new(1, 1), 'c');

        let mut locs = Vec::new();
        let mut values = Vec::new();
        for (loc, ch) in v.iter() {
            locs.push(loc);
            values.push(*ch);
        }

        assert_eq!(
            locs,
            vec![Point::new(0, 0), Point::new(1, 0), Point::new(0, 1), Point::new(1, 1)]
        );
        assert_eq!(values, vec!['.', 'a', 'b', 'c']);
    }
}
