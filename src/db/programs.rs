use rusqlite::{Result, params};

use super::Db;

#[derive(Debug, Clone)]
pub struct TrainingProgram {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub duration_weeks: i64,
}

#[derive(Debug, Clone)]
pub struct Exercise {
    pub id: i64,
    pub program_id: i64,
    pub name: String,
    pub sets: i64,
    pub reps: i64,
    pub weight: i64,
    pub notes: String,
}

impl Db {
    pub fn add_program(&self, name: &str, description: &str, duration_weeks: i64) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO training_programs (name, description, duration_weeks) VALUES (?1, ?2, ?3)",
            params![name, description, duration_weeks],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_program(
        &self,
        id: i64,
        name: &str,
        description: &str,
        duration_weeks: i64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE training_programs SET name=?1, description=?2, duration_weeks=?3 WHERE id=?4",
            params![name, description, duration_weeks, id],
        )?;
        Ok(())
    }

    pub fn delete_program(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM exercise_completions WHERE client_program_id IN \
             (SELECT id FROM client_programs WHERE program_id=?1)",
            params![id],
        )?;
        self.conn.execute(
            "DELETE FROM exercise_completions WHERE exercise_id IN \
             (SELECT id FROM exercises WHERE program_id=?1)",
            params![id],
        )?;
        self.conn.execute(
            "DELETE FROM program_checkins WHERE client_program_id IN \
             (SELECT id FROM client_programs WHERE program_id=?1)",
            params![id],
        )?;
        self.conn.execute(
            "DELETE FROM client_programs WHERE program_id=?1",
            params![id],
        )?;
        self.conn
            .execute("DELETE FROM exercises WHERE program_id=?1", params![id])?;
        self.conn
            .execute("DELETE FROM training_programs WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_programs(&self) -> Result<Vec<TrainingProgram>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, duration_weeks FROM training_programs ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TrainingProgram {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                duration_weeks: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    pub fn add_exercise(
        &self,
        program_id: i64,
        name: &str,
        sets: i64,
        reps: i64,
        weight: i64,
        notes: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO exercises (program_id, name, sets, reps, weight, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![program_id, name, sets, reps, weight, notes],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_exercise(
        &self,
        id: i64,
        name: &str,
        sets: i64,
        reps: i64,
        weight: i64,
        notes: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE exercises SET name=?1, sets=?2, reps=?3, weight=?4, notes=?5 WHERE id=?6",
            params![name, sets, reps, weight, notes, id],
        )?;
        Ok(())
    }

    pub fn delete_exercise(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM exercise_completions WHERE exercise_id=?1",
            params![id],
        )?;
        self.conn
            .execute("DELETE FROM exercises WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_exercises_for_program(&self, program_id: i64) -> Result<Vec<Exercise>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, program_id, name, sets, reps, weight, notes \
             FROM exercises WHERE program_id=?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![program_id], |row| {
            Ok(Exercise {
                id: row.get(0)?,
                program_id: row.get(1)?,
                name: row.get(2)?,
                sets: row.get(3)?,
                reps: row.get(4)?,
                weight: row.get(5)?,
                notes: row.get(6)?,
            })
        })?;
        rows.collect()
    }
}
