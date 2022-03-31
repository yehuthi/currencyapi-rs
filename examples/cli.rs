use clap::{Parser, Subcommand};
use currencyapi::{currency::CurrencyCode, Latest};

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

	match cli.command {
		CliCommand::Rates { base, currencies } => {
			let request = Latest::new(cli.token.as_str(), base, currencies.iter().copied());
			let response = request.send::<180>(&client).await.unwrap();
			for (currency, value) in response.iter() {
				println!("{currency} {value}");
			}
		}
		CliCommand::Convert { from, to, amount } => {
			let request = Latest::new(cli.token.as_str(), None, std::iter::empty());
			let response = request.send::<180>(&client).await.unwrap();
			let result = response.convert(from, to, amount).unwrap();
			println!("{} {} = {} {}", amount, from, result, to);
		}
	}
}
