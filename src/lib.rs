use hex::decode;

pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, String> {
    // TODO: Decode hex string into Vec<u8>, return error string on failure
    decode(hex_str).map_err(|e| e.to_string())
}

pub fn to_big_endian(bytes: &[u8]) -> Vec<u8> {
    // TODO: Reverse the byte order of input slice and return as Vec<u8>
    bytes.iter().rev().copied().collect()
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    // TODO: Implement conversion of bytes slice to hex string
    //encode(bytes)
    bytes.iter().map(|f| format!("{:02x}", f)).collect()
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
    // TODO: Implement conversion of hex string to bytes vector
    decode(hex)
}

pub fn swap_endian_u32(num: u32) -> [u8; 4] {
    // TODO: Implement little-endian byte swap for u32
    let mut version = num.to_be_bytes();
    version.reverse();
    version
}

pub fn parse_satoshis(input: &str) -> Result<u64, String> {
    // TODO: Parse input string to u64, return error string if invalid
    input
        .parse()
        .map_err(|_| "Invalid satoshi amount".to_string())
}

#[derive(PartialEq, Eq)]
pub enum ScriptType {
    P2PKH,
    P2WPKH,
    Unknown,
}

pub fn classify_script(script: &[u8]) -> ScriptType {
    // TODO: Match script pattern and return corresponding ScriptType
    // P2PKH: 76 a9 14 [20 bytes] 88 ac (25 bytes total)
    match script {
        [0x76, 0xa9, 0x14] => ScriptType::P2PKH,

        // P2WPKH: 00 14 [20 bytes] (22 bytes total)
        [0x00, 0x14, 0xff] => ScriptType::P2WPKH,

        // Tout autre pattern
        _ => ScriptType::Unknown,
    }
}

// TODO: complete Outpoint tuple struct
pub struct Outpoint(pub String, pub u32);

pub fn read_pushdata(script: &[u8]) -> &[u8] {
    // TODO: Return the pushdata portion of the script slice (assumes pushdata starts at index 2)
    // Version simple : assume que les données commencent à l'index 2
    // (après opcode + longueur)
    if script.len() <= 2 {
        return &[];
    }
    &script[2..]
}

pub trait Wallet {
    fn balance(&self) -> u64;
}

pub struct TestWallet {
    pub confirmed: u64,
}

impl Wallet for TestWallet {
    fn balance(&self) -> u64 {
        // TODO: Return the wallet's confirmed balance
        self.confirmed
    }
}

pub fn apply_fee(balance: &mut u64, fee: u64) {
    // TODO: Subtract fee from mutable balance reference
    let diff = *balance - fee;
    *balance = diff;
}

pub fn move_txid(txid: String) -> String {
    // TODO: Return formatted string including the txid for display or logging
    format!("txid: {}", txid)
}

// TODO: Add necessary derive traits
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    OpChecksig,
    OpDup,
    OpInvalid,
}

impl Opcode {
    pub fn from_byte(byte: u8) -> Result<Self, String> {
        // TODO: Implement mapping from byte to Opcode variant
        match byte {
            0x76 => Ok(Opcode::OpDup),
            0xac => Ok(Opcode::OpChecksig),
            _ => Err("Invalid opcode: 0x00".to_string()),
        }
    }
}

// TODO: Add necessary derive traits

pub trait UTXOfunc {
    fn depense(&self) -> Self;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UTXO {
    pub txid: Vec<u8>,
    pub vout: u32,
    pub value: u64,
}

impl UTXOfunc for UTXO {
    fn depense(&self) -> Self {
        UTXO {
            txid: self.txid.clone(),
            vout: self.vout,
            value: self.value,
        }
    }
}

pub fn consume_utxo(utxo: UTXO) -> UTXO {
    // TODO: Implement UTXO consumption logic (if any)
    utxo.depense()
}
