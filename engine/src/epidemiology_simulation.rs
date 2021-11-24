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

use core::borrow::Borrow;
use core::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Local};
use futures::join;
use futures::StreamExt;
use rand::{Rng, thread_rng};

use load_census_data::load_table_from_disk;

use crate::agent::Citizen;
use crate::census_geography::load_areas_from_shape_file;
use crate::census_geography::output_area::OutputArea;
use crate::config::{CensusPopulation, Config, StartingInfections};
use crate::constants::HOSPITAL_STAFF_PERCENTAGE;
use crate::disease::Disease;
use crate::disease_state_machine::State;
use crate::interventions::hospital::BuildNewHospital;
use crate::interventions::Interventions;
use crate::interventions::lockdown::LockdownIntervention;
use crate::interventions::vaccination::VaccinateIntervention;
use crate::listeners::create_listeners;
use crate::listeners::events::counts::Counts;
use crate::listeners::listener::{Listener, Listeners};

pub struct Epidemiology<T: rand::RngCore> {
    pub output_areas: HashMap<String, OutputArea>,
    pub disease: Disease,
    pub sim_id: String,
    pub current_population: u32,
    listeners: Listeners,
    counts_at_hour: Counts,
    rng: T,
    current_hour: u16,
    interventions: Interventions,
}

impl<T: rand::RngCore> Epidemiology<T> {
    /// First build the map - see geography
    /// Then generates a population of the given size
    ///     Each agent, is given a random home and work place that they use throughout the pandemic
    pub fn new(config: &Config, sim_id: String) -> Epidemiology<T> {
        let start = Instant::now();
        let mut rng = thread_rng();

        let disease = config.get_disease();
        let start_infections = config.get_starting_infections();
        let mut total_population = 0;

        let census_data = load_table_from_disk("blagh".to_string()).unwrap();
        let mut output_area_polygon_map = load_areas_from_shape_file("census_map_areas/England_wa_2011/england_wa_2011.shp").expect("Failed to load output area map");

        let mut output_areas: HashMap<String, OutputArea> = HashMap::new();
        for (code, polygon) in output_area_polygon_map.into_iter() {
            // TODO Add failure case
            let census_for_current_area = census_data.get(&code).unwrap();
            total_population += census_for_current_area.population_size as u32;
            output_areas.insert(code.to_string(), OutputArea::new(code.to_string(), polygon, census_for_current_area, &mut rng));
        }

        info!("Initialization completed in {} seconds", start.elapsed().as_secs_f32());
        Epidemiology {
            output_areas,
            disease,
            sim_id,
            current_population: total_population,
            listeners: create_listeners(config, total_population),
            counts_at_hour: Counts::counts_at_start(total_population, &config.get_starting_infections()),
            rng,
            current_hour: 0,
            interventions: Interventions::init_interventions(),
        }
    }
    pub async fn run(&mut self, config: &Config) {
        //listeners.grid_updated(&self.grid);
        let start_time = Instant::now();
        // TODO Figure out what outgoing is
        // Think it's transferring engine?
        let mut outgoing = Vec::new();
        let percent_outgoing = 0.0;

        self.counts_at_hr.log();
        for simulation_hour in 1..config.get_hours() {
            debug!("Hour: {}, Total Agents: {}, Counts {:?}",simulation_hour, self.current_population,self.counts_at_hr);
            self.counts_at_hr.increment_hour();

            self.simulate(simulation_hour, percent_outgoing, config.enable_citizen_state_messages());
            self.interventions.process_interventions();
            //listeners.counts_updated(*counts_at_hr);
            Epidemiology::process_interventions(&mut self.interventions, &self.counts_at_hr, &mut self.listeners,
                                                &mut self.rng, config, &mut self.grid);

            if Epidemiology::stop_simulation(&mut interventions.lockdown, *counts_at_hr) {
                info!("Finished early, with stats: {:?}",counts_at_hr);
                break;
            }

            if simulation_hour % 100 == 0 {
                info!("Throughput: {} iterations/sec; simulation hour {} of {}",
                      simulation_hour as f32 / start_time.elapsed().as_secs_f32(),
                      simulation_hour, config.get_hours());
                self.counts_at_hr.log();
            }
        }
        let elapsed_time = start_time.elapsed().as_secs_f32();
        info!("Number of iterations: {}, Total Time taken {} seconds", self.counts_at_hr.get_hour(), elapsed_time);
        info!("Iterations/sec: {}", self.counts_at_hr.get_hour() as f32 / elapsed_time);
        //listeners.simulation_ended();
    }

    fn simulate(&mut self, disease: &Disease, percent_outgoing: f64,
                outgoing: &mut Vec<(Point, Traveller)>, publish_citizen_state: bool) {
        write_buffer.clear();
        csv_record.clear();
        for (cell, agent) in read_buffer.iter() {
            let mut current_agent = *agent;
            let infection_status = current_agent.state_machine.is_infected();
            let point = current_agent.perform_operation(*cell, simulation_hour, &grid, read_buffer, rng, disease);
            Epidemiology::update_counts(csv_record, &current_agent);

            if infection_status == false && current_agent.state_machine.is_infected() == true {
                listeners.citizen_got_infected(&cell);
            }

            let agent_option = write_buffer.get(&point);
            let new_location = match agent_option {
                Some(mut _agent) => cell, //occupied
                _ => &point
            };

            if simulation_hour % 24 == 0 && current_agent.can_move()
                && rng.gen_bool(percent_outgoing) {
                let traveller = Traveller::from(&current_agent);
                outgoing.push((*new_location, traveller));
            }

            write_buffer.insert(*new_location, current_agent);
            if publish_citizen_state {
                listeners.citizen_state_updated(simulation_hour, &current_agent, new_location);
            }
        }
        assert_eq!(csv_record.total(), write_buffer.current_population());
    }
    fn stop_simulation(lock_down_details: &mut LockdownIntervention, row: Counts) -> bool {
        row.get_exposed() == 0 && row.get_infected() == 0 && row.get_hospitalized() == 0
    }
    fn update_counts(counts_at_hr: &mut Counts, citizen: &Citizen) {
        match citizen.state_machine.state {
            State::Susceptible { .. } => { counts_at_hr.update_susceptible(1) }
            State::Exposed { .. } => { counts_at_hr.update_exposed(1) }
            State::Infected { .. } => {
                if citizen.is_hospitalized() {
                    counts_at_hr.update_hospitalized(1);
                } else {
                    counts_at_hr.update_infected(1)
                }
            }
            State::Recovered { .. } => { counts_at_hr.update_recovered(1) }
            State::Deceased { .. } => { counts_at_hr.update_deceased(1) }
        }
    }

    fn lock_city(hr: i32, write_buffer_reference: &mut AgentLocationMap) {
        info!("Locking the city. Hour: {}", hr);
        for (_v, agent) in write_buffer_reference.iter_mut() {
            if !agent.is_essential_worker() {
                agent.set_isolation(true);
            }
        }
    }

    fn unlock_city(hr: i32, write_buffer_reference: &mut AgentLocationMap) {
        info!("Unlocking city. Hour: {}", hr);
        for (_v, agent) in write_buffer_reference.iter_mut() {
            if agent.is_isolated() {
                agent.set_isolation(false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::GeographyParameters;
    use crate::interventions::InterventionConfig;
    use crate::interventions::vaccination::VaccinateConfig;

    use super::*;

    #[test]
    fn should_init() {
        let pop = AutoPopulation {
            number_of_agents: 10,
            public_transport_percentage: 1.0,
            working_percentage: 1.0,
        };
        let disease = Disease::new(0, 0, 0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0);
        let vac = VaccinateConfig {
            at_hour: 5000,
            percent: 0.2,
        };
        // TODO Fix this
        /*        let geography_parameters = GeographyParameters::new(100, 0.003);
                let config = Config::new(Population::Auto(pop), disease, geography_parameters, vec![], 100, vec![InterventionConfig::Vaccinate(vac)], None);
                let epidemiology: Epidemiology = Epidemiology::new(&config, "id".to_string());
                let expected_housing_area = Area::new(Point::new(0, 0), Point::new(39, 100));
                assert_eq!(epidemiology.grid.housing_area, expected_housing_area);

                let expected_transport_area = Area::new(Point::new(40, 0), Point::new(49, 100));
                assert_eq!(epidemiology.grid.transport_area, expected_transport_area);

                let expected_work_area = Area::new(Point::new(50, 0), Point::new(69, 100));
                assert_eq!(epidemiology.grid.work_area, expected_work_area);

                let expected_hospital_area = Area::new(Point::new(70, 0), Point::new(79, 0));
                assert_eq!(epidemiology.grid.hospital_area, expected_hospital_area);

                assert_eq!(epidemiology.agent_location_map.current_population(), 10);*/
    }
}
