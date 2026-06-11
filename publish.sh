#!/bin/bash
# Script helper untuk mempublikasikan Rust Crate Zakki Store SDK ke crates.io

# Silakan set token Anda atau masukkan saat diminta
if [ -z "$CRATES_IO_TOKEN" ]; then
  read -sp "Masukkan Crates.io API Token Anda: " CRATES_IO_TOKEN
  echo ""
fi

if [ -z "$CRATES_IO_TOKEN" ]; then
  echo "❌ Token tidak boleh kosong."
  exit 1
fi

echo "📤 Mengunggah crate ke crates.io..."
cargo publish --token "$CRATES_IO_TOKEN"
