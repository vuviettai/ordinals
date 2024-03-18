use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

use super::table::Table;
pub fn establish_pgconnection() -> PgConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url)
    .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/*
 * Helper functions
 */

pub struct WriteTransaction {
  connection: PgConnection,
}

impl WriteTransaction {
  pub fn new() -> Self {
    let connection = establish_pgconnection();
    Self { connection }
  }
  pub fn open_table<DieselTable>(&mut self, table: DieselTable) -> Result<Table, anyhow::Error> {
    let table = Table::new(&mut self.connection, table);
    Ok(table)
  }
}
