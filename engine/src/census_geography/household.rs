use std::fmt::{Display, Formatter};

use uuid::Uuid;

use load_census_data::table_144_enum_values::AreaClassification;

use crate::census_geography::AreaCode;

impl Display for AreaCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Output Area: {}, Area Type: {:?}, Building ID: {}", self.output_code, self.area_type, self.building_id)
    }
}

pub struct Household {
    /// This is unique to the specific output area - ~250 households
    household_code: AreaCode,
    /// A list of all the ID's of agents who live at his household
    residents: Vec<Uuid>,
}

impl Household {
    pub fn new(household_code: AreaCode) -> Household {
        Household { household_code, residents: Vec::new() }
    }
    pub fn add_citizen(&mut self, citizen_id: Uuid) {
        self.residents.push(citizen_id);
    }
    pub fn household_code(&self) -> &AreaCode {
        &self.household_code
    }
    pub fn residents(&self) -> &Vec<Uuid> {
        &self.residents
    }
}

impl Display for Household {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Household: {}, with {} residents", self.household_code, self.residents.len())
    }
}