use enum_map::EnumMap;

use load_census_data::population_and_density_per_output_area::PopulationRecord as PopRecord;
use load_census_data::table_144_enum_values::PersonType;

use crate::agent::{Citizen, WorkStatus};
use crate::census_geography::household::{AreaCode, Household};

pub struct OutputArea {
    pub code: String,
    pub citizens: Vec<Citizen>,
    pub area_size: f32,
    pub density: f32,
    pub households: EnumMap<AreaClassification, Vec<Household>>,
    pub polygon: geo_types::Polygon<f64>,
}

impl OutputArea {
    pub fn new(code: String, polygon: geo_types::Polygon<f64>, census_data: &PopRecord, rng: &mut impl rand::RngCore) -> OutputArea {
        // TODO Fix this
        let mut household_classification = EnumMap::default();
        let mut citizens = Vec::with_capacity(census_data.population_size as usize);
        for (area, pop_count) in census_data.population_counts.iter() {
            // TODO Currently assigning 4 people per household
            // Should use census data instead
            let household_size = 4;
            let household_number = pop_count[PersonType::All] / household_size;
            let mut generated_population = 0;
            let mut households = Vec::with_capacity(household_number as usize);
            for _ in 0..household_number {
                let area_code = AreaCode::new(code.to_string(), area);
                let mut household = Household::new(area_code);
                for _ in 0..household_size {
                    // TODO Add workplaces to citizens
                    let mut citizen = Citizen::new(area_code.clone(), area_code.clone(), WorkStatus::NA, rng);
                    household.add_citizen(citizen.id);
                    citizens.push(citizen);
                    generated_population += 1;
                }
                households.push(household);
                if generated_population >= pop_count[PersonType::All] {
                    break;
                }
            }
            household_classification[area] = households;
        }
        OutputArea {
            code,
            citizens,
            area_size: census_data.area_size,
            density: census_data.density,
            households: household_classification,
            polygon,
        }
    }
}

#[derive(PartialEq, Deserialize, Debug, Enum)]
pub enum AreaClassification {
    #[serde(alias = "Total")]
    Total,
    #[serde(alias = "Urban (total)")]
    UrbanTotal,
    #[serde(alias = "Urban major conurbation")]
    UrbanMajorConurbation,
    #[serde(alias = "Urban minor conurbation")]
    UrbanMinorConurbation,
    #[serde(alias = "Urban city and town")]
    UrbanCity,
    #[serde(alias = "Urban city and town in a sparse setting")]
    UrbanSparseTownCity,
    #[serde(alias = "Rural (total)")]
    RuralTotal,
    #[serde(alias = "Rural town and fringe")]
    RuralTown,
    #[serde(alias = "Rural town and fringe in a sparse setting")]
    RuralSparseTown,
    #[serde(alias = "Rural village")]
    RuralVillage,
    #[serde(alias = "Rural village in a sparse setting")]
    RuralSparseVillage,
    #[serde(alias = "Rural hamlet and isolated dwellings")]
    RuralHamlet,
    #[serde(alias = "Rural hamlet and isolated dwellings in a sparse setting")]
    RuralSparseHamlet,
}