mod contract;

use crate::prelude::*;

async fn drip(task_count: u16) -> EthAccount {
	use async_std::task;
	use futures::prelude::*;
	use reqwest::{header::*, ClientBuilder};

	let account = EthAccount::random();
	let client = ClientBuilder::new()
		.default_headers(HeaderMap::from_iter([(
			ORIGIN,
			"https://portal.astar.network"
				.parse()
				.expect("Invalid URI."),
		)]))
		.build()
		.expect("Never fail.");
	let json = serde_json::json!({"destination":account.address});
	let mut tasks = Vec::new();

	(0..task_count).for_each(|_| {
		let account = account.clone();
		let client = client.clone();
		let json = json.clone();

		tasks.push(task::spawn(async move {
			loop {
				if let Ok(resp) = client
					.post("https://astar-discord-faucet.herokuapp.com/shiden/drip")
					.json(&json)
					.send()
					.await
				{
					if let Ok(r) = resp.text().await {
						if [
							"hash",
							"You already requested the Faucet",
							"Cannot request more",
						]
						.iter()
						.any(|s| r.contains(s))
						{
							tracing::info!("{}: {}", account.address, r);

							return <Result<_, ()>>::Ok(account);
						} else {
							tracing::debug!("{}", r);
						}
					}
				}
			}
		}))
	});

	let (account, _) = future::select_ok(tasks).await.expect("Never fail.");

	account.save().unwrap();

	account
}

async fn interact_contract(account: &EthAccount) -> AnyResult<()> {
	let api = EthApi::new("https://evm.shiden.astar.network")?;
	let nonce = api.nonce_of(&account.address).await?;

	tracing::info!(
		"zap_in: {:?}",
		contract::zap_in(&api, &account, nonce).await?
	);
	tracing::info!(
		"approve_kaco_lp: {:?}",
		contract::approve_kaco_lp(&api, &account, nonce + 1).await?
	);
	tracing::info!(
		"deposit_a_vault: {:?}",
		contract::deposit_a_vault(&api, &account, nonce + 2).await?
	);

	Ok(())
}

pub async fn farm() -> AnyResult<()> {
	// Test.
	// let account = EthAccount::from_str(
	// 	"secret-key-here",
	// 	"address-here",
	// )?;
	// interact_contract(&account).await?;

	// Acquire SDN.
	// Can not turn on while you are `farm`ing.
	// loop {
	// 	drip(256);
	// }

	// Farm.
	for account in EthAccount::load()? {
		tracing::info!("account {}", account.address);
		interact_contract(&account).await?;
	}

	Ok(())
}
