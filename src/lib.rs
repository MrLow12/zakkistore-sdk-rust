use serde_json::{json, Value};
use std::error::Error;

pub struct ZakkiStore {
    base_url: String,
    token: String,
    iduser: Option<String>,
    email: Option<String>,
    pin: Option<String>,
    auto_withdraw: bool,
}

impl ZakkiStore {
    pub fn new(
        base_url: Option<&str>,
        token: Option<&str>,
        iduser: Option<&str>,
        email: Option<&str>,
        pin: Option<&str>,
        auto_withdraw: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let mut final_base_url = base_url.unwrap_or("https://qris.zakki.store").to_string();
        let mut final_token = token.unwrap_or("").to_string();

        // Smart detection if token is placed in base_url parameter
        if let Some(url_or_token) = base_url {
            if !url_or_token.starts_with("http://") && !url_or_token.starts_with("https://") && token.is_none() {
                final_token = url_or_token.to_string();
                final_base_url = "https://qris.zakki.store".to_string();
            }
        }

        if final_token.is_empty() {
            return Err("token wajib disertakan dalam konfigurasi SDK.".into());
        }
        if final_base_url.is_empty() {
            return Err("base_url wajib disertakan dalam konfigurasi SDK.".into());
        }

        if final_base_url.ends_with('/') {
            final_base_url.pop();
        }

        Ok(Self {
            base_url: final_base_url,
            token: final_token,
            iduser: iduser.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
            pin: pin.map(|s| s.to_string()),
            auto_withdraw,
        })
    }

    pub fn enable_auto_withdraw(&mut self, status: bool) {
        self.auto_withdraw = status;
    }

    pub fn enableAutoWithdraw(&mut self, status: bool) {
        self.enable_auto_withdraw(status);
    }

    // ==========================================================
    // --- 1. PAYMENT GATEWAY (QRIS TOPUP) ---
    // ==========================================================

    pub fn topup(&self, nominal: u64) -> Result<Value, Box<dyn Error>> {
        self._request("/topup", "POST", Some(json!({
            "token": self.token,
            "nominal": nominal
        })))
    }

    pub fn cektopup(&self, idtopup: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/cektopup", "GET", Some(json!({
            "idtopup": idtopup
        })))
    }

    pub fn cektopup2(&self, idtopup: &str) -> String {
        let encoded: String = idtopup.chars().map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        }).collect();
        format!("{}/cektopup2?idtopup={}", self.base_url.trim_end_matches('/'), encoded)
    }

    pub fn cancel(&self, id_transaksi: Option<&str>, all_pending: bool) -> Result<Value, Box<dyn Error>> {
        let mut payload = json!({ "token": self.token });
        if let Some(id) = id_transaksi {
            payload["id_transaksi"] = json!(id);
        }
        if all_pending {
            payload["all"] = json!(true);
        }
        self._request("/cancel", "POST", Some(payload))
    }

    // ==========================================================
    // --- 2. TRANSAKSI H2H (HOST-TO-HOST) ---
    // ==========================================================

    pub fn listkode(&self, jenis: Option<&str>, product_type: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let mut payload = json!({});
        if let Some(j) = jenis {
            payload["jenis"] = json!(j);
        }
        if let Some(t) = product_type {
            payload["type"] = json!(t);
        }
        self._request("/listkode", "GET", Some(payload))
    }

    pub fn h2h(&self, kode: &str, tujuan: Option<&str>, ref_id: Option<&str>) -> Result<Value, Box<dyn Error>> {
        self._request("/h2h", "POST", Some(json!({
            "token": self.token,
            "kode": kode,
            "tujuan": tujuan,
            "refID": ref_id
        })))
    }

    pub fn cekh2h(&self, id_trx: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/cekh2h", "GET", Some(json!({ "id": id_trx })))
    }

    pub fn myh2h(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/myh2h", "GET", Some(json!({ "token": self.token })))
    }

    // ==========================================================
    // --- 3. PERBANKAN & TRANSFER SALDO ---
    // ==========================================================

    pub fn checkbank(&self) -> Result<Value, Box<dyn Error>> {
        let mut payload = json!({ "token": self.token });
        if let Some(ref id) = self.iduser {
            payload["iduser"] = json!(id);
        } else if let Some(ref email) = self.email {
            payload["email"] = json!(email);
        }

        let mut bank_res = self._request("/checkbank", "GET", Some(payload.clone()))?;

        if self.auto_withdraw {
            if let Some(data) = bank_res.get("data") {
                if let Some(bank_detail) = data.get("bank_detail") {
                    if let Some(balance_val) = bank_detail.get("balance") {
                        let balance = match balance_val {
                            Value::Number(n) => n.as_f64().unwrap_or(0.0),
                            Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                            _ => 0.0,
                        };

                        if balance > 0.0 {
                            let balance_u64 = balance as u64;
                            match self.tarik(balance_u64) {
                                Ok(withdraw_res) => {
                                    if let Ok(updated_res) = self._request("/checkbank", "GET", Some(payload)) {
                                        bank_res = updated_res;
                                        bank_res["auto_withdraw_executed"] = json!(true);
                                        bank_res["auto_withdraw_amount"] = json!(balance_u64);
                                        bank_res["auto_withdraw_message"] = json!(withdraw_res.get("message").and_then(|v| v.as_str()).unwrap_or("Auto-withdraw berhasil dijalankan."));
                                    }
                                }
                                Err(e) => {
                                    bank_res["auto_withdraw_executed"] = json!(false);
                                    bank_res["auto_withdraw_error"] = json!(e.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(bank_res)
    }

    pub fn checkname(&self, number: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/checkname", "GET", Some(json!({ "number": number.trim() })))
    }

    pub fn transfer(&self, to: &str, amount: u64) -> Result<Value, Box<dyn Error>> {
        self._request("/transfer", "POST", Some(json!({
            "token": self.token,
            "to": to,
            "amount": amount
        })))
    }

    pub fn tabung(&self, jumlah: u64) -> Result<Value, Box<dyn Error>> {
        let pin = self.pin.as_ref().ok_or("[ZakkiStore SDK Error] PIN transaksi diperlukan untuk melakukan transaksi tabung.")?;
        let mut payload = json!({
            "token": self.token,
            "jumlah": jumlah,
            "pin": pin
        });
        if let Some(ref id) = self.iduser {
            payload["iduser"] = json!(id);
        }
        if let Some(ref email) = self.email {
            payload["email"] = json!(email);
        }
        self._request("/tabung", "POST", Some(payload))
    }

    pub fn tarik(&self, jumlah: u64) -> Result<Value, Box<dyn Error>> {
        let pin = self.pin.as_ref().ok_or("[ZakkiStore SDK Error] PIN transaksi diperlukan untuk melakukan transaksi tarik.")?;
        let mut payload = json!({
            "token": self.token,
            "jumlah": jumlah,
            "pin": pin
        });
        if let Some(ref id) = self.iduser {
            payload["iduser"] = json!(id);
        }
        if let Some(ref email) = self.email {
            payload["email"] = json!(email);
        }
        self._request("/tarik", "POST", Some(payload))
    }

    pub fn checkmutasi(&self, mutasi_type: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let mut payload = json!({
            "token": self.token,
            "type": mutasi_type.unwrap_or("all")
        });
        if let Some(ref id) = self.iduser {
            payload["iduser"] = json!(id);
        }
        if let Some(ref email) = self.email {
            payload["email"] = json!(email);
        }
        self._request("/checkmutasi", "GET", Some(payload))
    }

    // ==========================================================
    // --- 4. NOKTEL MARKETPLACE (OTP VIRTUAL) ---
    // ==========================================================

    pub fn noktel_stok(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/noktel/stok", "GET", Some(json!({ "token": self.token })))
    }

    pub fn noktelStok(&self) -> Result<Value, Box<dyn Error>> {
        self.noktel_stok()
    }

    pub fn noktel_buy(&self, category: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/noktel/buy", "POST", Some(json!({
            "token": self.token,
            "category": category.trim()
        })))
    }

    pub fn noktelBuy(&self, category: &str) -> Result<Value, Box<dyn Error>> {
        self.noktel_buy(category)
    }

    pub fn noktel_get_otp(&self, account_id: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/noktel/getotp", "GET", Some(json!({
            "token": self.token,
            "account_id": account_id.trim()
        })))
    }

    pub fn noktelGetOtp(&self, account_id: &str) -> Result<Value, Box<dyn Error>> {
        self.noktel_get_otp(account_id)
    }

    pub fn noktel_cancel(&self, invoice_id: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/noktel/cancel", "POST", Some(json!({
            "token": self.token,
            "invoice_id": invoice_id.trim()
        })))
    }

    pub fn noktelCancel(&self, invoice_id: &str) -> Result<Value, Box<dyn Error>> {
        self.noktel_cancel(invoice_id)
    }

    pub fn noktel_history(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/noktel/history", "GET", Some(json!({ "token": self.token })))
    }

    pub fn noktelHistory(&self) -> Result<Value, Box<dyn Error>> {
        self.noktel_history()
    }

    // ==========================================================
    // --- 5. REWARD KOMPUTASI & UTILITY ---
    // ==========================================================

    pub fn cekmining(&self, idmining: &str) -> Result<Value, Box<dyn Error>> {
        if idmining.is_empty() {
            return Err("Parameter idmining wajib diisi.".into());
        }
        self._request("/cekmining", "GET", Some(json!({ "idmining": idmining.trim() })))
    }

    pub fn mymining(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/mymining", "GET", Some(json!({ "token": self.token })))
    }

    pub fn mining_start(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/mining/start", "GET", Some(json!({ "token": self.token })))
    }

    pub fn miningStart(&self) -> Result<Value, Box<dyn Error>> {
        self.mining_start()
    }

    pub fn mining_submit(&self, nonce: Value, signature: &str) -> Result<Value, Box<dyn Error>> {
        if signature.is_empty() {
            return Err("Parameter signature wajib disertakan.".into());
        }
        self._request("/mining/submit", "POST", Some(json!({
            "token": self.token,
            "nonce": nonce,
            "signature": signature
        })))
    }

    pub fn miningSubmit(&self, nonce: Value, signature: &str) -> Result<Value, Box<dyn Error>> {
        self.mining_submit(nonce, signature)
    }

    pub fn cekgacha(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/cekgacha", "GET", Some(json!({ "token": self.token })))
    }

    // ==========================================================
    // --- 6. UTILITY & SECURITY ---
    // ==========================================================

    pub fn whitelistip(&self, ip: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/whitelistip", "POST", Some(json!({
            "token": self.token,
            "ip": ip.trim()
        })))
    }

    pub fn delwhitelistip(&self, ip: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/delwhitelistip", "POST", Some(json!({
            "token": self.token,
            "ip": ip.trim()
        })))
    }

    pub fn leaderboard(&self, limit: u32, period: Option<&str>) -> Result<Value, Box<dyn Error>> {
        self._request("/leaderboard", "GET", Some(json!({
            "limit": limit,
            "period": period.unwrap_or("all").trim()
        })))
    }

    pub fn status(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/status", "GET", None)
    }

    // ==========================================================
    // --- 7. METODE INTEGRASI BARU ---
    // ==========================================================

    pub fn set_callback(&self, site: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/setcallback", "GET", Some(json!({
            "token": self.token,
            "site": site.trim()
        })))
    }

    pub fn setcallback(&self, site: &str) -> Result<Value, Box<dyn Error>> {
        self.set_callback(site)
    }

    pub fn del_callback(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/delcallback", "GET", Some(json!({ "token": self.token })))
    }

    pub fn delcallback(&self) -> Result<Value, Box<dyn Error>> {
        self.del_callback()
    }

    pub fn set_notif_bot(&self, telegram_id: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/setnotifbot", "GET", Some(json!({
            "token": self.token,
            "id": telegram_id.trim()
        })))
    }

    pub fn setnotifbot(&self, telegram_id: &str) -> Result<Value, Box<dyn Error>> {
        self.set_notif_bot(telegram_id)
    }

    pub fn del_notif_bot(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/delnotifbot", "GET", Some(json!({ "token": self.token })))
    }

    pub fn delnotifbot(&self) -> Result<Value, Box<dyn Error>> {
        self.del_notif_bot()
    }

    pub fn check_transfer(&self, idtransfer: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/checktransfer", "GET", Some(json!({ "idtransfer": idtransfer.trim() })))
    }

    pub fn checktransfer(&self, idtransfer: &str) -> Result<Value, Box<dyn Error>> {
        self.check_transfer(idtransfer)
    }

    pub fn my_transfer(&self, transfer_type: Option<&str>) -> Result<Value, Box<dyn Error>> {
        self._request("/mytransfer", "GET", Some(json!({
            "token": self.token,
            "type": transfer_type.unwrap_or("all").trim()
        })))
    }

    pub fn mytransfer(&self, transfer_type: Option<&str>) -> Result<Value, Box<dyn Error>> {
        self.my_transfer(transfer_type)
    }

    pub fn my_topup(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/mytopup", "GET", Some(json!({ "token": self.token })))
    }

    pub fn mytopup(&self) -> Result<Value, Box<dyn Error>> {
        self.my_topup()
    }

    pub fn cek_my_ip(&self) -> Result<Value, Box<dyn Error>> {
        self._request("/cekmyip", "GET", None)
    }

    pub fn cekmyip(&self) -> Result<Value, Box<dyn Error>> {
        self.cek_my_ip()
    }

    pub fn cek_ip(&self, ip: &str) -> Result<Value, Box<dyn Error>> {
        self._request("/cekip", "GET", Some(json!({ "ip": ip.trim() })))
    }

    pub fn cekip(&self, ip: &str) -> Result<Value, Box<dyn Error>> {
        self.cek_ip(ip)
    }

    // ==========================================================
    // --- CORE REQUEST HANDLER ---
    // ==========================================================

    fn _request(&self, endpoint: &str, method: &str, data: Option<Value>) -> Result<Value, Box<dyn Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = if method.to_uppercase() == "GET" {
            let mut req = ureq::get(&url);
            if let Some(params) = data {
                if let Some(obj) = params.as_object() {
                    for (k, v) in obj {
                        let v_str = match v {
                            Value::String(s) => s.to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => v.to_string(),
                        };
                        req = req.query(k, &v_str);
                    }
                }
            }
            req.call()
        } else {
            let req = ureq::post(&url).set("Content-Type", "application/json");
            if let Some(body) = data {
                req.send_json(body)
            } else {
                req.send_json(json!({}))
            }
        };

        match response {
            Ok(res) => {
                let res_json: Value = res.into_json()?;
                Ok(res_json)
            }
            Err(ureq::Error::Status(code, res)) => {
                let res_text = res.into_string().unwrap_or_default();
                let mut res_json: Value = serde_json::from_str(&res_text).unwrap_or(json!({ "message": res_text }));
                
                let mut err_msg = res_json.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("HTTP Error")
                    .to_string();

                if code == 403 || err_msg.to_lowercase().contains("ip") {
                    err_msg.push_str("\n⚠️ [IP BLOCKED / UNREGISTERED] IP Anda diblokir atau belum terdaftar di whitelist API. Silakan hubungi developer via WhatsApp (https://wa.me/6283844082339) atau Telegram (https://t.me/zakki_store) untuk mendapatkan bantuan.");
                }

                Err(format!("[ZakkiStore SDK Error] {}", err_msg).into())
            }
            Err(e) => Err(format!("[ZakkiStore SDK Error] Koneksi Gagal: {}", e).into()),
        }
    }
}
