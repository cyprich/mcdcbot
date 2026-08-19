use log::{error, warn};
use sqlx::{postgres::PgPoolOptions, query_as, query_scalar};

use crate::models::{DimensionEnum, PendingWaypoint, PendingWaypointRow, Waypoint};

pub type Pool = sqlx::Pool<sqlx::Postgres>;
type Builder = sqlx::QueryBuilder<sqlx::Postgres>;

pub async fn create_pool() -> anyhow::Result<Pool> {
    let url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    Ok(pool)
}

pub async fn select_waypoints(
    pool: &Pool,
    dimension: &Option<DimensionEnum>,
    completed: &Option<bool>,
) -> anyhow::Result<Vec<crate::models::Waypoint>> {
    let mut tx = pool.begin().await?;
    let mut builder = Builder::new(
        "select 
            w.id id, 
            w.name name, 
            x, y, z, 
            d.name dimension, 
            completed
        from waypoints w 
        join dimensions d on w.dimension = d.id 
        where 1=1 ",
    );

    // chceck dimension
    if let Some(dimension) = dimension {
        let dim_id = query_scalar!(
            "select id from dimensions where name = $1",
            dimension.to_string()
        )
        .fetch_one(&mut *tx)
        .await;

        if let Err(e) = dim_id {
            error!("{}", e);
            return Err(anyhow::Error::from(e));
        }

        builder.push(" and dimension = ");
        builder.push_bind(dim_id.unwrap());
    }

    // check completed
    if let Some(completed) = completed {
        builder.push(" and completed = ");
        builder.push_bind(completed);
    }

    // order
    builder.push(" order by id");

    // build & fetch
    let waypoints = builder
        .build_query_as::<Waypoint>()
        .fetch_all(&mut *tx)
        .await?;

    Ok(waypoints)
}

pub async fn select_pending_waypoints(pool: &Pool) -> anyhow::Result<Vec<PendingWaypoint>> {
    let rows = query_as!(
        PendingWaypointRow,
        "select * from pending_waypoints order by id"
    )
    .fetch_all(pool)
    .await?;

    let result = rows
        .iter()
        .filter_map(|r| match r.try_into() {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Failed converting '{:?}' to PendingWaypoing: {}", r, e);
                None
            }
        })
        .collect::<Vec<PendingWaypoint>>();

    Ok(result)
}

pub async fn insert_pending_waypoint(
    pool: &Pool,
    waypoint: &PendingWaypoint,
) -> anyhow::Result<i32> {
    let mut tx = pool.begin().await?;

    let dimension = match &waypoint.dimension_name {
        // if dimension was changed - we need to find ID
        Some(dimension) => {
            let id = query_scalar!("select id from dimensions where name = $1", dimension)
                .fetch_optional(&mut *tx)
                .await?;

            match id {
                Some(val) => Some(val),
                None => {
                    return Err(anyhow::Error::msg(format!(
                        "Couldn't find dimension '{}' in database",
                        dimension
                    )));
                }
            }
        }
        // if dimension was not changed, leave it be
        None => None,
    };

    // make `Option<Option<bool>>` into two variables for database
    let (completed_changed, completed_value) = match waypoint.completed {
        Some(value) => (true, value),
        None => (false, None),
    };

    let result = query_scalar!(
        "insert into pending_waypoints
        (action, author_name, author_id, waypoint_id, name, x, y, z, dimension, completed_changed, completed_value) 
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        returning id",
        waypoint.action.to_string(),
        waypoint.author_name,
        waypoint.author_id,
        waypoint.waypoint_id,
        waypoint.name,
        waypoint.x,
        waypoint.y,
        waypoint.z,
        dimension,
        completed_changed,
        completed_value
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(result)
}
