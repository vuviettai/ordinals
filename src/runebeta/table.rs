use diesel::PgConnection;

use crate::{
  schema::{
    self,
    runes::{dsl::*, rune},
  },
  Result,
};

use super::models::Rune;

pub(super) struct Table<'db, 'tx, DieselTable> {
  pub connection: &'db mut PgConnection,
  pub table: DieselTable, //transaction: &'txn WriteTransaction<'db>,
                          //tree: BtreeMut<'txn, K, V>,
}

impl<'db, 'tx, DieselTable> Table<'db, 'tx, DieselTable> {
  pub fn new(connection: &'db mut PgConnection, table: DieselTable) -> Self {
    Self { connection, table }
  }
  pub fn insert<V>(self, value: &V) -> Result {
    diesel::insert_into(self.table)
      .values(value)
      .returning(DieselTable::as_returning())
      .get_result(self.connection)
      .expect("Error saving new post")
  }
  pub fn filter(&self) {
    let results = runes
      .filter(id.eq(true))
      .limit(5)
      .select(Rune::as_select())
      .load(self.connection)
      .expect("Error loading posts");
  }

  // pub fn inserts<V>(self, values: &[V]) -> Result {
  //   diesel::insert_into(self.table)
  //     .values(values)
  //     .returning(Rune::as_returning())
  //     .get_result(self.connection)
  //     .expect("Error saving new post")
  // }
}
