// pub struct WaypointRow {
//     pub id: i32,
//     pub x: i32,
//     pub y: i32,
//     pub z: i32,
//     pub name: String,
//     pub dimension: i32,
//     pub done: Option<bool>,
// }
//
// pub struct DimensionRow {
//     pub id: i32,
//     pub name: String,
// }

use std::fmt::Display;

pub struct Waypoint {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub name: String,
    pub dimension: String,
    pub completed: Option<bool>,
}

impl Display for Waypoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let mut result = format!(
        //     "#{} - {}: {}/{}/{} - {}",
        //     self.id, self.name, self.x, self.y, self.z, self.dimension
        // );

        let mut result = format!(
            "{}: {}/{}/{} - {}",
            self.name, self.x, self.y, self.z, self.dimension
        );

        if let Some(val) = self.completed
            && val
        {
            result = format!("{} (✓)", result,)
        }

        write!(f, "{}", result)
    }
}
