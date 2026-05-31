# 🦀 Zakkistore SDK for Rust

**Official B2B Client Library for Zakki Store API Gateway**

Pustaka Rust resmi untuk memudahkan integrasi layanan Host-to-Host (H2H) prabayar/pascabayar, payment gateway QRIS otomatis, perbankan Virtual Account (VA), Noktel OTP virtual, mining reward, dan gacha koin Zakki Store ke dalam proyek Rust Anda.

---

## 🚀 Instalasi & Inisialisasi

Tambahkan dependensi berikut ke dalam file `Cargo.toml` proyek Anda:

```toml
[dependencies]
zakkistore-sdk = { git = "https://github.com/MrLow12/zakkistore-sdk-rust.git" }
serde_json = "1.0"
```

### Inisialisasi Klien

```rust
use zakkistore_sdk::ZakkiStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inisialisasi klien SDK
    let zakki = ZakkiStore::new(
        Some("https://qris.zakki.store"), // URL API Server
        Some("API_TOKEN_ANDA"),          // Token API
        Some("IBO99"),                   // iduser
        Some("member@gmail.com"),        // email
        Some("123456"),                  // PIN transaksi (Wajib untuk tarik/tabung)
        true,                            // Aktifkan auto-withdraw
    )?;
    
    Ok(())
}
```

---

## 🛠️ Fitur Unggulan

### 🔄 Auto-Withdraw Saldo VA
Jika opsi `auto_withdraw` diaktifkan sebagai `true`, SDK akan memicu penarikan dana VA bank otomatis secara *real-time* menjadi saldo utama aplikasi (BukaOlshop) ketika fungsi `checkbank()` dipanggil.

---

## 📑 Daftar Referensi Metode Lengkap

SDK Rust ini mendukung secara penuh seluruh **25 fungsi resmi** dengan nama dan perilaku yang konsisten dengan SDK versi Node.js (NPM):

### 1. Payment Gateway (QRIS Top Up)
*   `zakki.topup(nominal)` — Membuat QRIS dinamis instan dengan nominal kode unik.
*   `zakki.cektopup(idtopup)` — Cek status pembayaran QRIS.
*   `zakki.cancel(id_transaksi, all_pending)` — Batalkan transaksi pending.

### 2. Transaksi H2H
*   `zakki.listkode(jenis, product_type)` — Katalog kode produk aktif, deskripsi, dan harga.
*   `zakki.h2h(kode, tujuan, ref_id)` — Mengirim order transaksi H2H.
*   `zakki.cekh2h(id_trx)` — Cek detail status pengisian, SN, dan harga beli order H2H.
*   `zakki.myh2h()` — Mengambil 20 riwayat pembelian H2H terupdate.

### 3. Perbankan & Transfer VA
*   `zakki.checkbank()` — Cek saldo, VA member, mutasi, dan pemicu Auto-Withdraw.
*   `zakki.checkname(number)` — Verifikasi nama asli pemilik VA Bank Zakki tujuan.
*   `zakki.transfer(to, amount)` — Transfer saldo antar Virtual Account member Bank Zakki.
*   `zakki.tabung(jumlah)` — Menabung / deposit saldo dari aplikasi utama (BukaOlshop) ke Bank (butuh PIN).
*   `zakki.tarik(jumlah)` — Menarik dana tabungan ke saldo aplikasi (butuh PIN).
*   `zakki.checkmutasi(mutasi_type)` — Riwayat mutasi Tarik/Tabung (`tarik`, `tabung`, `all`).

### 4. Noktel Marketplace (OTP Virtual)
*   `zakki.noktelStok()` — Cek stok nomor virtual yang ready.
*   `zakki.noktelBuy(category)` — Membeli nomor virtual baru untuk OTP.
*   `zakki.noktelGetOtp(account_id)` — Menarik kode OTP Telegram secara real-time.
*   `zakki.noktelCancel(invoice_id)` — Membatalkan nomor yang pending OTP & auto-refund.
*   `zakki.noktelHistory()` — Mengambil daftar riwayat pembelian Noktel.

### 5. Reward Komputasi & Game
*   `zakki.cekmining()` — Cek status kesulitan global, block reward, dan miner aktif.
*   `zakki.mymining()` — Riwayat koin mining SHA256 milik akun Anda.
*   `zakki.cekgacha()` — Statistik poin, kemenangan, dan keuntungan gacha member.

### 6. Keamanan & Utilitas
*   `zakki.whitelistip(ip)` — Whitelist IP server Anda untuk otorisasi API H2H.
*   `zakki.delwhitelistip(ip)` — Hapus IP server dari whitelist.
*   `zakki.leaderboard(limit, period)` — Mengambil peringkat sultan topup teraktif.
*   `zakki.status()` — Informasi beban CPU, metrik finansial, dan kesehatan sistem.

---

## 🛡️ Protokol Keamanan API

> [!WARNING]
> **Selalu jalankan SDK ini di sisi backend (Server-side)!**
> Jangan pernah mengekspos API Token dan PIN Anda langsung di frontend aplikasi / browser klien publik demi mencegah potensi pencurian saldo.
