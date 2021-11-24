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
use std::time::SystemTime;

use chrono::{DateTime, Local};

use crate::config::Config;
use crate::listeners::listener::{Listener, Listeners};

pub mod events;
pub mod listener;

fn output_file_format(config: &Config) -> String {
    let now: DateTime<Local> = SystemTime::now().into();
    let mut output_file_prefix = config.get_output_file().unwrap_or("simulation".to_string());
    format!("{}_{}", output_file_prefix, now.format("%Y-%m-%dT%H:%M:%S"))
}

pub(crate) fn create_listeners(config: &Config, population: u32) -> Listeners {
    /*let output_file_format = output_file_format(config);
    let counts_file_name = format!("{}.csv", output_file_format);

    let csv_listener = CsvListener::new(counts_file_name);
    // TODO Fix this
    let population = 0;
    //let population = self.agent_location_map.current_population();

    let hotspot_tracker = Hotspot::new();
    let intervention_reporter = InterventionReporter::new(format!("{}_interventions.json", output_file_format));
    let mut listeners_vec: Vec<Box<dyn Listener>> = vec![Box::new(csv_listener),
                                                         Box::new(hotspot_tracker),
                                                         Box::new(intervention_reporter)];

    Listeners::from(listeners_vec)*/
    Listeners::from(Vec::new())
}
/*pub mod csv_service;
pub mod disease_tracker;

pub mod travel_counter;
pub mod intervention_reporter;
*/