use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};
use sqlx::{FromRow, Row};
use std::path::Path;
use time::OffsetDateTime;
use log::{info, error};
use urlencoding::encode;
use sqlx::sqlite::{SqliteConnectOptions};
use sqlx::ConnectOptions; // 引入 trait 以使用 connect_with

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: i32,
    pub name: Option<String>,
    pub digest: String,
    pub size: i64,
    pub path: String,
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
            path: row.try_get("path")?,
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
    path TEXT NOT NULL,
    save_time TEXT DEFAULT (datetime('now')),
    more_info TEXT
);
";

pub struct Db {}

impl Db {
    pub async fn new(db_path: String) -> anyhow::Result<SqliteConnection> {
        info!("开始初始化数据库，路径: {}", db_path);
        
        let path = Path::new(&db_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                sqlx::Error::Io(e) // 将 IO 错误转换为 sqlx 错误，或者你自己的错误处理
            })?;
            info!("父目录已创建/确认: {}", parent.display());
        }

        // 2. 获取绝对路径 (保持原有逻辑)
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(sqlx::Error::Io)?
                .join(path)
        };
        
        info!("绝对路径: {}", abs.display());

        // 使用 SqliteConnectOptions
        let options = SqliteConnectOptions::new()
            .filename(&abs) // 直接传入 PathBuf，库会自动处理路径转义
            .create_if_missing(true); // 如果数据库文件不存在则创建

        // 4. 建立连接
        let mut conn = SqliteConnection::connect_with(&options).await.map_err(|e| {
            error!("连接数据库失败: {}", e);
            e
        })?;
        
        info!("数据库连接成功");

        // 创建表
        sqlx::query(SCHEMA_SQL).execute(&mut conn).await?;
        
        info!("数据库表已创建/确认");

        Ok(conn)
    }

    pub async fn store_backup(backup: &Backup, conn: &mut SqliteConnection) -> anyhow::Result<()> {
        let save_time_str = backup
            .save_time
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        sqlx::query(
            r#"INSERT INTO backups (name, digest, size,path, save_time, more_info)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
            .bind(&backup.name)
            .bind(&backup.digest)
            .bind(backup.size)
            .bind(&backup.path)
            .bind(save_time_str)
            .bind(&backup.more_info)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn get_all_backup(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Backup>> {
        let backups = sqlx::query_as::<_, Backup>(
            r#"SELECT id, name, digest, size,path, save_time, more_info FROM backups"#,
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
            r#"SELECT id, name, digest, size, path, save_time, more_info FROM backups WHERE id = ?"#,
        )
            .bind(id)
            .fetch_optional(conn)
            .await?;

        Ok(backup)
    }

    pub async fn delete_backup(conn: &mut SqliteConnection, id: i32) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM backups WHERE id = ?")
            .bind(id)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn get_backup_by_digest(
        conn: &mut SqliteConnection,
        digest: &str,
    ) -> anyhow::Result<Option<Backup>> {
        let backup = sqlx::query_as::<_, Backup>(
            r#"SELECT id, name, digest, size, path, save_time, more_info FROM backups WHERE digest = ?"#,
        )
            .bind(digest)
            .fetch_optional(conn)
            .await?;

        Ok(backup)
    }
}

