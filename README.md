# Poloniex API

API implementation for the [Poloniex](https://poloniex.com/) market-place.

**Please Donate**

+ **ETC:** 0x7bC5Ff6Bc22B4C6Af135493E6a8a11A62D209ae5
+ **XMR:** 49S4VziJ9v2CSkH6mP9km5SGeo3uxhG41bVYDQdwXQZzRF6mG7B4Fqv2aNEYHmQmPfJcYEnwNK1cAGLHMMmKaUWg25rHnkm

**Poloniex API Documentation:**
+ https://poloniex.com/support/


## Example

```rust
extern crate poloniex;

fn main() {
  let mut api = poloniex::Poloniex::new();

  api
    .ticker()
    .map_err(|e| println!("a buh-buh happend, {}", e))
    .map(|tick| if let Some(info) = tick.get("USDT_BTC") {
        println!("{:?}", info.lowest_ask);
      });
}
```
