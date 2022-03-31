use currencyapi::Latest;

#[tokio::main]
async fn main() {
	let token = std::env::args().nth(1).unwrap();
	let client = reqwest::Client::new();
	let response = Latest::new(token.as_str(), None, std::iter::empty())
		.send::<180>(&client)
		.await
		.unwrap();
	for (currency, value) in response.iter() {
		println!("{currency}, {value}");
	}
}
