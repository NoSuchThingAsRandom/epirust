#[macro_use]
extern crate enum_map;
#[macro_use]
extern crate serde_json;
mod nomis_download;
pub mod parsing_error;
pub mod population_and_density_per_output_area;
pub mod table_144_enum_values;

use std::collections::HashMap;
use log::{debug, error, info, warn};
use crate::parsing_error::CensusError;
use crate::population_and_density_per_output_area::PopulationRecord;

pub fn load_table_from_disk(table_name: String) ->Result<HashMap<String,PopulationRecord>,CensusError>{
    info!("Loading census table: '{}'",table_name);
    let reader=csv::Reader::from_path("data/download/PopulationAndDensityPerEnglandOutputArea(144)-temp-Records.csv").unwrap();
    nomis_download::DataFetcher::parse_table(reader)

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
