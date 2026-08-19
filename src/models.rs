use std::fmt::Display;

pub struct Dimension {
    pub id: i32,
    pub name: String,
}

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

impl Waypoint {
    pub fn to_embed_text(&self, show_completed: bool, show_dimension: bool) -> String {
        // name + id
        let mut text = format!("**{}** \\#{} ", self.name, self.id);

        // completed
        if let Some(val) = self.completed
            && val
            && show_completed
        {
            text.push_str("*(Completed)*");
        };

        // coords
        text.push_str(&format!("\n{} / {} / {}", self.x, self.y, self.z));

        // dimension
        if show_dimension {
            text.push_str(&format!("\n*{}*", self.dimension));
        };

        text
    }
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

#[derive(sqlx::FromRow)]
pub struct PendingWaypoint {
    pub id: i32,
    pub action: PendingWaypointAction,
    pub author: String,

    pub waypoint_id: Option<i32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub z: Option<i32>,
    pub name: Option<String>,
    pub dimension: Option<String>,
    pub completed: Option<Option<bool>>,
}

pub enum PendingWaypointAction {
    Add,
    Edit,
    Delete,
}

impl Display for PendingWaypointAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            PendingWaypointAction::Add => "Add",
            PendingWaypointAction::Edit => "Edit",
            PendingWaypointAction::Delete => "Delete",
        };

        write!(f, "{}", val)
    }
}
