use sqlx::{postgres::PgPoolOptions, query_as};

use crate::models;

pub type Pool = sqlx::Pool<sqlx::Postgres>;

pub async fn create_pool() -> anyhow::Result<Pool> {
    let url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    Ok(pool)
}

pub async fn select_waypoints(pool: &Pool) -> anyhow::Result<Vec<models::Waypoint>> {
    let waypoints = query_as!(
        models::Waypoint,
        "
select 
    w.id id, 
    w.name name, 
    x, y, z, 
    d.name dimension, 
    completed
from waypoints w 
join dimensions d on w.dimension = d.id
order by id
"
    )
    .fetch_all(pool)
    .await?;

    Ok(waypoints)
}
