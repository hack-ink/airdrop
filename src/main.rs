pub mod eth;
pub mod util;

mod avault;

mod prelude {
	pub use crate::{
		eth::{self, Account as EthAccount, Api as EthApi},
		util,
	};
	pub use anyhow::Result as AnyResult;
}
use prelude::*;

async fn farm() -> AnyResult<()> {
	avault::farm().await?;

	Ok(())
}

#[async_std::main]
async fn main() -> AnyResult<()> {
	tracing_subscriber::fmt::init();

	farm().await?;

	Ok(())
}
