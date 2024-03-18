use anyhow::{bail, ensure, Context};
use bip39::Mnemonic;
use bitcoin::absolute::LockTime;
use bitcoin::{OutPoint, Transaction, TxOut, Txid};

use bitcoincore_rpc::{Auth, Client, RpcApi};
use ord::runes::etching::Etching;
use ord::schema::rune_entries::spacers;
use ord::settings::Settings;
use ord::wallet::Wallet;
use ord::{fund_raw_transaction, Chain, Edict, Rune, Runestone};
use ordinals::Height;
use reqwest::Url;
use std::str::FromStr;

const WALLET_NAME: &str = "taivv";
const PASSPHRASE: &str = "";
const MNEMONIC: &str =
  "artwork sweet believe ski tackle direct machine solar start joy dinosaur narrow";
const USERNAME: &str = "bitcointestnet";
const PASSWORD: &str = "bitcoincodelight";
//const BITCOIN_RPC_URL: &str = "http://222.253.82.244:18332";
const BITCOIN_RPC_URL: &str = "http://192.168.1.254:18332";
const WALLET_ADDRESS: &str = "tb1qlvsrakudqt2eca7w732cvkdq8awjgwtf8skssj";
const WALLET_PRIV_KEY: &str = "cNSfqJ4R9wU6Dfv4Yuuyc6Y63isCuBQBV6nMJmBCEF13bBdL8BHA";
fn main() -> Result<(), anyhow::Error> {
  let mnemonic = Mnemonic::from_str(MNEMONIC)?;
  let mnemonic_seed = mnemonic.to_seed(PASSPHRASE);
  let settings = Settings {
    bitcoin_data_dir: None,
    bitcoin_rpc_password: Some(PASSWORD.to_string()),
    bitcoin_rpc_url: Some(BITCOIN_RPC_URL.to_string()),
    bitcoin_rpc_username: Some(USERNAME.to_string()),
    chain: Some(Chain::Testnet),
    commit_interval: None,
    config: None,
    config_dir: None,
    cookie_file: None,
    data_dir: None,
    first_inscription_height: None,
    height_limit: None,
    hidden: None,
    index: None,
    index_cache_size: None,
    index_runes: true,
    index_sats: true,
    index_spent_sats: true,
    index_transactions: true,
    integration_test: false,
    no_index_inscriptions: false,
    server_password: None,
    server_url: None,
    server_username: None,
  };

  //Create bitcoin client
  let client = create_bitcoin_client()?;
  let blockchain_info = client.get_blockchain_info()?;
  println!("{:?}", &blockchain_info);
  let unload_result = client.unload_wallet(Some(WALLET_NAME));
  println!("UnLoadedWallet {:?}", &unload_result);
  let load_result = client.load_wallet(WALLET_NAME);
  println!("LoadedWallet {:?}", &load_result);
  //   let wallet = build_wallet(true, settings);
  //   if let Err(err) = wallet {
  //     println!("Build wallet error: {:?}", &err);
  //   }
  //Create wallet
  //Wallet::initialize(WALLET_NAME.to_string(), &settings, mnemonic_seed)?;
  Ok(())
}

fn create_bitcoin_client() -> Result<Client, anyhow::Error> {
  let bitcoin_credentials = Auth::UserPass(USERNAME.to_string(), PASSWORD.to_string());
  let client = Client::new(BITCOIN_RPC_URL, bitcoin_credentials)
    .with_context(|| format!("failed to connect to Bitcoin Core RPC at `{BITCOIN_RPC_URL}`"))?;
  Ok(client)
}
fn build_wallet(no_sync: bool, settings: Settings) -> Result<Wallet, anyhow::Error> {
  let wallet = Wallet::build(
    WALLET_NAME.to_string(),
    no_sync,
    settings.clone(),
    BITCOIN_RPC_URL
      .parse::<Url>()
      .context("invalid server URL")?,
  );
  wallet
}

fn etch(bitcoin_client: Client) -> Result<Txid, anyhow::Error> {
  //let SpacedRune { rune, spacers } = self.rune;
  let rune = Rune(100);
  let count = bitcoin_client.get_block_count()?;
  let symbol = "CODE";
  //   ensure!(
  //     wallet.get_rune(rune)?.is_none(),
  //     "rune `{}` has already been etched",
  //     rune,
  //   );

  let minimum_at_height =
    Rune::minimum_at_height(Chain::Testnet, Height(u32::try_from(count).unwrap() + 1));

  ensure!(
    rune >= minimum_at_height,
    "rune is less than minimum for next block: {} < {minimum_at_height}",
    rune,
  );

  //ensure!(!rune.is_reserved(), "rune `{}` is reserved", rune);

  //   ensure!(
  //     self.divisibility <= ord::runes::MAX_DIVISIBILITY,
  //     "<DIVISIBILITY> must be equal to or less than 38"
  //   );

  let destination = WALLET_ADDRESS.to_string();

  let runestone = Runestone {
    etching: Some(Etching {
      divisibility: 3,
      mint: None,
      rune: Some(rune),
      spacers: 0,
      symbol: Some('c'),
    }),
    edicts: vec![Edict {
      amount: 10000,
      id: 0,
      output: 1,
    }],
    default_output: None,
    burn: false,
    claim: None,
  };

  let script_pubkey = runestone.encipher();

  ensure!(
    script_pubkey.len() <= 82,
    "runestone greater than maximum OP_RETURN size: {} > 82",
    script_pubkey.len()
  );

  let unfunded_transaction = Transaction {
    version: 2,
    lock_time: LockTime::ZERO,
    input: Vec::new(),
    output: vec![
      TxOut {
        script_pubkey,
        value: 0,
      },
      TxOut {
        script_pubkey: destination.script_pubkey(),
        value: TARGET_POSTAGE.to_sat(),
      },
    ],
  };

  let inscriptions = wallet
    .inscriptions()
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<Vec<OutPoint>>();

  if !bitcoin_client.lock_unspent(&inscriptions)? {
    bail!("failed to lock UTXOs");
  }

  let unsigned_transaction =
    fund_raw_transaction(&bitcoin_client, self.fee_rate, &unfunded_transaction)?;

  let signed_transaction = bitcoin_client
    .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
    .hex;

  let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;

  Ok(transaction)
}

fn edict() {}
