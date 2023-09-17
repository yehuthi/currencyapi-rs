use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use currencyapi::{CurrencyCode, latest, RateLimitIgnore, Rates};

#[derive(Parser, Debug)]
pub struct Cli {
	token: String,
	#[clap(subcommand)]
	command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
	Rates {
		base: Option<CurrencyCode>,
		currencies: Vec<CurrencyCode>,
	},
	Convert {
		from: CurrencyCode,
		to: CurrencyCode,
		amount: f64,
	},
}

#[cfg(feature = "rust_decimal")] type Rate = rust_decimal::Decimal;
#[cfg(not(feature = "rust_decimal"))] type Rate = f64;

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	let client = reqwest::Client::new();

	let request = latest::Builder::from(cli.token.as_str());
	match cli.command {
		CliCommand::Rates { base, currencies } => {
			let mut rates = Rates::<Rate>::new();
			let request = request.base_currency(base).currencies(currencies).build();
			let metadata = rates
				.fetch_latest::<DateTime<Utc>, RateLimitIgnore>(&client, request)
				.await
				.unwrap();
			println!("Fetched {} rates as of {}", rates.len(), metadata.last_updated_at);
			for (currency, value) in rates.iter() { println!("{currency} {value}"); }
		}
		CliCommand::Convert { from, to, amount } => {
			let mut rates = Rates::<Rate>::new();
			let request = request.currencies([from,to]).build();
			rates.fetch_latest::<DateTime<Utc>, RateLimitIgnore>(&client, request).await.unwrap();
			println!("{} {} = {} {}", amount, from, rates.convert(&amount.try_into().unwrap(), from, to).unwrap(), to);
		}
	}
}
