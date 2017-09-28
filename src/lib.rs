extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha512;
use std::collections::HashMap;
use std::fmt::Write;
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;


///
/// Representing a key secret pair from Poloniex.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub key: String,
    pub secret: String,
}

#[derive(Deserialize, Serialize)]
pub struct TickPair {
    pub id: u32,
    pub last: String,
    #[serde(rename = "lowestAsk")]
    pub lowest_ask: String,
    #[serde(rename = "highestBid")]
    pub highest_bid: String,
    #[serde(rename = "percentChange")]
    pub percent_change: String,
    #[serde(rename = "baseVolume")]
    pub base_volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    #[serde(rename = "isFrozen")]
    pub is_frozen: String,
    pub high24hr: String,
    pub low24hr: String,
}

pub type Tick = HashMap<String, TickPair>;

pub struct Poloniex {
    core: Core,
    client: Client<::hyper_tls::HttpsConnector<::hyper::client::HttpConnector>>,
}

impl Poloniex {
    pub fn new() -> Poloniex {
        let core = Core::new().unwrap();
        let client = Client::configure()
            .connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
            .build(&core.handle());

        Poloniex {
            client: client,
            core: core,
        }
    }

    fn public(&mut self, url: &str) -> Result<::hyper::Chunk, ::hyper::Error> {
        let work = self.client
            .get(
                format!("https://poloniex.com/public{}", url)
                    .parse()
                    .unwrap(),
            )
            .and_then(|res| res.body().concat2());

        self.core.run(work)
    }

    pub fn ticker(&mut self) -> Result<Tick, String> {
        self.public("?command=returnTicker")
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

    fn private(
        &mut self,
        account: &Account,
        params: &mut HashMap<String, String>,
    ) -> Result<hyper::Chunk, hyper::Error> {
        let url = "https://poloniex.com/tradingApi";
        let timestamp = ::std::time::UNIX_EPOCH.elapsed().unwrap();
        let nonce = format!("{}{}", timestamp.as_secs(), timestamp.subsec_nanos());

        params.insert("nonce".to_owned(), nonce);

        let mut body = params.iter().fold(
            String::new(),
            |data, item| data + item.0 + "=" + item.1 + "&",
        );
        body.pop();

        let mut hmac = Hmac::new(Sha512::new(), account.secret.as_bytes());

        hmac.input(body.as_bytes());


        let mut req = ::hyper::Request::new(::hyper::Method::Post, url.parse().unwrap());

        {
            let headers = req.headers_mut();
            let sign = hmac.result();

            let mut hex = String::new();
            for byte in sign.code() {
                write!(&mut hex, "{:02x}", byte).expect("could not create hmac hex");
            }

            headers.set_raw("Content-type", "application/x-www-form-urlencoded");
            headers.set_raw("Key", account.key.clone());
            headers.set_raw("Sign", hex);
        }

        req.set_body(body);

        let work = self.client.request(req).and_then(
            |res| res.body().concat2(),
        );

        self.core.run(work)
    }

    pub fn return_balances(&mut self, account: &Account) -> Result<HashMap<String, String>, String> {
        let mut params = HashMap::new();

        params.insert("command".to_owned(), "returnBalances".to_owned());

        self.private(account, &mut params)
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }
}
