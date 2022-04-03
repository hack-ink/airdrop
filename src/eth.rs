use crate::prelude::*;

#[derive(Clone)]
pub struct Account {
	pub secret_key: secp256k1::SecretKey,
	pub address: String,
}
impl Account {
	const STORE: &'static str = "eth-accounts";

	pub fn random() -> Self {
		use secp256k1::{rand, Secp256k1};
		use tiny_keccak::Hasher;

		let secp = Secp256k1::new();
		let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
		let mut address = [0; 32];
		let mut hasher = tiny_keccak::Keccak::v256();

		// Never out of range.
		hasher.update(&public_key.serialize_uncompressed()[1..]);
		hasher.finalize(&mut address);

		// Never out of range.
		let address = array_bytes::bytes2hex("0x", &address[12..]);

		Self {
			secret_key,
			address,
		}
	}

	pub fn hex_secret_key(&self) -> String {
		array_bytes::bytes2hex("0x", self.secret_key.serialize_secret())
	}

	pub fn save(&self) -> AnyResult<()> {
		use std::{fs::OpenOptions, io::Write};

		let mut w = OpenOptions::new()
			.create(true)
			.append(true)
			.open(Self::STORE)?;

		writeln!(w, "{},{}", self.hex_secret_key(), self.address)?;

		Ok(())
	}

	pub fn from_str(secret: &str, address: &str) -> AnyResult<Self> {
		use secp256k1::SecretKey;

		Ok(Self {
			secret_key: SecretKey::from_slice(
				&array_bytes::hex2bytes(secret)
					.map_err(|_| anyhow::anyhow!("Invalid secret: {}.", secret))?,
			)?,
			address: address.to_string(),
		})
	}

	pub fn load() -> AnyResult<Vec<Self>> {
		use std::{fs::File, io::Read};

		let mut r = File::open(Self::STORE)?;
		let mut s = String::new();
		let mut accounts = Vec::new();

		r.read_to_string(&mut s)?;

		for l in s.lines() {
			let (secret, address) = l
				.split_once(',')
				.ok_or_else(|| anyhow::anyhow!("Corrupt DB: {}.", l))?;

			accounts.push(Self::from_str(secret, address)?);
		}

		Ok(accounts)
	}
}

pub struct Api(web3::Web3<web3::transports::Http>);
impl Api {
	pub fn new(uri: &str) -> AnyResult<Self> {
		use web3::{transports::Http, Web3};

		Ok(Self(Web3::new(Http::new(uri)?)))
	}

	pub fn eth(&self) -> web3::api::Eth<web3::transports::Http> {
		self.0.eth()
	}

	pub async fn nonce_of(&self, address: &str) -> AnyResult<web3::types::U256> {
		loop {
			match self
				.eth()
				.transaction_count(
					array_bytes::hex_try_into(address)
						.map_err(|_| anyhow::anyhow!("Invalid address: {}.", address))?,
					None,
				)
				.await
			{
				Ok(n) => return Ok(n),
				Err(e) => {
					tracing::error!("{:?}", e);
				}
			}
		}
	}

	fn build_contract(
		&self,
		contract: &str,
		abi: &str,
	) -> AnyResult<web3::contract::Contract<web3::transports::Http>> {
		use std::fs::File;

		use ethabi::Contract as ContractAbi;
		use web3::contract::Contract;

		Ok(Contract::new(
			self.eth(),
			array_bytes::hex_into_unchecked(contract),
			ContractAbi::load(File::open(abi)?)?,
		))
	}

	pub async fn query<P, F, R>(
		&self,
		contract: &str,
		abi: &str,
		func: &str,
		params: P,
		from: F,
		options: web3::contract::Options,
	) -> AnyResult<R>
	where
		P: Clone + web3::contract::tokens::Tokenize,
		F: Clone + Into<Option<ethabi::Address>>,
		R: std::fmt::Debug + web3::contract::tokens::Detokenize,
	{
		let contract = self.build_contract(contract, abi)?;

		loop {
			let options = options.clone();
			let params = params.clone();
			let from = from.clone();

			match contract.query(func, params, from, options, None).await {
				Ok(r) => {
					tracing::debug!("{:?}", r);

					return Ok(r);
				}
				Err(e) => {
					tracing::error!("{:?}", e);
				}
			}
		}
	}

	pub async fn send_tx<P>(
		&self,
		account: &Account,
		contract: &str,
		abi: &str,
		func: &str,
		params: P,
		mut options: web3::contract::Options,
	) -> AnyResult<web3::types::H256>
	where
		P: Clone + web3::contract::tokens::Tokenize,
	{
		let contract = self.build_contract(contract, abi)?;
		let address = array_bytes::hex_into_unchecked(&account.address);

		loop {
			// Skip, if the nonce had already been provided.
			if options.nonce.is_none() {
				if let Ok(nonce) = self.eth().transaction_count(address, None).await {
					options.nonce = Some(nonce);
				}
			}

			let options = options.clone();
			let params = params.clone();

			match contract
				.signed_call(func, params, options, &account.secret_key)
				.await
			{
				Ok(r) => {
					tracing::debug!("{:?}", r);

					return Ok(r);
				}
				Err(e) => {
					// TODO: handle nonce too low
					tracing::error!("{:?}", e);
				}
			}
		}
	}
}
