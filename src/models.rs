use std::fmt::Display;

#[derive(sqlx::FromRow)]
pub struct Waypoint {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub name: String,
    pub dimension: String,
    pub completed: Option<bool>,
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum DimensionEnum {
    #[name = "overworld"]
    Overworld,
    #[name = "nether"]
    Nether,
    #[name = "end"]
    End,
}

impl Display for DimensionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            DimensionEnum::Overworld => "The Overworld",
            DimensionEnum::Nether => "The Nether",
            DimensionEnum::End => "The End",
        };

        write!(f, "{}", val)
    }
}
