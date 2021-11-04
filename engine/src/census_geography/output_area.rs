use crate::census_geography::household::Household;

pub struct OutputArea {
    pub code: String,
    pub total_population: u16,
    pub agents: Vec<Agent>,
    pub area_size: f32,
    pub density: f32,
    pub population_counts: EnumMap<AreaClassification, Vec<Household>>,

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