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

use crate::config::StartingInfections;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Counts {
    hour: u32,
    susceptible: u32,
    exposed: u32,
    infected: u32,
    hospitalized: u32,
    recovered: u32,
    deceased: u32,
}

impl Counts {
    #[cfg(test)]
    pub fn new_test(hour: u32, susceptible: u32, exposed: u32, infected: u32, hospitalized: u32, recovered: u32, deceased: u32) -> Counts {
        Counts { hour, susceptible, exposed, infected, hospitalized, recovered, deceased }
    }

    pub fn new(susceptible: u32, exposed: u32, infected: u32) -> Counts {
        Counts { hour: 0, susceptible, exposed, infected, hospitalized: 0, recovered: 0, deceased: 0 }
    }
    pub fn counts_at_start(population: u32, start_infections: &StartingInfections) -> Counts {
        let s = population - start_infections.total();
        let e = start_infections.get_exposed();
        let i = start_infections.total_infected();
        assert_eq!(s + e + i, population);
        Counts::new(s, e, i)
    }

    pub fn susceptible(&self) -> u32 {
        self.susceptible
    }

    pub fn exposed(&self) -> u32 {
        self.exposed
    }

    pub fn infected(&self) -> u32 {
        self.infected
    }

    pub fn hospitalized(&self) -> u32 {
        self.hospitalized
    }

    pub fn recovered(&self) -> u32 {
        self.recovered
    }

    pub fn deceased(&self) -> u32 {
        self.deceased
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }

    pub fn update_susceptible(&mut self, count: i32) {
        if count.is_negative() {
            self.susceptible = self.susceptible.checked_sub(count.unsigned_abs()).expect("Cannot have a negative susceptible count!");
        } else {
            self.susceptible = self.susceptible.checked_add(count.unsigned_abs()).expect("Overflowed the maximum value for susceptible count!");
        }
    }

    pub fn update_exposed(&mut self, count: i32) {
        if count.is_negative() {
            self.exposed = self.exposed.checked_sub(count.unsigned_abs()).expect("Cannot have a negative exposed count!");
        } else {
            self.exposed = self.exposed.checked_add(count.unsigned_abs()).expect("Overflowed the maximum value for exposed count!");
        }
    }

    pub fn update_infected(&mut self, count: i32) {
        if count.is_negative() {
            self.infected = self.infected.checked_sub(count.unsigned_abs()).expect("Cannot have a negative infected count!");
        } else {
            self.infected = self.infected.checked_add(count.unsigned_abs()).expect("Overflowed the maximum value for infected count!");
        }
    }

    pub fn update_recovered(&mut self, count: i32) {
        if count.is_negative() {
            self.recovered = self.recovered.checked_sub(count.unsigned_abs()).expect("Cannot have a negative recovered count!");
        } else {
            self.recovered = self.recovered.checked_add(count.unsigned_abs()).expect("Overflowed the maximum value for recovered count!");
        }
    }

    pub fn update_deceased(&mut self, count: i32) {
        if count.is_negative() {
            self.deceased = self.deceased.checked_sub(count.unsigned_abs()).expect("Cannot have a negative deceased count!");
        } else {
            self.deceased = self.deceased.checked_add(count.unsigned_abs()).expect("Overflowed the maximum value for deceased count!");
        }
    }

    pub fn update_hospitalized(&mut self, count: i32) {
        if count.is_negative() {
            self.hospitalized = self.hospitalized.checked_sub(count.unsigned_abs()).expect("Cannot have a negative hospitilized count!");
        } else {
            self.hospitalized = self.hospitalized.checked_add(count.unsigned_abs()).expect("Overflowed the maximum value for hospitilized count!");
        }
    }

    pub fn increment_hour(&mut self) {
        self.hour += 1;
    }

    pub fn clear(&mut self) {
        self.susceptible = 0;
        self.exposed = 0;
        self.infected = 0;
        self.hospitalized = 0;
        self.recovered = 0;
        self.deceased = 0;
    }

    pub fn total(&self) -> u32 {
        self.susceptible +
            self.exposed +
            self.infected +
            self.hospitalized +
            self.recovered +
            self.deceased
    }

    pub fn log(&self) {
        info!("S: {}, E:{}, I: {}, H: {}, R: {}, D: {}", self.susceptible(), self.exposed(),
              self.infected(), self.hospitalized(), self.recovered(),
              self.deceased())
    }
}


#[cfg(test)]
mod tests {
    use crate::listeners::events::counts::Counts;

    #[test]
    fn should_create_counts() {
        let counts = Counts::new(100, 1, 2);
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 2);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_update_susceptible() {
        let mut counts = Counts::new(100, 1, 2);
        counts.update_susceptible(5);
        assert_eq!(counts.susceptible, 105);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 2);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_update_exposed() {
        let mut counts = Counts::new(100, 1, 0);
        counts.update_exposed(5);
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 6);
        assert_eq!(counts.infected, 0);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_update_infected() {
        let mut counts = Counts::new(100, 1, 0);
        counts.update_infected(5);
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 5);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_update_recovered() {
        let mut counts = Counts::new(100, 1, 0);
        counts.update_recovered(5);
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 0);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 5);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_update_deceased() {
        let mut counts = Counts::new(100, 1, 0);
        counts.update_deceased(5);
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 0);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 5);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_update_quarantined() {
        let mut counts = Counts::new(100, 1, 0);
        counts.update_hospitalized(5);
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 0);
        assert_eq!(counts.hospitalized, 5);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 0);
    }

    #[test]
    fn should_increment_hour() {
        let mut counts = Counts::new(100, 1, 0);
        counts.increment_hour();
        assert_eq!(counts.susceptible, 100);
        assert_eq!(counts.exposed, 1);
        assert_eq!(counts.infected, 0);
        assert_eq!(counts.hospitalized, 0);
        assert_eq!(counts.recovered, 0);
        assert_eq!(counts.deceased, 0);
        assert_eq!(counts.hour, 1);
    }
}
