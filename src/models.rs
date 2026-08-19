use std::fmt::Display;

#[derive(Debug, sqlx::FromRow)]
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

    pub fn try_from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Overworld),
            2 => Some(Self::Nether),
            3 => Some(Self::End),
            _ => None,
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

impl TryFrom<String> for DimensionEnum {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "overworld" | "the overworld" => Ok(Self::Overworld),
            "nether" | "the nether" => Ok(Self::Overworld),
            "end" | "the end" => Ok(Self::Overworld),
            _ => Err(anyhow::Error::msg(format!(
                "Couldn't convert '{}' to DimensionEnum",
                value
            ))),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
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

impl TryInto<Waypoint> for PendingWaypoint {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Waypoint, Self::Error> {
        let id = self.waypoint_id.unwrap_or(0);
        let x = match self.x {
            Some(val) => val,
            None => return Err(anyhow::Error::msg("X was not defined")),
        };
        let y = match self.x {
            Some(val) => val,
            None => return Err(anyhow::Error::msg("Y was not defined")),
        };
        let z = match self.x {
            Some(val) => val,
            None => return Err(anyhow::Error::msg("Z was not defined")),
        };
        let name = match self.name {
            Some(val) => val,
            None => return Err(anyhow::Error::msg("Name was not defined")),
        };
        let dimension = match self.dimension_name {
            Some(val) => val,
            None => return Err(anyhow::Error::msg("Dimension Name was not defined")),
        };

        let completed = self.completed.unwrap_or(None);

        Ok(Waypoint {
            id,
            x,
            y,
            z,
            name,
            dimension,
            completed,
        })
    }
}

#[derive(Debug)]
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

#[derive(Debug, sqlx::FromRow)]
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

impl TryInto<PendingWaypoint> for PendingWaypointRow {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<PendingWaypoint, Self::Error> {
        let completed = match (self.completed_changed, self.completed_value) {
            (None, _) => None,
            (Some(false), _) => None,
            (Some(true), Some(val)) => Some(Some(val)),
            (Some(true), None) => Some(None),
        };

        let action = PendingWaypointAction::try_from(self.action)?;

        let dimension_name =
            DimensionEnum::try_from_id(self.dimension.unwrap_or(-1)).map(|val| val.to_string());

        let result = PendingWaypoint {
            id: self.id,
            action,
            author_name: self.author_name,
            author_id: self.author_id,
            waypoint_id: self.waypoint_id,
            x: self.x,
            y: self.y,
            z: self.z,
            name: self.name.clone(),
            dimension_id: self.dimension,
            dimension_name,
            completed,
        };

        Ok(result)
    }
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

        let dimension_name =
            DimensionEnum::try_from_id(self.dimension.unwrap_or(-1)).map(|val| val.to_string());

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
            dimension_name,
            completed,
        };

        Ok(result)
    }
}
