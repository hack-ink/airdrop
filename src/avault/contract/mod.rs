use crate::prelude::*;

const GAS_PRICE: u128 = 1_100_000_000_u128;
const ZAP_CONTRACT: &str = "0x5af88505cf2ce57bb5e36816d7853a221f6fc981";
const DEPOSIT_CONTRACT: &str = "0x9a6080753a35dcd8e77102ae83a93170a831393e";
const LP_CONTRACT: &str = "0x456c0082de0048ee883881ff61341177fa1fef40";

// https://blockscout.com/shiden/address/0x5Af88505CF2cE57bb5e36816d7853A221F6Fc981/transactions
pub async fn zap_in(
	api: &EthApi,
	account: &EthAccount,
	nonce: ethabi::Uint,
) -> AnyResult<web3::types::H256> {
	use ethabi::{Address, Token, Uint};
	use web3::contract::Options;

	let to = Address::from(array_bytes::hex2array_unchecked(LP_CONTRACT));

	api.send_tx(
		account,
		ZAP_CONTRACT,
		"src/avault/contract/zap-kaco-shiden.json",
		"zapIn",
		(Token::Address(to),),
		Options::with(|o| {
			o.value = Some(Uint::from(50_000_000_000_000_u128));
			o.gas_price = Some(Uint::from(GAS_PRICE));
			o.gas = Some(Uint::from(500_000_u128));
			o.nonce = Some(nonce);
		}),
	)
	.await
}

pub async fn approve_kaco_lp(
	api: &EthApi,
	account: &EthAccount,
	nonce: ethabi::Uint,
) -> AnyResult<web3::types::H256> {
	use ethabi::{Address, Token, Uint};
	use web3::contract::Options;

	let spender = Address::from(array_bytes::hex2array_unchecked(DEPOSIT_CONTRACT));
	let amount = Uint::from_dec_str(
		"115792089237316195423570985008687907853269984665640564039457584007913129639935",
	)
	.expect("Never fail.");

	api.send_tx(
		account,
		LP_CONTRACT,
		"src/avault/contract/a-vault-pcs.json",
		"approve",
		(Token::Address(spender), Token::Uint(amount)),
		Options::with(|o| {
			o.gas_price = Some(Uint::from(GAS_PRICE));
			o.gas = Some(Uint::from(500_000_u128));
			o.nonce = Some(nonce);
		}),
	)
	.await
}

pub async fn deposit_a_vault(
	api: &EthApi,
	account: &EthAccount,
	nonce: ethabi::Uint,
) -> AnyResult<web3::types::H256> {
	// use std::time::Duration;

	// use async_std::task;
	use ethabi::{Address, Token, Uint};
	use web3::contract::Options;

	let user_address = Address::from(array_bytes::hex2array_unchecked(&account.address));
	let want_amt = Uint::from(130000000000000_u128);
	// let want_amt = {
	// 	let mut amount = Uint::from(0_u32);

	// 	while amount == Uint::from(0_u32) {
	// 		let a = if let Token::Uint(a) = api
	// 			.query(
	// 				LP_CONTRACT,
	// 				"src/avault/contract/a-vault-pcs.json",
	// 				"balanceOf",
	// 				(Token::Address(user_address),),
	// 				None,
	// 				Options::default(),
	// 			)
	// 			.await?
	// 		{
	// 			amount = a;
	// 		} else {
	// 			unreachable!()
	// 		};

	// 		task::sleep(Duration::from_secs(6)).await;
	// 	}

	// 	amount
	// };

	api.send_tx(
		account,
		DEPOSIT_CONTRACT,
		"src/avault/contract/a-vault-pcs.json",
		"deposit",
		(Token::Address(user_address), Token::Uint(want_amt)),
		Options::with(|o| {
			o.gas_price = Some(Uint::from(GAS_PRICE));
			o.gas = Some(Uint::from(500_000_u128));
			o.nonce = Some(Uint::from(nonce));
		}),
	)
	.await
}
