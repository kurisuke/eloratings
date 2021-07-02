use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use std::env;

use crate::model::{MatchHistory, Team};

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn init() -> Result<Db, Box<dyn std::error::Error>> {
        dotenv().ok();
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
        Ok(Db { pool })
    }

    pub async fn insert_team(&self, team: &Team) -> Result<i64, Box<dyn std::error::Error>> {
        let mut conn = self.pool.acquire().await?;

        let id = sqlx::query!(
            r#"
REPLACE INTO teams ( team_id, team_name, successor_id )
VALUES ( ?1, ?2, ?3 )
"#,
            team.team_id,
            team.team_name,
            team.successor_id
        )
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn insert_match_history(
        &self,
        mh: &MatchHistory,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let mut conn = self.pool.acquire().await?;

        let date_str = mh.date.format("%Y-%m-%d").to_string();

        let id = sqlx::query!(
            r#"
REPLACE INTO match_history ( team_id, date, team_home, team_away, goals_home, goals_away, tournament, location, elo )
VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9 )
"#,
            mh.team_id,
            date_str,
            mh.team_home,
            mh.team_away,
            mh.goals_home,
            mh.goals_away,
            mh.tournament,
            mh.location,
            mh.elo
        )
            .execute(&mut conn)
            .await?
            .last_insert_rowid();

        Ok(id)
    }
}
