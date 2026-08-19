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
        let mut text = format!("\n**{}** \\#{} ", self.name, self.id);

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

impl DimensionEnum {
    // WARN: this might not reflect the database, thus can cause errors
    pub fn id(&self) -> i32 {
        match self {
            DimensionEnum::Overworld => 1,
            DimensionEnum::Nether => 2,
            DimensionEnum::End => 3,
        }
    }
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
    pub author_name: String,
    pub author_id: String,

    pub waypoint_id: Option<i32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub z: Option<i32>,
    pub name: Option<String>,
    pub dimension_id: Option<i32>,
    pub dimension_name: Option<String>,
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

impl TryFrom<String> for PendingWaypointAction {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lower = value.to_lowercase();

        match lower.as_str() {
            "add" => Ok(Self::Add),
            "edit" => Ok(Self::Edit),
            "delete" => Ok(Self::Delete),
            _ => Err(anyhow::Error::msg(format!(
                "Unknown value for PendingWaypointAction: {}",
                value
            ))),
        }
    }
}

impl PendingWaypoint {
    pub fn to_embed_text(&self) -> String {
        // action + id
        let mut text = format!("\nPending **{}** \\#{}:", self.action, self.id);

        if let Some(id) = self.waypoint_id {
            text.push_str(&format!("\nID: #{}", id));
        }

        if let Some(name) = &self.name {
            text.push_str(&format!("\nName: **{}**", name));

            if let Some(x) = &self.x {
                text.push_str(&format!("\nX coordinate: {}", x));
            }

            if let Some(y) = &self.y {
                text.push_str(&format!("\nY coordinate: {}", y));
            }

            if let Some(z) = &self.z {
                text.push_str(&format!("\nZ coordinate: {}", z));
            }

            let dim = match (&self.dimension_name, self.dimension_id) {
                (None, None) => "".to_string(),
                (None, Some(id)) => format!("\nDimension #{}", id),
                (Some(name), None) => format!("\nDimension: {}", name),
                (Some(name), Some(id)) => format!("\nDimension: {} (#{})", name, id),
            };
            text.push_str(&dim);

            if let Some(completed) = self.completed
                && let Some(val) = completed
            {
                let comp = format!("Completed: {}", val);
                text.push_str(&comp);
            };
        }

        text.push_str(&format!("\nRequested by: {}", self.author_id));

        text
    }
}

#[derive(Debug)]
pub struct PendingWaypointRow {
    pub id: i32,
    pub action: String,
    pub author_name: String,
    pub author_id: String,
    pub waypoint_id: Option<i32>,
    pub name: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub z: Option<i32>,
    pub dimension: Option<i32>,
    pub completed_changed: Option<bool>,
    pub completed_value: Option<bool>,
}

impl TryInto<PendingWaypoint> for &PendingWaypointRow {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<PendingWaypoint, Self::Error> {
        let completed = match (self.completed_changed, self.completed_value) {
            (None, _) => None,
            (Some(false), _) => None,
            (Some(true), Some(val)) => Some(Some(val)),
            (Some(true), None) => Some(None),
        };

        let action = PendingWaypointAction::try_from(self.action.clone())?;

        let result = PendingWaypoint {
            id: self.id,
            action,
            author_name: self.author_name.clone(),
            author_id: self.author_id.clone(),
            waypoint_id: self.waypoint_id,
            x: self.x,
            y: self.y,
            z: self.z,
            name: self.name.clone(),
            dimension_id: self.dimension,
            dimension_name: None, // TODO: get this somehow
            completed,
        };

        Ok(result)
    }
}
