use diesel::{
  deserialize::{FromSql, FromSqlRow},
  dsl::IsNull,
  expression::AsExpression,
  pg::Pg,
  prelude::*,
  serialize::{Output, ToSql},
  sql_types::{Jsonb, Text},
};
//https://stackoverflow.com/questions/77629993/error-extending-diesel-with-wrapper-type-for-u128
#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[diesel(sql_type = Text)]
pub struct U128(pub u128);
impl ToSql<Text, Pg> for U128 {
  fn to_sql<'b>(&self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    write!(out, "{}", self.0.to_string())?;
    Ok(IsNull::No)
  }
}

impl FromSql<Text, Pg> for U128 {
  fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
    let s = String::from_utf8_lossy(bytes.as_bytes());
    Ok(U128(s.parse()?))
  }
}

#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[diesel(sql_type = Jsonb)]
pub struct MintEntry {}

impl FromSql<Jsonb, Pg> for MintEntry {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl ToSql<Jsonb, Pg> for MintEntry {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
  }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutpointRuneBalance {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i16,
  pub balance_id: String,
  pub balance_value: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RuneEntries {
  pub id: i64,
  pub rune_height: i32,
  pub rune_index: i16,
  pub burned: Vec<u8>,
  pub divisibility: i16,
  pub etching: String,
  pub mint: Option<MintEntry>,
  pub mints: i64,
  pub rnumber: i64,
  pub spacers: i32,
  pub supply: Vec<u8>,
  pub symbol: Option<String>,
  pub rtimestamp: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Rune {
  pub id: i64,
  pub rune: String,
  pub tx_height: u64,
  pub rune_index: i16,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::sequence_number_runeid)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SequenceNumberRuneId {
  pub id: i64,
  pub sequence_number: i32,
  pub tx_height: u64,
  pub rune_index: i16,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexingStatistic {
  pub id: i64,
  pub schema: i32,
  pub blessed_inscriptions: i32,
  pub commits: i32,
  pub cursed_inscriptions: i32,
  pub index_runes: i32,
  pub index_sats: i32,
  pub lost_sats: i32,
  pub outputs_traversed: i32,
  pub reserved_runes: i32,
  pub satranges: i64,
  pub unbound_inscriptions: i32,
  pub index_transactions: i32,
  pub index_spend_sats: i32,
  pub initial_sync_time: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIndexingStatistic<'a> {
  pub schema: i32,
  pub blessed_inscriptions: i32,
  pub commits: i32,
  pub cursed_inscriptions: i32,
  pub index_runes: i32,
  pub index_sats: i32,
  pub lost_sats: i32,
  pub outputs_traversed: i32,
  pub reserved_runes: i32,
  pub satranges: i64,
  pub unbound_inscriptions: i32,
  pub index_transactions: i32,
  pub index_spend_sats: i32,
  pub initial_sync_time: i64,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::txid_rune)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxidRune {
  pub id: i64,
  pub tx_hash: String,
  pub rune: U128,
}

///Models for create

#[derive(Insertable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
pub struct NewOutpointRuneBalance<'a> {
  pub tx_hash: &'a str,
  pub vout: i16,
  pub balance_id: &'a str,
  pub balance_value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRuneEntries<'a> {
  pub rune_height: u64,
  pub rune_index: i16,
  pub burned: &'a [u8],
  pub divisibility: i16,
  pub etching: &'a str,
  pub mint: Option<MintEntry>,
  pub mints: i64,
  pub rnumber: i64,
  pub spacers: i32,
  pub supply: &'a [u8],
  pub symbol: Option<&'a str>,
  pub rtimestamp: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRune<'a> {
  pub rune: &'a str,
  pub tx_height: u64,
  pub rune_index: i16,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sequence_number_runeid)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSequenceNumberRuneId {
  pub sequence_number: i32,
  pub tx_height: u64,
  pub rune_index: i16,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::txid_rune)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTxidRune<'a> {
  pub tx_hash: &'a str,
  pub rune: U128,
}

//ContentTypeCounts
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::content_type_counts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContentTypeCount {
  pub id: i32,
  pub content_type: Option<String>,
  pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::content_type_counts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContentTypeCount {
  pub content_type: Option<String>,
  pub count: i64,
}

//Inscription
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::inscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Inscriptions {
  pub id: i64,
  pub home: i32,
  pub sequence_number: i32,
  pub head: U128,
  pub tail: U128,
  pub inscription_index: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::inscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewInscriptions {
  pub home: i32,
  pub sequence_number: i32,
  pub head: U128,
  pub tail: U128,
  pub inscription_index: i32,
}

//InscriptionEntry
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::inscription_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InscriptionEntry {
  pub id: i64,
  pub charms: i16,
  pub fee: i64,
  pub height: i32,
  pub tx_hash: String,
  pub inscription_index: i32,
  pub inscription_number: i32,
  pub parent: Option<i32>,
  pub sat: Option<i64>,
  pub sequence_number: i32,
  pub timestamp: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::inscription_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewInscriptionEntry {
  pub charms: i16,
  pub fee: i64,
  pub height: i32,
  pub tx_hash: String,
  pub inscription_index: i32,
  pub inscription_number: i32,
  pub parent: Option<i32>,
  pub sat: Option<i64>,
  pub sequence_number: i32,
  pub timestamp: i64,
}

//Satpoint
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::satpoints)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Satpoint {
  pub id: i64,
  pub sequence_number: i32,
  pub tx_hash: String,
  pub vout: i32,
  pub sat_offset: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::satpoints)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSatpoint {
  pub sequence_number: i32,
  pub tx_hash: String,
  pub vout: i32,
  pub sat_offset: i64,
}

//Transaction
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
  pub id: i64,
  pub version: i32,
  pub lock_time: i32,
  pub tx_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransaction {
  pub version: i32,
  pub lock_time: i32,
  pub tx_hash: String,
}

//TransactionIn
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionIn {
  pub id: i64,
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i64,
  pub script_sig: String,
  pub sequence_number: i64,
  pub witness: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionIn {
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i64,
  pub script_sig: String,
  pub sequence_number: i64,
  pub witness: String,
}

//TransactionOut
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionOut {
  pub id: i64,
  pub tx_hash: String,
  pub value: i64,
  pub script_pubkey: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionOut {
  pub tx_hash: String,
  pub value: i64,
  pub script_pubkey: String,
}

//BlockTimestamp
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::indexing_block_timestamps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexingBlockTimestamp {
  pub id: i64,
  pub block_height: i32,
  pub timestamps: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::indexing_block_timestamps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIndexingBlockTimestamp {
  pub block_height: i32,
  pub timestamps: i64,
}
