#![forbid(unsafe_code)]

use async_trait::async_trait;
use axum_session::{DatabaseError, DatabasePool, Session, SessionStore};
use chrono::Utc;
use sqlx::{pool::Pool, PgPool, Postgres};

pub type SessionPgSession = Session<SessionPgPool>;
pub type SessionPgSessionStore = SessionStore<SessionPgPool>;

#[derive(Debug, Clone)]
pub struct SessionPgPool {
    pool: Pool<Postgres>,
}

impl From<Pool<Postgres>> for SessionPgPool {
    fn from(conn: PgPool) -> Self {
        SessionPgPool { pool: conn }
    }
}

fn extract_user_id_from_session(session_json: &str) -> Option<i64> {
    let value: serde_json::Value = serde_json::from_str(session_json).ok()?;
    value
        .get("data")?
        .get("user_auth_session_id")?
        .as_str()?
        .parse()
        .ok()
}

#[async_trait]
impl DatabasePool for SessionPgPool {
    async fn initiate(&self, table_name: &str) -> Result<(), DatabaseError> {
        sqlx::query(
            &r#"
            create table if not exists %%TABLE_NAME%% (
                id varchar(128) not null primary key,
                expires bigint null,
                session text not null,
                user_id bigint references users(id) on delete cascade
            )
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericCreateError(err.to_string()))?;

        let (t,): (bool,) = sqlx::query_as(
            &r#"
            select data_type = 'integer'
            from information_schema.columns
            where table_name = '%%TABLE_NAME%%' and column_name = 'expires'
            "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericCreateError(err.to_string()))?;

        if t {
            sqlx::query(
                &r#"
                alter table %%TABLE_NAME%% alter column expires type bigint
                "#
                .replace("%%TABLE_NAME%%", table_name),
            )
            .execute(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericCreateError(err.to_string()))?;
        }

        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let result: Vec<(String,)> = sqlx::query_as(
            &r#"
            select id from %%TABLE_NAME%%
            where (expires is null or expires < $1)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .fetch_all(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let result: Vec<String> = result.into_iter().map(|(s,)| s).collect();

        sqlx::query(
            &r#"delete from %%TABLE_NAME%% where expires < $1"#
                .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
        Ok(result)
    }

    async fn count(&self, table_name: &str) -> Result<i64, DatabaseError> {
        let (count,) = sqlx::query_as(
            &r#"select count(*) from %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        Ok(count)
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), DatabaseError> {
        let user_id = extract_user_id_from_session(session);

        sqlx::query(
            &r#"
        insert into %%TABLE_NAME%%
            (id, session, expires, user_id) select $1, $2, $3, $4
        on conflict(id) do update set
            expires = excluded.expires,
            session = excluded.session,
            user_id = excluded.user_id
    "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .bind(session)
        .bind(expires)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericInsertError(err.to_string()))?;

        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, DatabaseError> {
        let result: Option<(String,)> = sqlx::query_as(
            &r#"
            select session from %%TABLE_NAME%%
            where id = $1 and (expires is null or expires > $2)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        Ok(result.map(|(session,)| session))
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), DatabaseError> {
        sqlx::query(
            &r#"delete from %%TABLE_NAME%% where id = $1"#.replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, DatabaseError> {
        let result: Option<(i64,)> = sqlx::query_as(
            &r#"
            select count(*) from %%TABLE_NAME%%
            where id = $1 and (expires is null or expires > $2)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        Ok(result.map(|(o,)| o).unwrap_or(0) > 0)
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), DatabaseError> {
        sqlx::query(&r#"truncate %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name))
            .execute(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let result: Vec<(String,)> = sqlx::query_as(
            &r#"
            select id from %%TABLE_NAME%%
            where (expires is null or expires > $1)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .fetch_all(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let result: Vec<String> = result.into_iter().map(|(s,)| s).collect();

        Ok(result)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
