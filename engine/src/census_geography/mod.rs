use std::collections::HashMap;
use std::slice::Iter;
use std::time::Instant;

use geo_types::{Coordinate, LineString};
use shapefile::{Polygon, Shape};
use shapefile::dbase::{FieldValue, Record};

use crate::census_geography::parsing_error::{ParsingError, ParsingErrorType};
use crate::constants;

//pub use self::area::Area;
pub use self::point::Point;

mod point;
pub mod output_area;
mod household;
mod parsing_error;

const DEBUG_ITERATION: usize = 5000;

/// Builds the output areas from the given csv file
///
/// Then returns a hashmap, with the key as the output area code, and the corresponding OutputArea struct
///
/// # Arguments
///
/// * `grid_size`:
///
/// returns: HashMap<String, OutputArea, RandomState>
///
/// # Examples
///
/// ```
///
/// ```
pub fn define_geography(grid_size: i32) -> HashMap<String, geo_types::Polygon<f64>> {
    let filename = "census_map_areas/England_wa_2011/england_wa_2011.shp";
    load_areas_from_shape_file(filename).expect("Failed to load output area map")
}

pub fn load_areas_from_shape_file(filename: &str) -> Result<HashMap<String, geo_types::Polygon<f64>>, ParsingError> {
    let mut reader =
        shapefile::Reader::from_path(filename).expect("Shape file for output areas does not exist!");
    let start_time = Instant::now();
    let mut data = HashMap::new();
    info!("Loading map data from file...");
    for (index, shape_record) in reader.iter_shapes_and_records().enumerate() {
        let (shape, record) = shape_record.unwrap();
        if let Shape::Polygon(polygon) = shape {
            assert!(!polygon.rings().is_empty());
            let rings: Vec<Coordinate<f64>>;
            let mut interior_ring;
            if polygon.rings().len() == 1 {
                rings = polygon.rings()[0].points().iter().map(|p| geo_types::Coordinate::from(*p)).collect();
                interior_ring = Vec::new();
            } else {
                interior_ring = polygon.rings().iter().map(|r| LineString::from(r.points().iter().map(|p| geo_types::Coordinate::from(*p)).collect::<Vec<Coordinate<f64>>>())).collect();
                rings = interior_ring.pop().unwrap().0;
            }
            let new_poly = geo_types::Polygon::new(LineString::from(rings), interior_ring);

            // Retrieve the area code:
            let code_record = record.get("code").expect("Missing required field 'code'");
            let code;
            if let FieldValue::Character(option_val) = code_record {
                code = option_val.clone().unwrap_or_else(|| String::from(""));
            } else {
                return Err(ParsingError::new(ParsingErrorType::InvalidDataType(format!("{:?}", code_record.field_type())), Some(format!("Unexpected field value type for area code: {}", code_record))));
            }

            data.insert(code, new_poly);
        } else {
            return Err(ParsingError::new(ParsingErrorType::InvalidDataType(format!("{:?}", shape.shapetype())), Some(format!("Unexpected shape type"))));
        }
        if index % DEBUG_ITERATION == 0 {
            debug!("  At index {} with time {:?}", index, start_time.elapsed());
        }
    }
    info!("Finished loading map data in {:?}", start_time.elapsed());
    Ok(data)
}

#[derive(Clone)]
pub struct AreaCode {
    output_code: String,
    area_type: AreaClassification,
    building_id: Uuid,
}

impl AreaCode {
    pub fn new(output_code: String, area_type: AreaClassification) -> AreaCode {
        AreaCode {
            output_code,
            area_type,
            building_id: Uuid::new_v4(),
        }
    }
}
/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_define_geography() {
        let grid = define_geography(10);
        assert_eq!(grid.housing_area.start_offset, Point::new(0, 0));
        assert_eq!(grid.housing_area.end_offset, Point::new(3, 10));

        assert_eq!(grid.transport_area.start_offset, Point::new(4, 0));
        assert_eq!(grid.transport_area.end_offset, Point::new(4, 10));

        assert_eq!(grid.work_area.start_offset, Point::new(5, 0));
        assert_eq!(grid.work_area.end_offset, Point::new(6, 10));

        assert_eq!(grid.hospital_area.start_offset, Point::new(7, 0));
        assert_eq!(grid.hospital_area.end_offset, Point::new(7, 10));
    }
}
*/