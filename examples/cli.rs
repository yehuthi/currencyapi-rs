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
		#[clap(parse(try_from_str))]
		from: CurrencyCode,
		#[clap(parse(try_from_str))]
		to: CurrencyCode,
		amount: f64,
	},
}

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	let client = reqwest::Client::new();

	let request = latest::Builder::from(cli.token.as_str());
	match cli.command {
		CliCommand::Rates { base, currencies } => {
			let mut request = request.currencies::<{ latest::buffer_size(4) }, _>(currencies);
			request.base_currency(base);
			let request = request.build();
			let response = request
				.send::<{ currency::list::LEN }>(&client)
				.await
				.unwrap();
			for (currency, value) in response.iter() {
				println!("{currency} {value}");
			}
		}
		CliCommand::Convert { from, to, amount } => {
			let request = request.build();
			let response = request.send::<180>(&client).await.unwrap();
			let result = response.convert(from, to, amount).unwrap();
			println!("{} {} = {} {}", amount, from, result, to);
		}
	}
}
