/*
 * EpiRust
 * Copyright (c) 2020  ThoughtWorks, Inc.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

use std::ops::{Add, Sub};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    //    TODO: remove pub usages
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    /// Returns an iterator for walking through the points surrounding this point
    /// Does NOT ensure that the points belong to any grid or area. The consumer needs to filter out
    /// neighbors that are not valid
    pub fn neighbor_iterator(&self) -> NeighborIterator {
        NeighborIterator::new(*self)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, second_point: Self) -> Self {
        Self { x: self.x + second_point.x, y: self.y + second_point.y }
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.y, y: self.y - rhs.y }
    }
}

pub struct NeighborIterator {
    point: Point,
    offsets: [(i32, i32); 8],
    index: i32,
}

impl NeighborIterator {
    fn new(point: Point) -> NeighborIterator {
        NeighborIterator {
            point,
            offsets: [(-1, -1), (0, -1), (1, -1),
                (-1, 0), (1, 0),
                (-1, 1), (0, 1), (1, 1)],
            index: -1,
        }
    }
}

impl Iterator for NeighborIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.offsets.get(self.index as usize).map(|offset| {
            self.point + Point::new(offset.0, offset.1)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let point = Point::new(1, 1);
        let second_point = Point::new(1, 1);

        let output = point + second_point;
        assert_eq!(output, Point::new(2, 2));
    }
    #[test]
    fn sub() {
        let point = Point::new(1, 1);
        let second_point = Point::new(1, 1);

        let output = point - second_point;
        assert_eq!(output, Point::new(0, 0));
    }

    #[test]
    fn should_iterate_over_neighbor_cells() {
        let cell = Point::new(1, 1);
        let neighbors: Vec<Point> = cell.neighbor_iterator().collect();
        assert_eq!(neighbors,
                   vec![Point::new(0, 0), Point::new(1, 0), Point::new(2, 0),
                        Point::new(0, 1), Point::new(2, 1),
                        Point::new(0, 2), Point::new(1, 2), Point::new(2, 2),
                   ])
    }
}
