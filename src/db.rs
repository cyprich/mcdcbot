use log::error;
use sqlx::{postgres::PgPoolOptions, query_as, query_scalar};

use crate::models::{self, DimensionEnum, Waypoint};

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
) -> anyhow::Result<Vec<models::Waypoint>> {
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
