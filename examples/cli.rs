use clap::{Parser, Subcommand};
use currencyapi::{
	currency::{self, CurrencyCode},
	latest,
};

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
			let request = request
				.currencies(currencies)
				.base_currency(base);
			let request = request.build();
			let response = request
				.send::<{ currency::list::ARRAY.len() }, Rate>(&client)
				.await
				.unwrap();
			for (currency, value) in response.rates.iter() {
				println!("{currency} {value}");
			}
		}
		CliCommand::Convert { from, to, amount } => {
			let request = request.build();
			let response = request.send::<180, Rate>(&client).await.unwrap();
			let result = response
				.rates
				.convert(&amount.try_into().unwrap(), from, to)
				.unwrap();
			println!("{} {} = {} {}", amount, from, result, to);
		}
	}
}
