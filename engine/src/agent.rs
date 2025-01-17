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
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use serde::{de, Deserialize, Deserializer};
use serde::de::Unexpected;
use uuid::Uuid;

use crate::allocation_map::AgentLocationMap;
use crate::config::StartingInfections;
use crate::constants;
use crate::disease::Disease;
use crate::disease_state_machine::DiseaseStateMachine;
use crate::geography::{Area, Grid, Point};

use crate::travel_plan::Traveller;

#[derive(Deserialize)]
pub struct PopulationRecord {
    //TODO move to a better place
    pub ind: i32,
    pub age: String,
    #[serde(deserialize_with = "bool_from_string")]
    pub working: bool,
    #[serde(deserialize_with = "bool_from_string")]
    pub pub_transport: bool,
}

/// Deserialize bool from String with custom value mapping
fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "True" => Ok(true),
        "False" => Ok(false),
        other => Err(de::Error::invalid_value(
            Unexpected::Str(other),
            &"True or False",
        )),
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum WorkStatus {
    Normal,
    Essential,
    HospitalStaff { work_start_at: i32 },
    NA,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Citizen {
    pub id: Uuid,
    immunity: i32,
    pub home_location: Area,
    pub work_location: Area,
    vaccinated: bool,
    pub uses_public_transport: bool,
    working: bool,
    hospitalized: bool,
    pub transport_location: Point,
    pub state_machine: DiseaseStateMachine,
    isolated: bool,
    current_area: Area,
    work_status: WorkStatus,
    work_quarantined: bool,
}

impl Citizen {
    pub fn new(home_location: Area, work_location: Area, transport_location: Point,
               uses_public_transport: bool, working: bool, work_status: WorkStatus, rng: &mut impl rand::RngCore) -> Citizen {
        Citizen::new_with_id(Uuid::new_v4(), home_location, work_location, transport_location, uses_public_transport,
                             working, work_status, rng)
    }

    pub fn new_with_id(id: Uuid, home_location: Area, work_location: Area, transport_location: Point,
                       uses_public_transport: bool, working: bool, work_status: WorkStatus, rng: &mut impl rand::RngCore) -> Citizen {
        let disease_randomness_factor = Citizen::generate_disease_randomness_factor(rng);

        Citizen {
            id,
            immunity: disease_randomness_factor,
            home_location,
            work_location,
            transport_location,
            vaccinated: false,
            uses_public_transport,
            working,
            hospitalized: false,
            state_machine: DiseaseStateMachine::new(),
            isolated: false,
            current_area: home_location,
            work_status,
            work_quarantined: false,
        }
    }

    pub fn from_traveller(traveller: &Traveller, home_location: Area, work_location: Area,
                          transport_location: Point, current_area: Area) -> Citizen {
        Citizen {
            id: traveller.id,
            immunity: traveller.immunity,
            home_location,
            work_location,
            vaccinated: traveller.vaccinated,
            uses_public_transport: traveller.uses_public_transport,
            working: false,
            hospitalized: false,
            transport_location,
            state_machine: traveller.state_machine,
            isolated: false,
            current_area,
            work_status: WorkStatus::NA {},
            work_quarantined: false,
        }
    }

    pub fn from_record(record: PopulationRecord, home_location: Area, work_location: Area,
                       transport_location: Point, rng: &mut impl rand::RngCore) -> Citizen {
        let disease_randomness_factor = Citizen::generate_disease_randomness_factor(rng);
        let work_status = Citizen::derive_work_status(record.working, rng);

        Citizen {
            id: Uuid::new_v4(),
            immunity: disease_randomness_factor,
            home_location,
            work_location,
            transport_location,
            vaccinated: false,
            uses_public_transport: record.pub_transport,
            working: record.working,
            hospitalized: false,
            state_machine: DiseaseStateMachine::new(),
            isolated: false,
            current_area: home_location,
            work_status,
            work_quarantined: false,
        }
    }

    pub fn get_infection_transmission_rate(&self, disease: &Disease) -> f64 {
        disease.get_current_transmission_rate(self.state_machine.get_infection_day() + self.immunity)
    }

    pub fn set_vaccination(&mut self, vaccinated: bool) {
        self.vaccinated = vaccinated;
    }

    pub fn can_move(&self) -> bool {
        if self.state_machine.is_symptomatic() || self.hospitalized || self.state_machine.is_deceased() || self.isolated {
            return false;
        }
        true
    }

    pub fn set_isolation(&mut self, state: bool) {
        self.isolated = state;
    }

    pub fn is_isolated(&self) -> bool {
        self.isolated
    }

    pub fn is_working(&self) -> bool {
        self.working
    }

    pub fn get_immunity(&self) -> i32 {
        self.immunity
    }

    pub fn is_vaccinated(&self) -> bool {
        self.vaccinated
    }

    fn generate_disease_randomness_factor(rng: &mut impl rand::RngCore) -> i32 {
        let option = constants::IMMUNITY_RANGE.choose(rng);
        *option.unwrap()
    }

    pub fn perform_operation(&mut self, cell: Point, simulation_hour: i32, grid: &Grid, map: &AgentLocationMap,
                             rng: &mut impl rand::RngCore, disease: &Disease) -> Point {
        self.routine(cell, simulation_hour, grid, map, rng, disease)
    }

    fn routine(&mut self, cell: Point, simulation_hour: i32, grid: &Grid, map: &AgentLocationMap,
               rng: &mut impl rand::RngCore, disease: &Disease) -> Point {
        let mut new_cell = cell;

        let current_hour = simulation_hour % constants::NUMBER_OF_HOURS;
        match current_hour {
            constants::ROUTINE_START_TIME => {
                self.update_infection_day();
                new_cell = self.hospitalize(cell, &grid.hospital_area, map, rng,disease);
            }
            constants::SLEEP_START_TIME..=constants::SLEEP_END_TIME => {
                if !self.is_hospital_staff() {
                    self.current_area = self.home_location;
                }
            }
            constants::ROUTINE_END_TIME => {
                new_cell = self.deceased(map, cell, rng, disease)
            }
            _ => {
                new_cell = self.perform_movements(cell, current_hour, simulation_hour, grid, map, rng, disease);
            }
        }
        new_cell
    }

    fn is_hospital_staff(&self) -> bool {
        return match self.work_status {
            WorkStatus::HospitalStaff { .. } => true,
            _ => false
        };
    }

    pub fn is_essential_worker(&self) -> bool {
        return match self.work_status {
            WorkStatus::Essential {} => true,
            _ => false
        };
    }

    fn perform_movements(&mut self, cell: Point, hour_of_day: i32, simulation_hr: i32, grid: &Grid,
                         map: &AgentLocationMap, rng: &mut impl rand::RngCore, disease: &Disease) -> Point {
        let mut new_cell = cell;
        match self.work_status {
            WorkStatus::Normal {} | WorkStatus::Essential {} => {
                match hour_of_day {
                    constants::ROUTINE_TRAVEL_START_TIME | constants::ROUTINE_TRAVEL_END_TIME => {
                        if self.uses_public_transport {
                            new_cell = self.goto_area(grid.transport_area, map, cell, rng);
                            self.current_area = grid.transport_area;
                        } else {
                            new_cell = self.move_agent_from(map, cell, rng);
                        }
                    }
                    constants::ROUTINE_WORK_TIME => {
                        new_cell = self.goto_area(self.work_location, map, cell, rng);
                        self.current_area = self.work_location;
                    }
                    constants::ROUTINE_WORK_END_TIME => {
                        new_cell = self.goto_area(self.home_location, map, cell, rng);
                        self.current_area = self.home_location;
                    }
                    _ => {
                        new_cell = self.move_agent_from(map, cell, rng);
                    }
                }
                self.update_infection_dynamics(new_cell, &map, simulation_hr, rng, &disease);
            }

            WorkStatus::HospitalStaff { work_start_at } => {
                if simulation_hr - work_start_at == (constants::HOURS_IN_A_DAY * constants::QUARANTINE_DAYS) {
                    self.work_quarantined = true;
                    return new_cell;
                }

                if simulation_hr - work_start_at == (constants::HOURS_IN_A_DAY * constants::QUARANTINE_DAYS * 2) {
                    new_cell = self.goto_area(self.home_location, map, cell, rng);
                    self.current_area = self.home_location;
                    self.work_status = WorkStatus::HospitalStaff { work_start_at: (simulation_hr + constants::HOURS_IN_A_DAY * constants::QUARANTINE_DAYS) };
                    return new_cell;
                }

                match hour_of_day {
                    constants::ROUTINE_WORK_TIME => {
                        if self.current_area != grid.hospital_area && work_start_at <= simulation_hr {
                            new_cell = self.goto_area(grid.hospital_area, map, cell, rng);
                            self.current_area = grid.hospital_area;
                            self.work_status = WorkStatus::HospitalStaff { work_start_at: simulation_hr };
                        }
                        self.work_quarantined = false;
                    }
                    constants::ROUTINE_WORK_END_TIME => {
                        self.work_quarantined = true;
                    }
                    _ => {
                        if !self.work_quarantined && self.can_move() {
                            new_cell = self.move_agent_from(map, cell, rng);
                        }
                    }
                }
                self.update_infection_dynamics(new_cell, &map, simulation_hr, rng, &disease);
            }

            WorkStatus::NA {} => {
                match hour_of_day {
                    constants::ROUTINE_WORK_TIME => {
                        new_cell = self.goto_area(grid.housing_area, map, cell, rng);
                        self.current_area = grid.housing_area;
                    }
                    constants::NON_WORKING_TRAVEL_END_TIME => {
                        new_cell = self.goto_area(self.home_location, map, cell, rng);
                        self.current_area = self.home_location;
                    }

                    _ => {
                        new_cell = self.move_agent_from(map, cell, rng);
                    }
                }
                self.update_infection_dynamics(new_cell, &map, simulation_hr, rng, &disease);
            }
        }
        new_cell
    }

    fn update_infection_dynamics(&mut self, cell: Point, map: &AgentLocationMap,
                                 sim_hr: i32, rng: &mut impl rand::RngCore, disease: &Disease) {
        self.update_exposure(cell, map, sim_hr, rng, disease);
        self.update_infection(sim_hr, rng, &disease);
        self.update_infection_severity(sim_hr, rng, disease);
    }

    fn update_infection_day(&mut self) {
        if self.state_machine.is_infected() {
            self.state_machine.increment_infection_day();
        }
    }

    fn hospitalize(&mut self, cell: Point, hospital: &Area, map: &AgentLocationMap, rng: &mut impl rand::RngCore,
                   disease: &Disease) -> Point {
        let mut new_cell = cell;
        if self.state_machine.is_infected() && !self.hospitalized {
            let to_be_hospitalized = self.state_machine.hospitalize(disease, self.immunity);
            if to_be_hospitalized {
                let (is_hospitalized, new_location) = AgentLocationMap::goto_hospital(map, hospital, cell, self, rng);
                new_cell = new_location;
                if is_hospitalized {
                    self.hospitalized = true;
                }
            }
        }
        new_cell
    }

    fn update_infection_severity(&mut self, sim_hr: i32, rng: &mut impl rand::RngCore, disease: &Disease) {
        if self.state_machine.is_pre_symptomatic() {
            self.state_machine.change_infection_severity(sim_hr, rng, disease);
        }
    }

    fn update_infection(&mut self, sim_hr: i32, rng: &mut impl rand::RngCore, disease: &Disease) {
        if self.state_machine.is_exposed() {
            self.state_machine.infect(rng, sim_hr, &disease);
        }
    }

    fn update_exposure(&mut self, cell: Point, map: &AgentLocationMap, sim_hr: i32, rng: &mut impl rand::RngCore,
                       disease: &Disease) {
        if self.state_machine.is_susceptible() && !self.work_quarantined && !self.vaccinated {
            let neighbours = self.current_area.get_neighbors_of(cell);

            let neighbor_that_spreads_infection = neighbours
                .filter(|p| map.is_point_in_grid(p))
                .filter_map(|cell| { map.get_agent_for(&cell) })
                .filter(|agent| agent.state_machine.is_infected() && !agent.hospitalized)
                .find(|neighbor| rng.gen_bool(neighbor.get_infection_transmission_rate(disease)));

            if neighbor_that_spreads_infection.is_some() {
                self.state_machine.expose(sim_hr);
            }
        }
    }

    fn goto_area(&mut self, target_area: Area, map: &AgentLocationMap, cell: Point, rng: &mut impl rand::RngCore) -> Point {
        //TODO: Refactor - Jayanta
        // If agent is working and current_area is work, target area is home and symptomatic then allow movement
        let mut override_movement = false;

        match self.work_status {
            WorkStatus::Normal {} | WorkStatus::Essential {} => {
                if self.work_location.contains(&cell) && target_area == self.home_location && (self.state_machine.is_mild_symptomatic() || self.state_machine.is_infected_severe()) {
                    override_movement = true;
                }
            }
            _ => {}
        }
        if !self.can_move() && !override_movement {
            return cell;
        }
        if self.working {
            let mut new_cell: Point = target_area.get_random_point(rng);
            if !map.is_cell_vacant(&new_cell) {
                new_cell = cell;
            }

            return map.move_agent(cell, new_cell);
        }
        self.move_agent_from(map, cell, rng)
    }

    fn deceased(&mut self, map: &AgentLocationMap, cell: Point, rng: &mut impl rand::RngCore,
                disease: &Disease) -> Point {
        let mut new_cell = cell;
        if self.state_machine.is_infected() {
            let result = self.state_machine.decease(rng, disease);
            if result.1 == 1 {
                new_cell = map.move_agent(cell, self.home_location.get_random_point(rng));
            }
            if result != (0, 0) {
                if self.hospitalized {
                    self.hospitalized = false;
                }
            }
        }
        new_cell
    }

    fn move_agent_from(&mut self, map: &AgentLocationMap, cell: Point, rng: &mut impl rand::RngCore) -> Point {
        if !self.can_move() {
            return cell;
        }
        let mut current_location = cell;
        if !self.current_area.contains(&cell) {
            current_location = self.current_area.get_random_point(rng);
        }

        let new_cell = self.current_area.get_neighbors_of(current_location)
            .filter(|p| map.is_point_in_grid(p))
            .filter(|p| map.is_cell_vacant(p))
            .choose(rng)
            .unwrap_or(cell);
        map.move_agent(cell, new_cell)
    }

    pub fn assign_essential_worker(&mut self, essential_workers_percentage: f64, rng: &mut impl rand::RngCore) {
        match self.work_status {
            WorkStatus::Normal {} => {
                if rng.gen_bool(essential_workers_percentage) {
                    self.work_status = WorkStatus::Essential {};
                }
            }
            _ => {}
        }
    }

    fn derive_work_status(is_working: bool, rng: &mut impl rand::RngCore) -> WorkStatus {
        if is_working {
            if rng.gen_bool(constants::HOSPITAL_STAFF_PERCENTAGE) {
                return WorkStatus::HospitalStaff { work_start_at: constants::ROUTINE_WORK_TIME };
            }
            return WorkStatus::Normal {};
        }
        return WorkStatus::NA {};
    }

    pub fn is_hospitalized(&self) -> bool {
        self.hospitalized
    }

    #[cfg(test)]
    pub fn is_exposed(&self) -> bool {
        self.state_machine.is_exposed()
    }

    #[cfg(test)]
    pub fn is_mild_asymptomatic(&self) -> bool {
        self.state_machine.is_mild_asymptomatic()
    }

    #[cfg(test)]
    pub fn is_mild_symptomatic(&self) -> bool {
        self.state_machine.is_mild_symptomatic()
    }

    #[cfg(test)]
    pub fn is_infected_severe(&self) -> bool {
        self.state_machine.is_infected_severe()
    }
}

/// Builds all the citizens - and related info
/// Allocates them evenly to homes and work places (NOTE Groups of people will share the same work/home places) - why?
/// Assigns them to a transport square,
pub fn citizen_factory(number_of_agents: i32, home_locations: &Vec<Area>, work_locations: &Vec<Area>, public_transport_locations: &Vec<Point>,
                       percentage_public_transport: f64, working_percentage: f64, rng: &mut impl rand::RngCore,
                       starting_infections: &StartingInfections) -> Vec<Citizen> {
    let mut agent_list = Vec::with_capacity(home_locations.len());
    for i in 0..number_of_agents as usize {
        let is_a_working_citizen = rng.gen_bool(working_percentage);

        let total_home_locations = home_locations.len();
        let total_work_locations = work_locations.len();
        // TODO Change this, to randomly distribute
        home_locations.choose(rng);
        let home_location = home_locations[(i % total_home_locations)];
        let work_location = work_locations[(i % total_work_locations)];

        let uses_public_transport = rng.gen_bool(percentage_public_transport)
            && is_a_working_citizen
            && i < public_transport_locations.len();
        //TODO: Check the logic - Jayanta
        let public_transport_location: Point = if uses_public_transport { public_transport_locations[i] } else {
            home_location.get_random_point(rng)
        };

        let work_location = if is_a_working_citizen { work_location } else {
            home_location
        };
        let work_status = Citizen::derive_work_status(is_a_working_citizen, rng);

        let agent = Citizen::new(home_location, work_location, public_transport_location,
                                 uses_public_transport, is_a_working_citizen, work_status, rng);

        agent_list.push(agent);
    }

    set_starting_infections(&mut agent_list, starting_infections, rng);

    agent_list
}

pub fn set_starting_infections(agent_list: &mut Vec<Citizen>, start_infections: &StartingInfections,
                               rng: &mut impl rand::RngCore) {
    if start_infections.total() as usize > agent_list.len() {
        panic!("There are {} people set to infect, but only {} agents available",
               start_infections.total(), agent_list.len())
    }
    if start_infections.total() == 0 {
        warn!("Simulation configured to start without any infected agents");
    }
    let mut to_infect = agent_list.iter_mut().choose_multiple(rng, start_infections.total() as usize);
    let mut citizens = to_infect.iter_mut();

    for _i in 0..start_infections.get_exposed() {
        citizens.next().unwrap().state_machine.expose(0);
    }
    for _i in 0..start_infections.get_infected_mild_asymptomatic() {
        citizens.next().unwrap().state_machine.set_mild_asymptomatic()
    }
    for _i in 0..start_infections.get_infected_mild_symptomatic() {
        citizens.next().unwrap().state_machine.set_mild_symptomatic()
    }
    for _i in 0..start_infections.get_infected_severe() {
        citizens.next().unwrap().state_machine.set_severe_infected()
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use super::*;

    fn before_each() -> Vec<Citizen> {
        let mut rng = thread_rng();
        let home_locations = vec![Area::new(Point::new(0, 0), Point::new(2, 2)), Area::new(Point::new(3, 0), Point::new(4, 2))];

        let work_locations = vec![Area::new(Point::new(5, 0), Point::new(6, 2)), Area::new(Point::new(7, 0), Point::new(8, 2))];

        let public_transport_location = vec![Point::new(5, 0), Point::new(5, 1), Point::new(5, 2), Point::new(5, 3)];
        let start_infections = StartingInfections::new(0, 0, 0, 1);
        citizen_factory(4, &home_locations, &work_locations, &public_transport_location, 0.5, 0.5,
                        &mut rng, &start_infections)
    }

    #[test]
    fn generate_citizen() {
        let citizen_list = before_each();
        let expected_home_locations = vec![Area::new(Point::new(0, 0), Point::new(2, 2)), Area::new(Point::new(3, 0), Point::new(4, 2))];

        assert_eq!(citizen_list.len(), 4);
        assert_eq!(citizen_list.iter().filter(|c| c.is_exposed()).count(), 1);

        for citizen in &citizen_list {
            assert!(expected_home_locations.contains(&citizen.home_location));
        }
    }

    #[test]
    fn should_set_starting_infections() {
        let home_location = Area::new(Point::new(0, 0), Point::new(10, 10));
        let work_location = Area::new(Point::new(11, 0), Point::new(20, 20));
        let mut citizens = Vec::new();
        let mut rng = thread_rng();
        for _i in 0..20 {
            let citizen = Citizen::new(home_location, work_location, Point::new(2, 2), false,
                                       true, WorkStatus::Normal, &mut rng);
            citizens.push(citizen);
        }

        let start_infections = StartingInfections::new(2, 3, 4, 5);

        set_starting_infections(&mut citizens, &start_infections, &mut rng);

        let actual_exposed = citizens.iter().filter(|citizen| citizen.is_exposed()).count();
        let actual_mild_asymp = citizens.iter().filter(|citizen| citizen.is_mild_asymptomatic()).count();
        let actual_mild_symp = citizens.iter().filter(|citizen| citizen.is_mild_symptomatic()).count();
        let actual_severe = citizens.iter().filter(|citizen| citizen.is_infected_severe()).count();

        assert_eq!(2, actual_mild_asymp);
        assert_eq!(3, actual_mild_symp);
        assert_eq!(4, actual_severe);
        assert_eq!(5, actual_exposed);
    }
}
