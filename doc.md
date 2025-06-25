# Cours Complet : Programmation Bitcoin en Rust

## Table des Matières
1. [Introduction à la programmation Bitcoin](#introduction-bitcoin)
2. [Satoshis et gestion des montants](#satoshis)
3. [Hexadécimal et sérialisation Bitcoin](#hexadecimal-bitcoin)
4. [Endianness dans Bitcoin](#endianness-bitcoin)
5. [Scripts Bitcoin et opcodes](#scripts-opcodes)
6. [UTXO et Outpoints](#utxo-outpoints)
7. [Wallets et gestion des frais](#wallets-fees)
8. [Implémentations complètes](#implementations-completes)

## 1. Introduction à la programmation Bitcoin {#introduction-bitcoin}

Bitcoin utilise intensivement les données binaires, l'hexadécimal, et des structures de données spécifiques. Ce cours couvre les concepts essentiels pour développer des applications Bitcoin en Rust.

### Concepts Bitcoin fondamentaux :
- **Satoshi** : Plus petite unité Bitcoin (1 BTC = 100,000,000 satoshis)
- **TXID** : Identifiant unique d'une transaction (hash SHA256 double)
- **UTXO** : Unspent Transaction Output (sortie de transaction non dépensée)
- **Script** : Programme qui détermine les conditions de dépense
- **Endianness** : Bitcoin utilise little-endian pour les nombres, big-endian pour les hashs

## 2. Satoshis et gestion des montants {#satoshis}

Bitcoin travaille exclusivement en satoshis pour éviter les erreurs de virgule flottante.

### Parsing des montants Bitcoin :

```rust
pub fn parse_satoshis(input: &str) -> Result<u64, String> {
    // Version de base
    input.parse::<u64>()
        .map_err(|e| format!("Montant satoshi invalide: {}", e))
}

// Version avec support des unités Bitcoin
pub fn parse_bitcoin_amount(input: &str) -> Result<u64, String> {
    let trimmed = input.trim().to_lowercase();
    
    if let Some(btc_str) = trimmed.strip_suffix(" btc") {
        // Conversion BTC -> satoshis
        let btc: f64 = btc_str.trim().parse()
            .map_err(|_| "Format BTC invalide")?;
        
        if btc < 0.0 || btc > 21_000_000.0 {
            return Err("Montant BTC hors limites".to_string());
        }
        
        Ok((btc * 100_000_000.0) as u64)
        
    } else if let Some(sat_str) = trimmed.strip_suffix(" sat") {
        // Direct en satoshis
        let sats: u64 = sat_str.trim().parse()
            .map_err(|_| "Format satoshi invalide")?;
            
        if sats > 2_100_000_000_000_000 { // 21M BTC en sats
            return Err("Trop de satoshis".to_string());
        }
        
        Ok(sats)
        
    } else {
        // Par défaut, assume satoshis
        let sats: u64 = trimmed.parse()
            .map_err(|_| "Format numérique invalide")?;
        Ok(sats)
    }
}

// Conversion inverse satoshis -> BTC
pub fn satoshis_to_btc(sats: u64) -> f64 {
    sats as f64 / 100_000_000.0
}

// Formatage pour affichage
pub fn format_bitcoin_amount(sats: u64) -> String {
    if sats >= 100_000_000 {
        format!("{:.8} BTC", satoshis_to_btc(sats))
    } else if sats >= 1000 {
        format!("{} sats", sats)
    } else {
        format!("{} sat", sats)
    }
}
```

## 3. Hexadécimal et sérialisation Bitcoin {#hexadecimal-bitcoin}

Bitcoin sérialise toutes ses données en binaire, souvent représentées en hexadécimal.

### Conversion hex/bytes pour Bitcoin :

```rust
use hex::{decode, encode};

// Décodage hex avec validation Bitcoin
pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, String> {
    // Validation longueur paire (requis pour hex valide)
    if hex_str.len() % 2 != 0 {
        return Err("Hex Bitcoin doit avoir une longueur paire".to_string());
    }
    
    // Décodage
    decode(hex_str).map_err(|e| format!("Hex invalide: {}", e))
}

// Encodage bytes -> hex (toujours minuscules en Bitcoin)
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    encode(bytes).to_lowercase()
}

// Conversion spécifique pour TXID (32 bytes requis)
pub fn hex_to_txid(hex: &str) -> Result<[u8; 32], String> {
    let bytes = decode_hex(hex)?;
    
    if bytes.len() != 32 {
        return Err(format!("TXID doit faire 32 bytes, reçu: {}", bytes.len()));
    }
    
    let mut txid = [0u8; 32];
    txid.copy_from_slice(&bytes);
    Ok(txid)
}

// Conversion pour clés publiques (33 bytes compressées, 65 non-compressées)
pub fn hex_to_pubkey(hex: &str) -> Result<Vec<u8>, String> {
    let bytes = decode_hex(hex)?;
    
    match bytes.len() {
        33 => {
            // Clé compressée (préfixe 02 ou 03)
            if bytes[0] != 0x02 && bytes[0] != 0x03 {
                return Err("Préfixe de clé compressée invalide".to_string());
            }
            Ok(bytes)
        },
        65 => {
            // Clé non-compressée (préfixe 04)
            if bytes[0] != 0x04 {
                return Err("Préfixe de clé non-compressée invalide".to_string());
            }
            Ok(bytes)
        },
        _ => Err(format!("Taille de clé publique invalide: {}", bytes.len()))
    }
}
```

## 4. Endianness dans Bitcoin {#endianness-bitcoin}

Bitcoin utilise different endianness selon le contexte :
- **Little-endian** : Pour les nombres (montants, indices)
- **Big-endian** : Pour les hashs (TXID, block hash)

### Gestion de l'endianness Bitcoin :

```rust
// Conversion big-endian (pour hashs Bitcoin)
pub fn to_big_endian(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().rev().copied().collect()
}

// Conversion little-endian pour u32 (indices UTXO, timestamps)
pub fn swap_endian_u32(num: u32) -> [u8; 4] {
    num.to_le_bytes()  // Bitcoin utilise little-endian pour les nombres
}

// Lecture d'un u32 little-endian depuis bytes
pub fn read_u32_le(bytes: &[u8]) -> Result<u32, String> {
    if bytes.len() < 4 {
        return Err("Pas assez de bytes pour u32".to_string());
    }
    
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

// Lecture d'un u64 little-endian (montants Bitcoin)
pub fn read_u64_le(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() < 8 {
        return Err("Pas assez de bytes pour u64".to_string());
    }
    
    let mut array = [0u8; 8];
    array.copy_from_slice(&bytes[0..8]);
    Ok(u64::from_le_bytes(array))
}

// Conversion TXID : hex -> bytes -> reverse pour affichage
pub fn format_txid_display(txid_hex: &str) -> Result<String, String> {
    let bytes = decode_hex(txid_hex)?;
    let reversed = to_big_endian(&bytes);
    Ok(bytes_to_hex(&reversed))
}
```

## 5. Scripts Bitcoin et opcodes {#scripts-opcodes}

Les scripts Bitcoin définissent les conditions de dépense des UTXO.

### Types de scripts courants :

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    P2PKH,      // Pay to Public Key Hash (legacy)
    P2WPKH,     // Pay to Witness Public Key Hash (SegWit v0)
    P2SH,       // Pay to Script Hash
    P2WSH,      // Pay to Witness Script Hash
    P2TR,       // Pay to Taproot (SegWit v1)
    Unknown,
}

pub fn classify_script(script: &[u8]) -> ScriptType {
    match script {
        // P2PKH: OP_DUP OP_HASH160 <20 bytes pubkey hash> OP_EQUALVERIFY OP_CHECKSIG
        [0x76, 0xa9, 0x14, .., 0x88, 0xac] if script.len() == 25 => {
            ScriptType::P2PKH
        },
        
        // P2SH: OP_HASH160 <20 bytes script hash> OP_EQUAL
        [0xa9, 0x14, .., 0x87] if script.len() == 23 => {
            ScriptType::P2SH
        },
        
        // P2WPKH: OP_0 <20 bytes pubkey hash>
        [0x00, 0x14, ..] if script.len() == 22 => {
            ScriptType::P2WPKH
        },
        
        // P2WSH: OP_0 <32 bytes script hash>
        [0x00, 0x20, ..] if script.len() == 34 => {
            ScriptType::P2WSH
        },
        
        // P2TR: OP_1 <32 bytes taproot output>
        [0x51, 0x20, ..] if script.len() == 34 => {
            ScriptType::P2TR
        },
        
        _ => ScriptType::Unknown,
    }
}

// Opcodes Bitcoin courants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Stack operations
    OpDup = 0x76,           // Duplicate top stack item
    
    // Crypto operations  
    OpHash160 = 0xa9,       // SHA256 + RIPEMD160
    OpChecksig = 0xac,      // Verify signature
    
    // Comparison
    OpEqual = 0x87,         // Return 1 if inputs are equal
    OpEqualverify = 0x88,   // OpEqual + OpVerify
    
    // Constants
    Op0 = 0x00,            // Push empty array
    Op1 = 0x51,            // Push 1
    
    OpInvalid = 0xff,      // Invalid opcode
}

impl Opcode {
    pub fn from_byte(byte: u8) -> Result<Self, String> {
        match byte {
            0x76 => Ok(Opcode::OpDup),
            0xa9 => Ok(Opcode::OpHash160), 
            0xac => Ok(Opcode::OpChecksig),
            0x87 => Ok(Opcode::OpEqual),
            0x88 => Ok(Opcode::OpEqualverify),
            0x00 => Ok(Opcode::Op0),
            0x51 => Ok(Opcode::Op1),
            _ => Err(format!("Opcode Bitcoin inconnu: 0x{:02x}", byte))
        }
    }
    
    pub fn to_byte(self) -> u8 {
        self as u8
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Opcode::OpDup => "OP_DUP",
            Opcode::OpHash160 => "OP_HASH160",
            Opcode::OpChecksig => "OP_CHECKSIG",
            Opcode::OpEqual => "OP_EQUAL",
            Opcode::OpEqualverify => "OP_EQUALVERIFY",
            Opcode::Op0 => "OP_0",
            Opcode::Op1 => "OP_1", 
            Opcode::OpInvalid => "OP_INVALID",
        }
    }
}

// Lecture des données push dans un script
pub fn read_pushdata(script: &[u8]) -> &[u8] {
    // Version simple : assume que les données commencent à l'index 2
    // (après opcode + longueur)
    if script.len() <= 2 {
        return &[];
    }
    &script[2..]
}

// Version complète avec analyse des opcodes push
pub fn extract_pushdata(script: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < script.len() {
        let opcode = script[i];
        
        match opcode {
            // Push direct (1-75 bytes)
            1..=75 => {
                let len = opcode as usize;
                if i + 1 + len > script.len() {
                    return Err("Script tronqué".to_string());
                }
                result.push(script[i + 1..i + 1 + len].to_vec());
                i += 1 + len;
            },
            
            // OP_PUSHDATA1
            76 => {
                if i + 2 > script.len() {
                    return Err("OP_PUSHDATA1 incomplet".to_string());
                }
                let len = script[i + 1] as usize;
                if i + 2 + len > script.len() {
                    return Err("Données PUSHDATA1 manquantes".to_string());
                }
                result.push(script[i + 2..i + 2 + len].to_vec());
                i += 2 + len;
            },
            
            // Autres opcodes (non-push)
            _ => {
                i += 1;
            }
        }
    }
    
    Ok(result)
}
```

## 6. UTXO et Outpoints {#utxo-outpoints}

Les UTXO sont les "pièces" non dépensées dans Bitcoin. Chaque UTXO est identifié par un Outpoint.

### Structures Bitcoin fondamentales :

```rust
// Outpoint : référence unique à une sortie de transaction
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub txid: [u8; 32],    // Hash de la transaction
    pub vout: u32,         // Index de la sortie (little-endian)
}

impl Outpoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        Self { txid, vout }
    }
    
    // Création depuis hex string
    pub fn from_hex(hex_txid: &str, vout: u32) -> Result<Self, String> {
        let txid = hex_to_txid(hex_txid)?;
        Ok(Self::new(txid, vout))
    }
    
    // Sérialisation Bitcoin (32 bytes + 4 bytes little-endian)
    pub fn serialize(&self) -> [u8; 36] {
        let mut result = [0u8; 36];
        result[0..32].copy_from_slice(&self.txid);
        result[32..36].copy_from_slice(&self.vout.to_le_bytes());
        result
    }
    
    // Désérialisation
    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 36 {
            return Err("Outpoint doit faire 36 bytes".to_string());
        }
        
        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes[0..32]);
        
        let vout = u32::from_le_bytes([
            bytes[32], bytes[33], bytes[34], bytes[35]
        ]);
        
        Ok(Self::new(txid, vout))
    }
    
    // Affichage lisible (TXID en big-endian)
    pub fn display(&self) -> String {
        let reversed_txid = to_big_endian(&self.txid);
        format!("{}:{}", bytes_to_hex(&reversed_txid), self.vout)
    }
}

// UTXO : Unspent Transaction Output
#[derive(Debug, Clone)]
pub struct UTXO {
    pub txid: Vec<u8>,       // Hash de transaction (32 bytes)
    pub vout: u32,           // Index de sortie
    pub value: u64,          // Valeur en satoshis
    pub script_pubkey: Vec<u8>, // Script de sortie
    pub height: Option<u32>, // Hauteur du bloc (optionnel)
}

impl UTXO {
    pub fn new(txid: Vec<u8>, vout: u32, value: u64) -> Self {
        Self {
            txid,
            vout, 
            value,
            script_pubkey: Vec::new(),
            height: None,
        }
    }
    
    // Création avec script
    pub fn with_script(txid: Vec<u8>, vout: u32, value: u64, script: Vec<u8>) -> Self {
        Self {
            txid,
            vout,
            value,
            script_pubkey: script,
            height: None,
        }
    }
    
    // Conversion vers Outpoint
    pub fn outpoint(&self) -> Result<Outpoint, String> {
        if self.txid.len() != 32 {
            return Err("TXID doit faire 32 bytes".to_string());
        }
        
        let mut txid_array = [0u8; 32];
        txid_array.copy_from_slice(&self.txid);
        
        Ok(Outpoint::new(txid_array, self.vout))
    }
    
    // Classification du type de script
    pub fn script_type(&self) -> ScriptType {
        classify_script(&self.script_pubkey)
    }
    
    // Vérification si confirmé (a une hauteur de bloc)
    pub fn is_confirmed(&self) -> bool {
        self.height.is_some()
    }
}

// Consommation d'UTXO (marquage comme dépensé)
pub fn consume_utxo(utxo: UTXO) -> UTXO {
    UTXO {
        txid: utxo.txid,
        vout: utxo.vout,
        value: 0,  // Valeur à 0 indique "dépensé"
        script_pubkey: utxo.script_pubkey,
        height: utxo.height,
    }
}

// Validation d'UTXO
pub fn validate_utxo(utxo: &UTXO) -> Result<(), String> {
    if utxo.txid.len() != 32 {
        return Err("TXID invalide".to_string());
    }
    
    if utxo.value == 0 {
        return Err("UTXO déjà dépensé".to_string());
    }
    
    if utxo.value > 2_100_000_000_000_000 {
        return Err("Valeur UTXO impossible".to_string());
    }
    
    if utxo.script_pubkey.is_empty() {
        return Err("Script vide".to_string());
    }
    
    Ok(())
}
```

## 7. Wallets et gestion des frais {#wallets-fees}

Les wallets Bitcoin gèrent les UTXO et calculent les frais de transaction.

### Implémentation de wallet Bitcoin :

```rust
pub trait Wallet {
    fn balance(&self) -> u64;
    fn get_utxos(&self) -> &[UTXO];
    fn add_utxo(&mut self, utxo: UTXO) -> Result<(), String>;
    fn estimate_fee(&self, outputs: usize) -> u64;
}

#[derive(Debug)]
pub struct TestWallet {
    pub confirmed: u64,          // Solde confirmé
    pub pending: u64,            // Solde en attente
    pub utxos: Vec<UTXO>,       // Liste des UTXO
    pub fee_rate: u64,          // Frais en sat/byte
}

impl TestWallet {
    pub fn new() -> Self {
        Self {
            confirmed: 0,
            pending: 0,
            utxos: Vec::new(),
            fee_rate: 1, // 1 sat/byte par défaut
        }
    }
    
    pub fn with_fee_rate(fee_rate: u64) -> Self {
        Self {
            confirmed: 0,
            pending: 0, 
            utxos: Vec::new(),
            fee_rate,
        }
    }
    
    // Calcul du solde par type de script
    pub fn balance_by_type(&self, script_type: ScriptType) -> u64 {
        self.utxos.iter()
            .filter(|utxo| utxo.script_type() == script_type)
            .map(|utxo| utxo.value)
            .sum()
    }
    
    // Sélection d'UTXO pour un paiement
    pub fn select_utxos(&self, target: u64) -> Result<Vec<&UTXO>, String> {
        let mut selected = Vec::new();
        let mut total = 0;
        
        // Tri par valeur décroissante (stratégie simple)
        let mut sorted_utxos = self.utxos.iter().collect::<Vec<_>>();
        sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));
        
        for utxo in sorted_utxos {
            if utxo.value == 0 { continue; } // Skip dépensés
            
            selected.push(utxo);
            total += utxo.value;
            
            if total >= target {
                return Ok(selected);
            }
        }
        
        Err("Fonds insuffisants".to_string())
    }
}

impl Wallet for TestWallet {
    fn balance(&self) -> u64 {
        self.confirmed
    }
    
    fn get_utxos(&self) -> &[UTXO] {
        &self.utxos
    }
    
    fn add_utxo(&mut self, utxo: UTXO) -> Result<(), String> {
        validate_utxo(&utxo)?;
        
        if utxo.is_confirmed() {
            self.confirmed += utxo.value;
        } else {
            self.pending += utxo.value;
        }
        
        self.utxos.push(utxo);
        Ok(())
    }
    
    // Estimation des frais basée sur la taille de transaction
    fn estimate_fee(&self, outputs: usize) -> u64 {
        // Estimation simple :
        // - Base : 10 bytes (version, locktime, etc.)
        // - Input : ~150 bytes chacun (P2PKH)
        // - Output : ~35 bytes chacun
        
        let inputs = 1; // Simplifié
        let tx_size = 10 + (inputs * 150) + (outputs * 35);
        
        (tx_size as u64) * self.fee_rate
    }
}

// Application des frais
pub fn apply_fee(balance: &mut u64, fee: u64) {
    if *balance >= fee {
        *balance -= fee;
    } else {
        *balance = 0;
    }
}

// Version sécurisée avec vérification
pub fn apply_fee_safe(balance: &mut u64, fee: u64) -> Result<(), String> {
    if *balance < fee {
        return Err(format!("Solde insuffisant: {} < {}", balance, fee));
    }
    
    *balance -= fee;
    Ok(())
}

// Calcul des frais de transaction réels
pub fn calculate_tx_fee(inputs: usize, outputs: usize, fee_rate: u64) -> u64 {
    // Tailles approximatives des composants de transaction
    let base_size = 10;  // Version (4) + locktime (4) + compteurs (2)
    
    let input_size = match inputs {
        0 => 0,
        _ => inputs * 150,  // Outpoint (36) + script (110) + sequence (4)
    };
    
    let output_size = outputs * 35;  // Value (8) + script length (1) + script (26)
    
    let total_size = base_size + input_size + output_size;
    
    (total_size as u64) * fee_rate
}

// Formatage des transactions
pub fn move_txid(txid: String) -> String {
    format!("Transaction: {}", txid)
}

pub fn format_transaction_summary(txid: &str, inputs: usize, outputs: usize, fee: u64) -> String {
    format!(
        "TX {} - Inputs: {}, Outputs: {}, Fee: {} sats ({})", 
        &txid[0..8], 
        inputs, 
        outputs, 
        fee,
        format_bitcoin_amount(fee)
    )
}
```

## 8. Implémentations complètes {#implementations-completes}

### Exemple d'utilisation complète :

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Création d'un wallet
    let mut wallet = TestWallet::with_fee_rate(5); // 5 sat/byte
    
    // Ajout d'UTXO depuis une transaction
    let txid_hex = "a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890";
    let utxo = UTXO::with_script(
        decode_hex(txid_hex)?,
        0,
        50000, // 0.0005 BTC
        // Script P2PKH 
        vec![0x76, 0xa9, 0x14, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x88, 0xac]
    );
    
    wallet.add_utxo(utxo)?;
    
    println!("Solde wallet: {} ({})", 
        wallet.balance(), 
        format_bitcoin_amount(wallet.balance())
    );
    
    // Classification du script
    let script_type = wallet.get_utxos()[0].script_type();
    println!("Type de script: {:?}", script_type);
    
    // Calcul des frais pour une transaction
    let fee = wallet.estimate_fee(2); // 2 outputs
    println!("Frais estimés: {} sats", fee);
    
    // Application des frais
    let mut balance = wallet.balance();
    apply_fee_safe(&mut balance, fee)?;
    println!("Solde après frais: {} sats", balance);
    
    // Parsing d'un montant Bitcoin
    let amount = parse_bitcoin_amount("0.001 btc")?;
    println!("Montant parsé: {} satoshis", amount);
    
    // Sélection d'UTXO pour paiement
    let target = 30000; // 0.0003 BTC
    match wallet.select_utxos(target + fee) {
        Ok(selected) => {
            println!("UTXO sélectionnés: {} pour payer {} sats", 
                selected.len(), target);
        },
        Err(e) => println!("Erreur sélection: {}", e),
    }
    
    Ok(())
}

// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_satoshi_conversion() {
        assert_eq!(parse_bitcoin_amount("1 btc").unwrap(), 100_000_000);
        assert_eq!(parse_bitcoin_amount("1000 sat").unwrap(), 1000);
        assert_eq!(satoshis_to_btc(100_000_000), 1.0);
    }
    
    #[test] 
    fn test_script_classification() {
        // Script P2PKH
        let p2pkh = [0x76, 0xa9, 0x14, /* 20 bytes hash */, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xac];
        assert_eq!(classify_script(&p2pkh), ScriptType::P2PKH);
        
        // Script P2WPKH  
        let p2wpkh = [0x00, 0x14, /* 20 bytes hash */];
        let mut full_p2wpkh = vec![0x00, 0x14];
        full_p2wpkh.extend_from_slice(&[0u8; 20]);
        assert_eq!(classify_script(&full_p2wpkh), ScriptType::P2WPKH);
    }
    
    #[test]
    fn test_outpoint() {
        let txid = [1u8; 32];
        let outpoint = Outpoint::new(txid, 0);
        
        // Test sérialisation/désérialisation
        let serialized = outpoint.serialize();
        let deserialized = Outpoint::deserialize(&serialized).unwrap();
        assert_eq!(outpoint, deserialized);
        
        // Test affichage
        let display = outpoint.display();
        assert!(!display.is_empty());
    }
    
    #[test]
    fn test_fee_calculation() {
        let fee = calculate_tx_fee(2, 2, 10); // 2 inputs, 2 outputs, 10 sat/byte
        assert!(fee > 0);
        
        let mut balance = 100000u64;
        apply_fee_safe(&mut balance, fee).unwrap();
        assert_eq!(balance, 100000 - fee);
    }
    
    #[test]
    fn test_wallet_operations() {
        let mut wallet = TestWallet::new();
        let utxo = UTXO::new(vec![0u8; 32], 0, 50000);
        
        wallet.add_utxo(utxo).unwrap();
        assert_eq!(wallet.balance(), 50000);
        
        let selected = wallet.select_utxos(30000).unwrap();
        assert_eq!(selected.len(), 1);
    }
}
```

## Concepts avancés Bitcoin

### 1. Gestion des clés et adresses

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Génération d'adresse Bitcoin à partir d'une clé publique
pub fn pubkey_to_p2pkh_address(pubkey: &[u8]) -> Result<String, String> {
    if pubkey.len() != 33 && pubkey.len() != 65 {
        return Err("Clé publique invalide".to_string());
    }
    
    // Simulation du hash160 (SHA256 + RIPEMD160)
    let mut hasher = DefaultHasher::new();
    pubkey.hash(&mut hasher);
    let hash160 = hasher.finish().to_le_bytes();
    
    // Création du script P2PKH
    let mut script = vec![0x76, 0xa9, 0x14]; // OP_DUP OP_HASH160 <20 bytes>
    script.extend_from_slice(&hash160[0..20]);
    script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
    
    Ok(bytes_to_hex(&script))
}

// Validation d'une adresse Bitcoin (version simplifiée)
pub fn validate_bitcoin_address(address: &str) -> Result<ScriptType, String> {
    // Addresses legacy commencent par '1' ou '3'
    if address.starts_with('1') {
        Ok(ScriptType::P2PKH)
    } else if address.starts_with('3') {
        Ok(ScriptType::P2SH)
    } else if address.starts_with("bc1q") {
        Ok(ScriptType::P2WPKH)
    } else if address.starts_with("bc1p") {
        Ok(ScriptType::P2TR)
    } else {
        Err("Format d'adresse invalide".to_string())
    }
}
```

### 2. Construction de transactions

```rust
#[derive(Debug, Clone)]
pub struct TxInput {
    pub outpoint: Outpoint,
    pub script_sig: Vec<u8>,    // Script de signature
    pub sequence: u32,          // Numéro de séquence
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64,             // Valeur en satoshis
    pub script_pubkey: Vec<u8>, // Script de sortie
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            version: 2,
            inputs: Vec::new(),
            outputs: Vec::new(),
            locktime: 0,
        }
    }
    
    // Ajout d'un input depuis un UTXO
    pub fn add_input(&mut self, utxo: &UTXO) -> Result<(), String> {
        let outpoint = utxo.outpoint()?;
        
        let input = TxInput {
            outpoint,
            script_sig: Vec::new(), // Sera rempli lors de la signature
            sequence: 0xffffffff,
        };
        
        self.inputs.push(input);
        Ok(())
    }
    
    // Ajout d'un output
    pub fn add_output(&mut self, value: u64, script: Vec<u8>) {
        let output = TxOutput {
            value,
            script_pubkey: script,
        };
        
        self.outputs.push(output);
    }
    
    // Calcul de la somme des inputs
    pub fn input_value(&self, utxos: &[UTXO]) -> u64 {
        self.inputs.iter()
            .filter_map(|input| {
                utxos.iter().find(|utxo| {
                    utxo.outpoint().map(|op| op == input.outpoint).unwrap_or(false)
                })
            })
            .map(|utxo| utxo.value)
            .sum()
    }
    
    // Calcul de la somme des outputs
    pub fn output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }
    
    // Calcul des frais
    pub fn fee(&self, utxos: &[UTXO]) -> u64 {
        let input_val = self.input_value(utxos);
        let output_val = self.output_value();
        
        if input_val >= output_val {
            input_val - output_val
        } else {
            0
        }
    }
    
    // Validation de la transaction
    pub fn validate(&self, utxos: &[UTXO]) -> Result<(), String> {
        if self.inputs.is_empty() {
            return Err("Transaction sans inputs".to_string());
        }
        
        if self.outputs.is_empty() {
            return Err("Transaction sans outputs".to_string());
        }
        
        let input_val = self.input_value(utxos);
        let output_val = self.output_value();
        
        if input_val < output_val {
            return Err("Inputs insuffisants".to_string());
        }
        
        // Vérification des montants
        for output in &self.outputs {
            if output.value == 0 {
                return Err("Output avec valeur nulle".to_string());
            }
            
            if output.value > 2_100_000_000_000_000 {
                return Err("Output trop important".to_string());
            }
        }
        
        Ok(())
    }
    
    // Sérialisation simplifiée de la transaction
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Version (4 bytes, little-endian)
        result.extend_from_slice(&self.version.to_le_bytes());
        
        // Nombre d'inputs (varint simplifié)
        result.push(self.inputs.len() as u8);
        
        // Inputs
        for input in &self.inputs {
            result.extend_from_slice(&input.outpoint.serialize());
            result.push(input.script_sig.len() as u8);
            result.extend_from_slice(&input.script_sig);
            result.extend_from_slice(&input.sequence.to_le_bytes());
        }
        
        // Nombre d'outputs
        result.push(self.outputs.len() as u8);
        
        // Outputs
        for output in &self.outputs {
            result.extend_from_slice(&output.value.to_le_bytes());
            result.push(output.script_pubkey.len() as u8);
            result.extend_from_slice(&output.script_pubkey);
        }
        
        // Locktime (4 bytes, little-endian)
        result.extend_from_slice(&self.locktime.to_le_bytes());
        
        result
    }
    
    // Calcul du TXID (hash double SHA256)
    pub fn txid(&self) -> [u8; 32] {
        let serialized = self.serialize();
        
        // Simulation du double SHA256
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        let first_hash = hasher.finish();
        
        let mut hasher2 = DefaultHasher::new();
        first_hash.hash(&mut hasher2);
        let second_hash = hasher2.finish();
        
        let mut result = [0u8; 32];
        result[0..8].copy_from_slice(&second_hash.to_le_bytes());
        result
    }
}

// Constructeur de transaction
pub struct TransactionBuilder {
    tx: Transaction,
    utxos: Vec<UTXO>,
    fee_rate: u64,
}

impl TransactionBuilder {
    pub fn new(fee_rate: u64) -> Self {
        Self {
            tx: Transaction::new(),
            utxos: Vec::new(),
            fee_rate,
        }
    }
    
    pub fn add_utxo(&mut self, utxo: UTXO) -> Result<(), String> {
        self.tx.add_input(&utxo)?;
        self.utxos.push(utxo);
        Ok(())
    }
    
    pub fn pay_to_address(&mut self, address: &str, amount: u64) -> Result<(), String> {
        let script_type = validate_bitcoin_address(address)?;
        
        // Création du script selon le type d'adresse
        let script = match script_type {
            ScriptType::P2PKH => {
                // Simulation d'un script P2PKH
                let mut script = vec![0x76, 0xa9, 0x14]; // OP_DUP OP_HASH160
                script.extend_from_slice(&[0u8; 20]); // Hash160 de l'adresse
                script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
                script
            },
            ScriptType::P2WPKH => {
                let mut script = vec![0x00, 0x14]; // OP_0 + 20 bytes
                script.extend_from_slice(&[0u8; 20]);
                script
            },
            _ => return Err("Type d'adresse non supporté".to_string()),
        };
        
        self.tx.add_output(amount, script);
        Ok(())
    }
    
    pub fn build(mut self) -> Result<Transaction, String> {
        // Calcul des frais
        let estimated_size = 250 + (self.tx.inputs.len() * 150) + (self.tx.outputs.len() * 35);
        let fee = (estimated_size as u64) * self.fee_rate;
        
        // Vérification des fonds
        let input_value = self.tx.input_value(&self.utxos);
        let output_value = self.tx.output_value();
        
        if input_value < output_value + fee {
            return Err("Fonds insuffisants pour les frais".to_string());
        }
        
        // Ajout du change si nécessaire
        let change = input_value - output_value - fee;
        if change > 546 { // Dust limit
            // Script de change (retour à l'expéditeur)
            let change_script = vec![0x76, 0xa9, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x88, 0xac];
            self.tx.add_output(change, change_script);
        }
        
        // Validation finale
        self.tx.validate(&self.utxos)?;
        
        Ok(self.tx)
    }
}
```

### 3. Utilitaires avancés

```rust
// Conversion entre différents formats de montants
pub struct BitcoinAmount(pub u64);

impl BitcoinAmount {
    pub fn from_btc(btc: f64) -> Result<Self, String> {
        if btc < 0.0 || btc > 21_000_000.0 {
            return Err("Montant BTC invalide".to_string());
        }
        
        Ok(Self((btc * 100_000_000.0) as u64))
    }
    
    pub fn from_mbtc(mbtc: f64) -> Result<Self, String> {
        Self::from_btc(mbtc / 1000.0)
    }
    
    pub fn from_satoshis(sats: u64) -> Self {
        Self(sats)
    }
    
    pub fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
    
    pub fn to_satoshis(&self) -> u64 {
        self.0
    }
    
    pub fn format(&self) -> String {
        if self.0 >= 100_000_000 {
            format!("{:.8} BTC", self.to_btc())
        } else if self.0 >= 100_000 {
            format!("{:.5} mBTC", self.to_btc() * 1000.0)
        } else {
            format!("{} sats", self.0)
        }
    }
}

// Gestionnaire de mempool simplifié
pub struct Mempool {
    transactions: Vec<Transaction>,
    utxo_spent: std::collections::HashSet<Outpoint>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            utxo_spent: std::collections::HashSet::new(),
        }
    }
    
    pub fn add_transaction(&mut self, tx: Transaction, utxos: &[UTXO]) -> Result<(), String> {
        // Validation
        tx.validate(utxos)?;
        
        // Vérification que les inputs ne sont pas déjà dépensés
        for input in &tx.inputs {
            if self.utxo_spent.contains(&input.outpoint) {
                return Err("Double dépense détectée".to_string());
            }
        }
        
        // Marquer les UTXO comme dépensés
        for input in &tx.inputs {
            self.utxo_spent.insert(input.outpoint.clone());
        }
        
        self.transactions.push(tx);
        Ok(())
    }
    
    pub fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }
    
    pub fn is_spent(&self, outpoint: &Outpoint) -> bool {
        self.utxo_spent.contains(outpoint)
    }
    
    pub fn remove_transaction(&mut self, txid: &[u8; 32]) {
        if let Some(pos) = self.transactions.iter().position(|tx| &tx.txid() == txid) {
            let tx = self.transactions.remove(pos);
            
            // Libérer les UTXO
            for input in &tx.inputs {
                self.utxo_spent.remove(&input.outpoint);
            }
        }
    }
}

// Exemple d'utilisation complète
fn example_bitcoin_transaction() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Exemple de Transaction Bitcoin ===");
    
    // Création d'un wallet avec quelques UTXO
    let mut wallet = TestWallet::with_fee_rate(10); // 10 sat/byte
    
    // Ajout d'UTXO
    let utxo1 = UTXO::with_script(
        vec![1u8; 32],
        0,
        100_000, // 0.001 BTC
        vec![0x76, 0xa9, 0x14, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x88, 0xac]
    );
    
    wallet.add_utxo(utxo1.clone())?;
    
    println!("Solde initial: {}", BitcoinAmount::from_satoshis(wallet.balance()).format());
    
    // Construction d'une transaction
    let mut builder = TransactionBuilder::new(10);
    builder.add_utxo(utxo1)?;
    builder.pay_to_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", 50_000)?;
    
    let tx = builder.build()?;
    
    println!("Transaction créée:");
    println!("- Inputs: {}", tx.inputs.len());
    println!("- Outputs: {}", tx.outputs.len());
    println!("- Frais: {} sats", tx.fee(&[utxo1.clone()]));
    println!("- TXID: {}", bytes_to_hex(&tx.txid()));
    
    // Test du mempool
    let mut mempool = Mempool::new();
    mempool.add_transaction(tx, &[utxo1])?;
    
    println!("Transaction ajoutée au mempool");
    println!("Transactions en attente: {}", mempool.get_transactions().len());
    
    Ok(())
}
```

Ce cours complet couvre maintenant tous les aspects Bitcoin présents dans votre code original, avec des implémentations détaillées et des exemples pratiques. Chaque section explique les concepts spécifiques à Bitcoin et propose plusieurs approches d'implémentation.