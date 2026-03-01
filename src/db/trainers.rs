use rusqlite::{params, Result};

use super::Db;

#[derive(Debug, Clone)]
pub struct Trainer {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub specialization: String,
}

impl Db {
    pub fn add_trainer(&self, name: &str, phone: &str, specialization: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO trainers (name, phone, specialization) VALUES (?1, ?2, ?3)",
            params![name, phone, specialization],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_trainer(
        &self,
        id: i64,
        name: &str,
        phone: &str,
        specialization: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE trainers SET name=?1, phone=?2, specialization=?3 WHERE id=?4",
            params![name, phone, specialization, id],
        )?;
        Ok(())
    }

    pub fn delete_trainer(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE clients SET trainer_id=NULL WHERE trainer_id=?1",
            params![id],
        )?;
        self.conn
            .execute("DELETE FROM trainers WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_trainers(&self) -> Result<Vec<Trainer>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, phone, specialization FROM trainers ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Trainer {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                specialization: row.get(3)?,
            })
        })?;
        rows.collect()
    }
}
