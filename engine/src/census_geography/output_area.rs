use std::collections::HashMap;

use enum_map::EnumMap;

use load_census_data::population_and_density_per_output_area::PopulationRecord as PopRecord;

use crate::agent::{Citizen, PopulationRecord};
use crate::allocation_map;
use crate::census_geography::household::Household;
use crate::geography::Grid;

pub struct OutputArea {
    pub code: String,
    pub agents: Vec<Citizen>,
    pub area_size: f32,
    pub density: f32,
    pub population_counts: EnumMap<AreaClassification, Vec<Household>>,
    pub polygon: geo_types::Polygon<f64>,
    pub agent_location_map: String,
    //allocation_map::AgentLocationMap,
    pub grid: String,//Grid,
}

impl OutputArea {
    pub fn new(code: String, polygon: geo_types::Polygon<f64>, census_data: &PopRecord) -> OutputArea {
        // TODO Fix this
        let household_number = 250;
        let household_size = &census_data.population_size / household_number;
        let mut generated_population = 0;
        for household in 0..household_number {
            generated_population += household_size;
        }
        OutputArea {
            code,
            agents: vec![],
            area_size: 0.0,
            density: 0.0,
            population_counts: census_data.population_counts,
            polygon: polygon,
            agent_location_map: String::new(),
            grid: String::new(),
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