use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};
use sqlx::{FromRow, Row};
use std::path::Path;
use time::OffsetDateTime;
use log::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: i32,
    pub name: Option<String>,
    pub digest: String,
    pub size: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub save_time: OffsetDateTime,
    pub more_info: Option<String>,
}

impl FromRow<'_, sqlx::sqlite::SqliteRow> for Backup {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let save_time_str: String = row.try_get("save_time")?;
        let save_time = OffsetDateTime::parse(
            &save_time_str,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "save_time".to_string(),
            source: Box::new(e),
        })?;

        Ok(Backup {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            digest: row.try_get("digest")?,
            size: row.try_get("size")?,
            save_time,
            more_info: row.try_get("more_info")?,
        })
    }
}

const SCHEMA_SQL: &str = r"
CREATE TABLE IF NOT EXISTS backups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT ,
    digest TEXT NOT NULL,
    size INTEGER NOT NULL,
    save_time TEXT DEFAULT (datetime('now')),
    more_info TEXT
);
";

pub struct Db {}

impl Db {
    pub async fn new(db_path: String) -> anyhow::Result<SqliteConnection> {
        // 确保父目录存在
        let path = Path::new(&db_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // 规范化为绝对路径，并统一为正斜杠
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let dsn = format!("sqlite://{}", abs.to_string_lossy().replace('\\', "/"));

        let mut conn = SqliteConnection::connect(&dsn).await?;

        // 创建表
        sqlx::query(SCHEMA_SQL).execute(&mut conn).await?;

        Ok(conn)
    }

    pub async fn store_backup(backup: &Backup, conn: &mut SqliteConnection) -> anyhow::Result<()> {
        let save_time_str = backup
            .save_time
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        sqlx::query(
            r#"INSERT INTO backups (name, digest, size, save_time, more_info)
               VALUES (?, ?, ?, ?, ?)"#,
        )
            .bind(&backup.name)
            .bind(&backup.digest)
            .bind(backup.size)
            .bind(save_time_str)
            .bind(&backup.more_info)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn get_all_backup(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Backup>> {
        let backups = sqlx::query_as::<_, Backup>(
            r#"SELECT id, name, digest, size, save_time, more_info FROM backups"#,
        )
            .fetch_all(conn)
            .await?;

        Ok(backups)
    }

    pub async fn get_backup_by_id(
        conn: &mut SqliteConnection,
        id: i32,
    ) -> anyhow::Result<Option<Backup>> {
        let backup = sqlx::query_as::<_, Backup>(
            r#"SELECT id, name, digest, size, save_time, more_info FROM backups WHERE id = ?"#,
        )
            .bind(id)
            .fetch_optional(conn)
            .await?;

        Ok(backup)
    }

    pub async fn delete_backup(conn: &mut SqliteConnection, backup: &Backup) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM backups WHERE id = ?")
            .bind(backup.id)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn get_backup_by_digest(
        conn: &mut SqliteConnection,
        digest: &str,
    ) -> anyhow::Result<Option<Backup>> {
        let backup = sqlx::query_as::<_, Backup>(
            r#"SELECT id, name, digest, size, save_time, more_info FROM backups WHERE digest = ?"#,
        )
            .bind(digest)
            .fetch_optional(conn)
            .await?;

        Ok(backup)
    }
}

