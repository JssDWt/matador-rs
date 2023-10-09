use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct LightningAddress {
    username: String,
    domain: String,
    callback: String,
}

use lightning_invoice::{Bolt11Invoice, SignedRawBolt11Invoice};

impl LightningAddress {
    pub async fn new(lnaddress: &str) -> Self {
        let (username, domain) = parse_lnaddress(lnaddress);
        let mut address = LightningAddress {
            username: username.to_string(),
            domain: domain.to_string(),
            callback: String::new(),
        };
        address.validate_and_set_callback().await;
        address
    }

    async fn validate_and_set_callback(&mut self) {
        let well_known_response = self.get_well_known_response().await;
        // TODO: Add validation logic here

        // if callback ends with "/" strip it
        if well_known_response.callback.ends_with("/") {
            self.callback =
                well_known_response.callback[..well_known_response.callback.len() - 1].to_string();
        } else {
            self.callback = well_known_response.callback;
        }
    }

    async fn get_well_known_response(&self) -> WellKnownResponse {
        let client = Client::new();
        let res = client
            .get(format!(
                "https://{}/.well-known/lnurlp/{}",
                self.domain, self.username
            ))
            .send()
            .await
            .expect("Failed to send request");

        serde_json::from_str(&res.text().await.unwrap()).expect("Failed to parse response")
    }

    pub async fn get_invoice(&self, amount_msat: i64) -> Bolt11Invoice {
        let client = Client::new();
        let callback_res = client
            .get(format!("{}?amount={}", self.callback, amount_msat))
            .send()
            .await
            .expect("Failed to send callback request");

        let response: CallbackResponse = serde_json::from_str(&callback_res.text().await.unwrap())
            .expect("Failed to parse callback response");

        Bolt11Invoice::from_signed(response.pr.parse::<SignedRawBolt11Invoice>().unwrap()).unwrap()
    }
}

fn parse_lnaddress(lnaddress: &str) -> (&str, &str) {
    let mut parts = lnaddress.split('@');
    let username = parts.next().expect("Invalid LNADDRESS");
    let domain = parts.next().expect("Invalid LNADDRESS");
    (username, domain)
}

#[derive(Debug, Deserialize)]
#[allow(unused, non_snake_case)]
struct PayerDataDetails {
    mandatory: bool,
}

#[derive(Debug, Deserialize)]
#[allow(unused, non_snake_case)]
struct PayerData {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<PayerDataDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<PayerDataDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pubkey: Option<PayerDataDetails>,
}

#[derive(Debug, Deserialize)]
#[allow(unused, non_snake_case)]
struct WellKnownResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    tag: String,
    commentAllowed: u32,
    callback: String,
    metadata: String,
    minSendable: i64,
    maxSendable: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    payerData: Option<PayerData>,
    nostrPubkey: String,
    allowsNostr: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct SuccessAction {
    tag: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused, non_snake_case)]
struct CallbackResponse {
    status: Option<String>,
    successAction: Option<SuccessAction>,
    verify: Option<String>,
    routes: Option<Vec<String>>,
    pr: String,
}
