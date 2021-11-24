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

use crate::config::Config;
use crate::interventions::hospital::{BuildNewHospital, BuildNewHospitalConfig};
use crate::interventions::lockdown::{LockdownConfig, LockdownIntervention};
use crate::interventions::vaccination::{VaccinateConfig, VaccinateIntervention};
use crate::listeners::events::counts::Counts;

pub mod hospital;
pub mod lockdown;
pub mod vaccination;
pub mod intervention_type;

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
#[serde(rename = "Intervention")]
pub enum InterventionConfig {
    Vaccinate(VaccinateConfig),
    Lockdown(LockdownConfig),
    BuildNewHospital(BuildNewHospitalConfig),
}

pub struct Interventions {
    pub vaccinate: VaccinateIntervention,
    pub lockdown: LockdownIntervention,
    pub build_new_hospital: BuildNewHospital,
}

impl Interventions {
    pub fn init_interventions(config: &Config, rng: &mut impl rand::RngCore) -> Interventions {
        let vaccinations = VaccinateIntervention::init(config);
        let lock_down_details = LockdownIntervention::init(config);
        let hospital_intervention = BuildNewHospital::init(config);
        let essential_workers_population = lock_down_details.get_essential_workers_percentage();

        // TODO Fix this
        /*        for (_, agent) in self.agent_location_map.iter_mut() {
                    agent.assign_essential_worker(essential_workers_population, rng);
                }
        */        Interventions {
            vaccinate: vaccinations,
            lockdown: lock_down_details,
            build_new_hospital: hospital_intervention,
        }
    }

    pub fn process_interventions(interventions: &mut Interventions, counts_at_hr: &Counts,
                                 listeners: &mut Listeners, rng: &mut impl rand::RngCore,
                                 config: &Config) {
        apply_vaccination_intervention(
            &interventions.vaccinate,
            counts_at_hr,
            write_buffer,
            rng,
            listeners,
        );

        if interventions.lockdown.should_apply(&counts_at_hr) {
            interventions.lockdown.apply();
            Epidemiology::lock_city(counts_at_hr.get_hour(), write_buffer);
            listeners.intervention_applied(counts_at_hr.get_hour(), &interventions.lockdown)
        }
        if interventions.lockdown.should_unlock(&counts_at_hr) {
            Epidemiology::unlock_city(counts_at_hr.get_hour(), write_buffer);
            interventions.lockdown.unapply();
            listeners.intervention_applied(counts_at_hr.get_hour(), &interventions.lockdown)
        }

        interventions.build_new_hospital.counts_updated(&counts_at_hr);
        if interventions.build_new_hospital.should_apply(counts_at_hr) {
            info!("Increasing the hospital size");
            grid.increase_hospital_size(config.get_grid_size());
            interventions.build_new_hospital.apply();

            listeners.grid_updated(grid);
            listeners.intervention_applied(counts_at_hr.get_hour(), &interventions.build_new_hospital);
        }
    }


    fn apply_vaccination_intervention(vaccinations: &VaccinateIntervention, counts: &Counts,
                                      write_buffer_reference: &mut AgentLocationMap, rng: &mut impl rand::RngCore,
                                      listeners: &mut Listeners) {
        match vaccinations.get_vaccination_percentage(counts) {
            Some(vac_percent) => {
                info!("Vaccination");
                Epidemiology::vaccinate(*vac_percent, write_buffer_reference, rng);
                listeners.intervention_applied(counts.get_hour(), vaccinations)
            }
            _ => {}
        };
    }

    fn vaccinate(vaccination_percentage: f64, write_buffer_reference: &mut AgentLocationMap, rng: &mut impl rand::RngCore) {
        for (_v, agent) in write_buffer_reference.iter_mut() {
            if agent.state_machine.is_susceptible() && rng.gen_bool(vaccination_percentage) {
                agent.set_vaccination(true);
            }
        }
    }
}