use rusqlite::{Result, params};

use super::Db;

#[derive(Debug, Clone)]
pub struct ProgramCheckIn {
    pub id: i64,
    pub client_program_id: i64,
    pub date: String,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct ExerciseCompletion {
    pub id: i64,
    pub exercise_id: i64,
    pub name: String,
    pub sets: i64,
    pub reps: i64,
    pub weight: i64,
    pub notes: String,
    pub completed: bool,
}

impl Db {
    pub fn toggle_checkin(&self, checkin_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE program_checkins SET completed = NOT completed WHERE id=?1",
            params![checkin_id],
        )?;
        Ok(())
    }

    pub fn get_checkins_for_client_program(
        &self,
        client_program_id: i64,
    ) -> Result<Vec<ProgramCheckIn>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, client_program_id, date, completed \
             FROM program_checkins WHERE client_program_id=?1 ORDER BY date",
        )?;
        let rows = stmt.query_map(params![client_program_id], |row| {
            Ok(ProgramCheckIn {
                id: row.get(0)?,
                client_program_id: row.get(1)?,
                date: row.get(2)?,
                completed: row.get::<_, i64>(3)? != 0,
            })
        })?;
        rows.collect()
    }

    pub fn get_client_program_for_checkin(&self, checkin_id: i64) -> i64 {
        self.conn
            .query_row(
                "SELECT client_program_id FROM program_checkins WHERE id=?1",
                params![checkin_id],
                |row| row.get(0),
            )
            .unwrap_or(0)
    }

    pub fn get_exercise_completions(
        &self,
        client_program_id: i64,
    ) -> Result<Vec<ExerciseCompletion>> {
        let program_id: i64 = self.conn.query_row(
            "SELECT program_id FROM client_programs WHERE id=?1",
            params![client_program_id],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "INSERT OR IGNORE INTO exercise_completions (client_program_id, exercise_id, completed)
             SELECT ?1, e.id, 0 FROM exercises e WHERE e.program_id=?2",
            params![client_program_id, program_id],
        )?;

        let mut stmt = self.conn.prepare(
            "SELECT ec.id, e.id, e.name, e.sets, e.reps, e.weight, e.notes, ec.completed
             FROM exercise_completions ec
             JOIN exercises e ON e.id = ec.exercise_id
             WHERE ec.client_program_id=?1
             ORDER BY e.id",
        )?;
        let rows = stmt.query_map(params![client_program_id], |row| {
            Ok(ExerciseCompletion {
                id: row.get(0)?,
                exercise_id: row.get(1)?,
                name: row.get(2)?,
                sets: row.get(3)?,
                reps: row.get(4)?,
                weight: row.get(5)?,
                notes: row.get(6)?,
                completed: row.get::<_, i64>(7)? != 0,
            })
        })?;
        rows.collect()
    }

    /// Returns `(exercise_id, checkin_id, completed)` ordered by exercise, then checkin.
    /// Ensures rows exist for every (exercise × checkin) pair before querying.
    pub fn get_exercise_week_completions(
        &self,
        client_program_id: i64,
    ) -> Result<Vec<(i64, i64, bool)>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO exercise_checkin_completions
                (client_program_id, exercise_id, checkin_id, completed)
             SELECT ?1, e.id, pc.id, 0
             FROM exercises e
             CROSS JOIN program_checkins pc
             WHERE e.program_id = (SELECT program_id FROM client_programs WHERE id=?1)
               AND pc.client_program_id = ?1",
            params![client_program_id],
        )?;
        let mut stmt = self.conn.prepare(
            "SELECT ecc.exercise_id, ecc.checkin_id, ecc.completed
             FROM exercise_checkin_completions ecc
             WHERE ecc.client_program_id=?1
             ORDER BY ecc.exercise_id ASC, ecc.checkin_id ASC",
        )?;
        let rows = stmt.query_map(params![client_program_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)? != 0,
            ))
        })?;
        rows.collect()
    }

    pub fn toggle_exercise_week(
        &self,
        client_program_id: i64,
        exercise_id: i64,
        checkin_id: i64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE exercise_checkin_completions SET completed = NOT completed
             WHERE client_program_id=?1 AND exercise_id=?2 AND checkin_id=?3",
            params![client_program_id, exercise_id, checkin_id],
        )?;
        Ok(())
    }

    pub fn toggle_exercise_completion(
        &self,
        client_program_id: i64,
        exercise_id: i64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE exercise_completions SET completed = NOT completed
             WHERE client_program_id=?1 AND exercise_id=?2",
            params![client_program_id, exercise_id],
        )?;
        Ok(())
    }
}
