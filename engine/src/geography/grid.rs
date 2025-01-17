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

use plotters::prelude::*;

use crate::{agent, constants};
use crate::agent::{Citizen, PopulationRecord};
use crate::config::{AutoPopulation, CsvPopulation, StartingInfections};
use crate::geography::{Area, Point};

use std::fs::File;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Grid {
    pub grid_size: i32,
    pub housing_area: Area,
    pub work_area: Area,
    pub transport_area: Area,
    pub hospital_area: Area,
    pub houses: Vec<Area>,
    pub offices: Vec<Area>,

    //Occupancy based on home and work locations - updated when travellers arrive/depart
    #[serde(skip_serializing)]
    pub houses_occupancy: HashMap<Area, i32>,
    #[serde(skip_serializing)]
    pub offices_occupancy: HashMap<Area, i32>,
}

impl Grid {
    pub fn generate_population(&mut self, auto_pop: &AutoPopulation, start_infections: &StartingInfections,
                               rng: &mut impl rand::RngCore) -> (Vec<Point>, Vec<Citizen>) {
        debug!("Generating Population");
        let number_of_agents = auto_pop.number_of_agents;
        let working_percentage = auto_pop.working_percentage;
        let public_transport_percentage = auto_pop.public_transport_percentage;

        //        TODO: fix the hack
        let number_of_agents_using_public_transport = number_of_agents as f64 * (public_transport_percentage + 0.1) * (working_percentage + 0.1);

        let transport_locations = self.transport_area.random_points(number_of_agents_using_public_transport.ceil() as usize, rng).expect("Not enough transport locations for the population!");
        debug!("Finished generating transport locations");

        let agent_list = agent::citizen_factory(number_of_agents, &self.houses, &self.offices,
                                                &transport_locations, public_transport_percentage, working_percentage,
                                                rng, start_infections);
        debug!("Finished creating agent list");

        let (home_loc, agents_in_order) = self.set_start_locations_and_occupancies(rng, &agent_list);

        self.draw(&home_loc, &self.houses, &self.offices);
        assert_eq!(home_loc.len(),agents_in_order.len());
        (home_loc,agents_in_order)
    }

    /// Makes sure that each house, has enough squares for the given number of agents
    ///
    /// TODO WHAT THE FUCK
    /// Why are we generating a new agent list that is an exact copy of the old data???
    fn set_start_locations_and_occupancies(&mut self, rng: &mut impl rand::RngCore, agent_list: &Vec<Citizen>) -> (Vec<Point>, Vec<Citizen>) {
        let mut home_loc: Vec<Point> = Vec::new();
        let agents_by_home_locations = Grid::group_agents_by_home_locations(&agent_list);
        let house_capacity = constants::HOME_SIZE * constants::HOME_SIZE;
        debug!("Finished grouping agents by home locations");
        let mut agents_in_order: Vec<Citizen> = Vec::with_capacity(agent_list.len());
        for (home, agents) in agents_by_home_locations {
            trace!("home: {:?} {:?}", home.start_offset, home.end_offset);
            trace!("agents in home: {:?}", agents.len());

            if agents.len() as i32 > house_capacity {
                panic!("There are {} agents assigned to a house, but house capacity is {}",
                       agents.len(), house_capacity)
            }

            let mut random_points_within_home = home.random_points(agents.len(), rng).expect("Not enough points in the home for agents",);
            assert_eq!(random_points_within_home.len(),agents.len());
            self.houses_occupancy.insert(*home, agents.len() as i32);

            for agent in agents {
                agents_in_order.push(*agent);
            }
            home_loc.append(&mut random_points_within_home.iter_mut().map(|p|home.start_offset+*p).collect());
        }
        debug!("Assigned starting location to agents");
        self.offices_occupancy = self.group_office_locations_by_occupancy(agents_in_order.as_slice());
        (home_loc, agents_in_order)
    }


    /// Takes a list of Citizen's and returns a hashmap of the agents that reside at the same position
    pub fn group_agents_by_home_locations(agent_list: &Vec<Citizen>) -> HashMap<&Area, Vec<&Citizen>> {
        let mut agents_by_home_locations: HashMap<&Area, Vec<&Citizen>> = HashMap::new();
        agent_list.iter().for_each(|agent| {
            match agents_by_home_locations.get(&agent.home_location) {
                None => {
                    agents_by_home_locations.insert(&agent.home_location, vec![&agent]);
                }
                Some(citizens) => {
                    let mut updated_citizens = citizens.clone();
                    updated_citizens.push(&agent);
                    agents_by_home_locations.insert(&agent.home_location, updated_citizens);
                }
            }
        });
        agents_by_home_locations
    }
    /// Creates a png file, of the worls
    /// Where each vertical slice is colour coded, to the area type (Home - Yellow, Transport - Grey, Work - Dark Blue , Hospital - Red)
    /// Draws the actual homes,
    fn draw(&self, home_locations: &Vec<Point>, homes: &Vec<Area>, offices: &Vec<Area>) {
        let mut draw_backend = BitMapBackend::new("grid.png", (self.grid_size as u32, self.grid_size as u32));
        Grid::draw_rect(&mut draw_backend, &self.housing_area, &plotters::style::YELLOW);
        Grid::draw_rect(&mut draw_backend, &self.transport_area, &plotters::style::RGBColor(121, 121, 121));
        Grid::draw_rect(&mut draw_backend, &self.work_area, &plotters::style::BLUE);
        Grid::draw_rect(&mut draw_backend, &self.hospital_area, &plotters::style::RED);
        for home in homes {
            Grid::draw_rect(&mut draw_backend, home, &plotters::style::RGBColor(204, 153, 0));
        }
        for office in offices {
            Grid::draw_rect(&mut draw_backend, office, &plotters::style::RGBColor(51, 153, 255));
        }
        for home in home_locations {
            draw_backend.draw_pixel((home.x, home.y), &plotters::style::BLACK.to_rgba()).unwrap();
        }
    }

    fn draw_rect(svg: &mut impl DrawingBackend, area: &Area, style: &RGBColor) {
        svg.draw_rect((area.start_offset.x, area.start_offset.y),
                      (area.end_offset.x, area.end_offset.y),
                      style, true).unwrap();
    }

    pub fn read_population(&mut self, csv_pop: &CsvPopulation, starting_infections: &StartingInfections,
                           rng: &mut impl rand::RngCore) -> (Vec<Point>, Vec<Citizen>) {
        let file = File::open(&csv_pop.file).expect("Could not read population file");
        let mut rdr = csv::Reader::from_reader(file);
        let mut homes_iter = self.houses.iter().cycle();
        let mut offices_iter = self.offices.iter().cycle();

        let mut citizens = Vec::new();
        for result in rdr.deserialize() {
            let record: PopulationRecord = result.expect("Could not deserialize population line");

            //TODO seems like transport point isn't being used on the routine() function
            let home = *homes_iter.next().unwrap();
            let office = *offices_iter.next().unwrap();
            let citizen = Citizen::from_record(record, home, office, home.get_random_point(rng), rng);
            citizens.push(citizen);
        }
        let house_capacity = (constants::HOME_SIZE * constants::HOME_SIZE) as usize;
        if citizens.len() > house_capacity * self.houses.len() {
            panic!("Cannot accommodate citizens into homes! There are {} citizens, but {} home points",
                   citizens.len(), house_capacity * self.houses.len());
        }

        let (home_loc, mut agents_in_order) = self.set_start_locations_and_occupancies(rng, &citizens);
        agent::set_starting_infections(&mut agents_in_order, starting_infections, rng);

        self.draw(&home_loc, &self.houses, &self.offices);
        (home_loc, agents_in_order)
    }

    pub fn increase_hospital_size(&mut self, grid_size: i32) {
        let start_offset = self.hospital_area.start_offset;
        let end_offset = Point::new(grid_size, grid_size);

        self.hospital_area = Area::new(start_offset, end_offset)
    }

    pub fn resize_hospital(&mut self, number_of_agents: usize, hospital_staff_percentage: f64, hospital_beds_percentage: f64) {
        let hospital_bed_count = (number_of_agents as f64 * hospital_beds_percentage +
            number_of_agents as f64 * hospital_staff_percentage).ceil() as usize;

        if !(hospital_bed_count > self.hospital_area.get_number_of_cells()) {
            let hospital_end_y = hospital_bed_count as i32 / (self.hospital_area.end_offset.x - self.hospital_area.start_offset.x);
            self.hospital_area = Area::new(self.hospital_area.start_offset, Point::new(self.hospital_area.end_offset.x, hospital_end_y));
            info!("Hospital capacity {}: ", hospital_bed_count);
        }
    }

    // pub fn group_home_locations_by_occupancy(&self, citizens: &[&Citizen]) -> HashMap<Area, i32> {
    //     let mut occupancy = HashMap::new();
    //     self.houses.iter().for_each(|house| {
    //         occupancy.insert(*house, 0);
    //     });
    //     citizens.iter().for_each(|citizen| {
    //         let home = citizen.home_location;
    //         *occupancy.get_mut(&home).expect("Unknown home! Doesn't exist in grid") += 1;
    //     });
    //     occupancy
    // }

    /// Retrieves the number of Citizens per work building
    pub fn group_office_locations_by_occupancy(&self, citizens: &[Citizen]) -> HashMap<Area, i32> {
        let mut occupancy = HashMap::new();
        self.offices.iter().for_each(|house| {
            occupancy.insert(*house, 0);
        });
        citizens.iter().filter(|citizen| citizen.is_working())
            .for_each(|worker| {
                let office = worker.work_location;
                *occupancy.get_mut(&office).expect("Unknown office! Doesn't exist in grid") += 1;
            });
        occupancy
    }

    pub fn choose_house_with_free_space(&self, _rng: &mut impl rand::RngCore) -> Area {
        let house_capacity = constants::HOME_SIZE * constants::HOME_SIZE;
        *self.houses_occupancy.iter().find(|(_house, occupants)| **occupants < house_capacity)
            .expect("Couldn't find any house with free space!").0
    }

    pub fn choose_office_with_free_space(&self, _rng: &mut impl rand::RngCore) -> Area {
        let office_capacity = constants::OFFICE_SIZE * constants::OFFICE_SIZE;
        *self.offices_occupancy.iter().find(|(_house, occupants)| **occupants < office_capacity)
            .expect("Couldn't find any offices with free space!").0
    }

    pub fn add_house_occupant(&mut self, house: &Area) {
        *self.houses_occupancy.get_mut(house).expect("Could not find house!") += 1;
    }

    pub fn add_office_occupant(&mut self, office: &Area) {
        *self.offices_occupancy.get_mut(office).expect("Could not find office!") += 1;
    }

    pub fn remove_house_occupant(&mut self, house: &Area) {
        *self.houses_occupancy.get_mut(house).expect("Could not find house!") -= 1;
    }

    pub fn remove_office_occupant(&mut self, office: &Area) {
        *self.offices_occupancy.get_mut(office).expect("Could not find office!") -= 1;
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use super::*;
    use crate::geography::define_geography;

    #[test]
    fn should_generate_population() {
        let mut rng = thread_rng();

        let mut grid = define_geography(100);
        let housing_area = grid.housing_area;
        let transport_area = grid.transport_area;
        let work_area = grid.work_area;

        let pop = AutoPopulation {
            number_of_agents: 10,
            public_transport_percentage: 0.2,
            working_percentage: 0.2,
        };
        let start_infections = StartingInfections::new(0, 0, 0, 1);
        let (home_locations, agent_list) = grid.generate_population(&pop, &start_infections, &mut rng);

        assert_eq!(home_locations.len(), 10);
        assert_eq!(agent_list.len(), 10);

        for agent in agent_list {
            assert!(housing_area.contains(&agent.home_location.start_offset));
            assert!(work_area.contains(&agent.work_location.end_offset)
                || housing_area.contains(&agent.home_location.start_offset)); //for citizens that are not working
            assert!(transport_area.contains(&agent.transport_location)
                || housing_area.contains(&agent.transport_location)) //for citizens that aren't using public transport
        }
    }

    #[test]
    fn should_increase_hospital_size() {
        let mut grid = define_geography(100);

        grid.increase_hospital_size(120);

        assert_eq!(grid.hospital_area.start_offset, Point::new(70, 0));
        assert_eq!(grid.hospital_area.end_offset, Point::new(120, 120));
    }

    #[test]
    fn grid_should_be_serializable_and_should_not_serialize_skipped_keys() {
        let grid: Grid = define_geography(75);

        let grid_message = serde_json::to_value(&grid).unwrap();

        let message = grid_message.as_object().unwrap();
        let keys = message.keys();
        assert_eq!(keys.len(), 7);
        assert!(message.contains_key("grid_size"));
        assert!(message.contains_key("housing_area"));
        assert!(message.contains_key("work_area"));
        assert!(message.contains_key("transport_area"));
        assert!(message.contains_key("hospital_area"));
        assert!(message.contains_key("houses"));
        assert!(message.contains_key("offices"));
    }

    #[test]
    fn should_resize_hospital() {
        let mut grid = define_geography(100);
        grid.resize_hospital(1000, 0.02, 0.01);

        assert_eq!(grid.hospital_area.start_offset, Point::new(70, 0));
        assert_eq!(grid.hospital_area.end_offset, Point::new(79, 3));
    }

    #[test]
    fn should_not_resize_hospital_if_population_is_too_high() {
        let mut grid = define_geography(100);
        grid.resize_hospital(50000, 0.02, 0.01);

        assert_eq!(grid.hospital_area.start_offset, Point::new(70, 0));
        assert_eq!(grid.hospital_area.end_offset, Point::new(79, 100));
    }
}
