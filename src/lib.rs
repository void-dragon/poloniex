//!
//! # Poloniex API
//!
//! API implementation for the [Poloniex](https://poloniex.com/) market-place.
//!
//! **Please Donate**
//!
//! + **BTC:** 17voJDvueb7iZtcLRrLtq3dfQYBaSi2GsU
//! + **ETC:** 0x7bC5Ff6Bc22B4C6Af135493E6a8a11A62D209ae5
//! + **XMR:** 49S4VziJ9v2CSkH6mP9km5SGeo3uxhG41bVYDQdwXQZzRF6mG7B4Fqv2aNEYHmQmPfJcYEnwNK1cAGLHMMmKaUWg25rHnkm
//!
//! **Poloniex API Documentation:**
//!
//! + https://poloniex.com/support/
//!
extern crate crypto;
extern crate curl;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha512;
use std::collections::HashMap;
use std::fmt::Write;
use std::io::Read;
use curl::easy::{Easy, List};


///
/// Representing a key secret pair from Poloniex.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub key: String,
    pub secret: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OpenOrder {
    #[serde(rename = "orderNumber")]
    pub order_number: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub rate: String,
    pub amount: String,
    pub total: String,
}

pub type OpenOrders = HashMap<String, OpenOrder>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OrderTrade {
    amount: String,
    date: String,
    rate: String,
    total: String,
    #[serde(rename = "tradeID")]
    trade_id: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Order {
    #[serde(rename = "orderNumber")]
    pub order_number: i64,
    #[serde(rename = "resultingTrades")]
    pub resulting_trades: Vec<OrderTrade>,
}

fn public(url: &str) -> Result<Vec<u8>, String> {
    let mut easy = Easy::new();
    let mut dst = Vec::new();

    easy.url(&format!("https://poloniex.com/public{}", url))
        .unwrap();

    let result = {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        transfer.perform()
    };

    result.map_err(|e| format!("{:?}", e)).and_then(
        |_x| Ok(dst),
    )
}

pub fn ticker() -> Result<Tick, String> {
    public("?command=returnTicker").and_then(|data| {
        serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
    })
}

fn private(account: &Account, params: &mut HashMap<String, String>) -> Result<Vec<u8>, String> {
    let timestamp = ::std::time::UNIX_EPOCH.elapsed().unwrap();
    let nonce = format!("{}{}", timestamp.as_secs(), timestamp.subsec_nanos());
    let mut dst = Vec::new();
    let mut easy = Easy::new();

    easy.url("https://poloniex.com/tradingApi").unwrap();
    easy.post(true).unwrap();

    params.insert("nonce".to_owned(), nonce);

    let mut body = params.iter().fold(
        String::new(),
        |data, item| data + item.0 + "=" + item.1 + "&",
    );
    body.pop();

    let mut body_bytes = body.as_bytes();
    let mut hmac = Hmac::new(Sha512::new(), account.secret.as_bytes());

    hmac.input(body_bytes);

    let mut list = List::new();
    let sign = hmac.result();

    let mut hex = String::new();
    for byte in sign.code() {
        write!(&mut hex, "{:02x}", byte).expect("could not create hmac hex");
    }
    list.append("Content-Type: application/x-www-form-urlencoded")
        .unwrap();
    list.append(&format!("Key: {}", account.key)).unwrap();
    list.append(&format!("Sign: {}", hex)).unwrap();

    easy.http_headers(list).unwrap();
    easy.post_field_size(body_bytes.len() as u64).unwrap();

    let result = {
        let mut transfer = easy.transfer();

        transfer
            .read_function(|buf| Ok(body_bytes.read(buf).unwrap_or(0)))
            .unwrap();

        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        transfer.perform()
    };

    result.map_err(|e| format!("{:?}", e)).and_then(
        |_x| Ok(dst),
    )
}

pub fn return_balances(account: &Account) -> Result<HashMap<String, String>, String> {
    let mut params = HashMap::new();

    params.insert("command".to_owned(), "returnBalances".to_owned());

    private(account, &mut params).and_then(|data| {
        serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
    })
}

pub fn return_open_orders(account: &Account, pair: Option<String>) -> Result<OpenOrders, String> {
    let mut params = HashMap::new();

    params.insert("command".to_owned(), "returnOpenOrders".to_owned());

    if let Some(p) = pair {
        params.insert("currencyPair".to_owned(), p);
    }

    private(account, &mut params).and_then(|data| {
        serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
    })
}

pub fn buy(account: &Account, pair: &str, rate: &str, amount: &str) -> Result<Order, String> {
    let mut params = HashMap::new();

    params.insert("command".to_owned(), "buy".to_owned());
    params.insert("currencyPair".to_owned(), String::from(pair));
    params.insert("rate".to_owned(), String::from(rate));
    params.insert("amount".to_owned(), String::from(amount));

    private(account, &mut params).and_then(|data| {
        serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
    })
}

pub fn sell(account: &Account, pair: &str, rate: &str, amount: &str) -> Result<Order, String> {
    let mut params = HashMap::new();

    params.insert("command".to_owned(), "sell".to_owned());
    params.insert("currencyPair".to_owned(), String::from(pair));
    params.insert("rate".to_owned(), String::from(rate));
    params.insert("amount".to_owned(), String::from(amount));

    private(account, &mut params).and_then(|data| {
        serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
    })
}
