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

use rand::Rng;

use crate::geography::Point;

use std::collections::HashSet;
use rand::prelude::SliceRandom;

/// Points have a inclusive start, and inclusive end
#[derive(Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub struct Area {
    pub start_offset: Point,
    pub end_offset: Point,
}

impl Area {
    pub fn new(start_offset: Point, end_offset: Point) -> Area {
        Area { start_offset, end_offset }
    }

    pub fn get_neighbors_of(&self, point: Point) -> impl Iterator<Item=Point> + '_ {
        point.neighbor_iterator().filter(move |p| {
            self.contains(p)
        })
    }

    pub fn iter(&self) -> AreaIterator {
        AreaIterator::new(*self)
    }


    /// Attempts to retrieve **number_of_points** randomly from the given area
    /// Returns None, if there are not enough cells available
    pub fn random_points(&self, number_of_points: usize, rng: &mut impl rand::RngCore) -> Option<Vec<Point>> {
        // When the probability of finding empty random cells is low, and to prevent getting stuck in a loop
        // Generate all the possible positions and then shuffle them randomly
        let number_of_cells = self.get_number_of_cells();
        if number_of_cells < number_of_points {
            error!("Not enough cells ({}) in area {:?} to allocate ({}) points to!", number_of_cells,self, number_of_points);
            None
        } else if (number_of_cells) < number_of_points * 2 {
            let mut points = Vec::with_capacity((number_of_cells) as usize);
            for a in self.start_offset.x..self.end_offset.x + 1 {
                for b in self.start_offset.y..self.end_offset.y {
                    let x = Point::new(a, b);
                    points.push(x);
                }
            }
            points.shuffle(rng);
            points.truncate(number_of_points as usize);
            Some(points)
        } else {
            let mut points: Vec<Point> = Vec::with_capacity(number_of_points as usize);
            let mut points_collision_checker: HashSet<Point> = HashSet::with_capacity(number_of_points as usize);
            while points.len() != (number_of_points as usize) {
                let rand_x = rng.gen_range(self.start_offset.x, self.end_offset.x + 1);
                let rand_y = rng.gen_range(self.start_offset.y, self.end_offset.y + 1);
                let new_point = Point::new(rand_x, rand_y);
                if !points_collision_checker.contains(&new_point) {
                    points.push(new_point);
                    points_collision_checker.insert(new_point);
                }
            }
            Some(points)
        }
    }

    pub fn get_random_point(&self, rng: &mut impl rand::RngCore) -> Point {
        let rand_x = rng.gen_range(self.start_offset.x, self.end_offset.x + 1);
        let rand_y = rng.gen_range(self.start_offset.y, self.end_offset.y + 1);

        Point::new(rand_x, rand_y)
    }

    pub fn contains(&self, point: &Point) -> bool {
        self.start_offset.x <= point.x && self.end_offset.x >= point.x
            && self.start_offset.y <= point.y && self.end_offset.y >= point.y
    }

    pub fn get_number_of_cells(&self) -> usize {
        ((self.end_offset.x - self.start_offset.x).abs() as usize + 1) * ((self.end_offset.y - self.start_offset.y).abs() as usize + 1)
    }
}

// We need to ignore the iter_index when comparing
impl PartialEq for Area {
    fn eq(&self, other: &Self) -> bool {
        self.start_offset == other.start_offset && self.end_offset == other.end_offset
    }
}

pub fn area_factory(start_point: Point, end_point: Point, size: i32) -> Vec<Area> {
    let feasible_houses_in_x_dim = (end_point.x - start_point.x + 1) / size;
    let feasible_houses_in_y_dim = (end_point.y - start_point.y + 1) / size;

    let mut areas = Vec::with_capacity((feasible_houses_in_y_dim * feasible_houses_in_x_dim) as usize);
    let mut current_start_point = start_point;

    for _i in 0..feasible_houses_in_y_dim {
        for _j in 0..feasible_houses_in_x_dim {
            let current_end_point: Point = Point::new(current_start_point.x + size - 1, current_start_point.y + size - 1);

            areas.push(Area::new(current_start_point, current_end_point));

            current_start_point.x = current_start_point.x + size;
        }

        current_start_point.x = start_point.x;
        current_start_point.y = current_start_point.y + size;
    }

    areas
}

pub struct AreaIterator {
    area: Area,
    iter_index: Point,
}

impl AreaIterator {
    pub fn new(area: Area) -> AreaIterator {
        AreaIterator {
            area,
            iter_index: Point::new(area.start_offset.x - 1, area.start_offset.y),
        }
    }
}

impl Iterator for AreaIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let mut new_x = self.iter_index.x + 1;
        let mut new_y = self.iter_index.y;
        if new_x > self.area.end_offset.x {
            new_x = self.area.start_offset.x;
            new_y += 1;
        }
        self.iter_index = Point::new(new_x, new_y);
        if new_y > self.area.end_offset.y {
            return Option::None;
        }
        Option::Some(self.iter_index)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;
    use std::iter::FromIterator;
    use rand::thread_rng;
    use super::*;

    fn get_area() -> Area {
        Area::new(Point { x: 0, y: 0 }, Point { x: 4, y: 4 })
    }


    fn assert_unique<T: Eq + Hash+Clone>(data: Vec<T>) {
        let mut set: HashSet<T> = HashSet::with_capacity(data.len());
        for d in data.clone(){
            set.insert(d);
        }
        assert_eq!(set.len(), data.len());
    }

    #[test]
    fn generate_points() {
        let area = get_area();
        let points: Option<Vec<Point>> = area.random_points(25, &mut thread_rng());
        assert!(points.is_some());
        let points=points.unwrap();
        assert_eq!(points.len(), 25);
        assert_unique(points);

        let points: Option<Vec<Point>> = area.random_points(5, &mut thread_rng());
        assert!(points.is_some());
        let points=points.unwrap();
        assert_eq!(points.len(), 5);
        assert_unique(points);
        let points: Option<Vec<Point>> = area.random_points(50, &mut thread_rng());
        assert!(points.is_none());

        let area = Area::new(Point::new(0, 0), Point::new(0, 0));
        let points: Option<Vec<Point>> = area.random_points(1, &mut thread_rng());
        assert!(points.is_some());
        let points=points.unwrap();
        assert_eq!(points.len(), 1);
        assert_unique(points);

        let points: Option<Vec<Point>> = area.random_points(0, &mut thread_rng());
        assert!(points.is_some());
        let points=points.unwrap();
        assert_eq!(points.len(), 0);
        assert_unique(points);

        let area = Area::new(Point::new(0, 0), Point::new(1, 1));
        let points: Option<Vec<Point>> = area.random_points(4, &mut thread_rng());
        assert!(points.is_some());
        let points=points.unwrap();
        assert_eq!(points.len(), 4);
        assert_unique(points);

        let points: Option<Vec<Point>> = area.random_points(2, &mut thread_rng());
        assert!(points.is_some());
        let points=points.unwrap();
        assert_eq!(points.len(), 2);
        assert_unique(points);
    }

    #[test]
    fn should_iterate_over_points_in_area() {
        let area = Area::new(Point { x: 0, y: 0 }, Point { x: 2, y: 2 });
        let x: Vec<Point> = area.iter().collect();
        assert_eq!(x, vec![Point::new(0, 0), Point::new(1, 0), Point::new(2, 0),
                           Point::new(0, 1), Point::new(1, 1), Point::new(2, 1),
                           Point::new(0, 2), Point::new(1, 2), Point::new(2, 2)]);

        let area = Area::new(Point { x: 1, y: 1 }, Point { x: 2, y: 2 });
        let x: Vec<Point> = area.iter().collect();
        assert_eq!(x, vec![Point::new(1, 1), Point::new(2, 1),
                           Point::new(1, 2), Point::new(2, 2)])
    }

    #[test]
    fn iterator_should_work_multiple_times() {
        let area = Area::new(Point { x: 0, y: 0 }, Point { x: 2, y: 2 });
        let x: Option<Point> = area.iter().find(|p| *p == Point::new(1, 1));
        assert!(x.is_some());

        let x: Option<Point> = area.iter().find(|p| *p == Point::new(1, 1));
        assert!(x.is_some());
    }

    #[test]
    fn should_create_areas() {
        let buildings = area_factory(Point::new(10, 0), Point::new(21, 10), 3);

        buildings.iter().for_each(|b| println!("start {:?}, end {:?}", b.start_offset, b.end_offset));

        assert_eq!(buildings.len(), 12);
        assert_eq!(buildings.get(0).unwrap().start_offset, Point::new(10, 0));
        assert_eq!(buildings.get(0).unwrap().end_offset, Point::new(12, 2));

        assert!(buildings.get(0).unwrap().contains(&Point::new(12, 0)));
        assert!(!buildings.get(1).unwrap().contains(&Point::new(12, 0)));
        assert!(buildings.get(1).unwrap().contains(&Point::new(13, 0)));

        assert!(buildings.get(0).unwrap().contains(&Point::new(12, 1)));
        assert!(!buildings.get(1).unwrap().contains(&Point::new(12, 1)));
        assert!(buildings.get(1).unwrap().contains(&Point::new(13, 1)));

        assert!(buildings.get(0).unwrap().contains(&Point::new(12, 2)));
        assert!(!buildings.get(1).unwrap().contains(&Point::new(12, 2)));
        assert!(buildings.get(1).unwrap().contains(&Point::new(13, 2)));

        assert_eq!(buildings.get(5).unwrap().start_offset, Point::new(13, 3));
        assert_eq!(buildings.get(5).unwrap().end_offset, Point::new(15, 5));

        assert_eq!(buildings.last().unwrap().start_offset, Point::new(19, 6));
        assert_eq!(buildings.last().unwrap().end_offset, Point::new(21, 8));
    }

    #[test]
    fn should_get_neighbor_of() {
        let area = get_area();
        let neighbors: Vec<Point> = area.get_neighbors_of(Point::new(4, 5)).collect();
        assert_eq!(neighbors.len(), 5);
        assert_eq!(neighbors.contains(&Point::new(4, 4)), true);
        assert_eq!(neighbors.contains(&Point::new(4, 6)), false);
    }

    #[test]
    fn should_return_true_if_area_has_point() {
        let area = get_area();

        let is_inside_area = area.contains(&Point::new(2, 3));
        assert_eq!(is_inside_area, true);
    }

    #[test]
    fn should_return_false_if_area_does_not_contain_point() {
        let area = get_area();

        let is_inside_area = area.contains(&Point::new(20, 3));
        assert_eq!(is_inside_area, false);
    }

    #[test]
    fn should_get_random_point() {
        let area = get_area();

        let random_point = area.get_random_point(&mut thread_rng());
        assert_eq!(area.contains(&random_point), true);
    }

    #[test]
    fn should_get_number_of_cells() {
        let area = get_area();

        assert_eq!(area.get_number_of_cells(), 25);
        let area = Area::new(Point::new(0, 0), Point::new(0, 0));
        assert_eq!(area.get_number_of_cells(), 1);

        let area = Area::new(Point::new(0, 0), Point::new(1, 1));
        assert_eq!(area.get_number_of_cells(), 4);
    }
}
