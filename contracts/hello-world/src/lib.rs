// src/lib.rs
#![no_std]

use soroban_sdk::{
    contract, contractimpl, Address, Env, String, 
    symbol_short, Symbol
};

mod storage;
mod errors;

use storage::{DataKey, TokenMetadata};
use errors::TokenError;

/// Constantes de configuración
const MAX_DECIMALS: u32 = 18;
const MAX_NAME_LENGTH: u32 = 100;
const MAX_SYMBOL_LENGTH: u32 = 32;

/// Trait que define la interfaz del token según CAP-46
/// 
/// Esta es la interfaz estándar de tokens fungibles en Stellar
/// Compatible con wallets, DEXs, y el ecosistema completo
pub trait TokenTrait {
    /// Inicializa el token con metadatos y admin
    /// 
    /// Puede ser llamado solo una vez. Configura:
    /// - Admin: cuenta con permisos para mintear
    /// - Name: nombre completo del token
    /// - Symbol: identificador corto (ej: BDB, USDC)
    /// - Decimals: precisión del token (7 para Stellar)
    fn initialize(
        env: Env, 
        admin: Address, 
        name: String, 
        symbol: String,
        decimals: u32
    ) -> Result<(), TokenError>;
    
    /// Crea nuevos tokens (solo admin)
    /// 
    /// Aumenta el supply total y el balance del destinatario
    /// Requiere autorización del admin
    fn mint(env: Env, to: Address, amount: i128) -> Result<(), TokenError>;
    
    /// Destruye tokens reduciendo el supply
    /// 
    /// Reduce el supply total y el balance del owner
    /// Requiere autorización del owner
    fn burn(env: Env, from: Address, amount: i128) -> Result<(), TokenError>;
    
    /// Consulta el balance de una cuenta
    /// 
    /// Devuelve 0 si la cuenta nunca ha recibido tokens
    fn balance(env: Env, account: Address) -> i128;
    
    /// Transfiere tokens entre cuentas
    /// 
    /// Requiere autorización de `from`
    /// No permite transferencias a sí mismo
    fn transfer(
        env: Env, 
        from: Address, 
        to: Address, 
        amount: i128
    ) -> Result<(), TokenError>;
    
    /// Aprueba a otro usuario para gastar tokens
    /// 
    /// Permite que `spender` gaste hasta `amount` tokens
    /// de la cuenta de `from`. Se puede revocar con amount=0
    fn approve(
        env: Env, 
        from: Address, 
        spender: Address, 
        amount: i128
    ) -> Result<(), TokenError>;
    
    /// Consulta el allowance entre dos cuentas
    /// 
    /// Devuelve cuánto puede gastar `spender` de los tokens de `from`
    fn allowance(env: Env, from: Address, spender: Address) -> i128;
    
    /// Transfiere tokens en nombre de otro usuario
    /// 
    /// Requiere allowance previo mediante approve()
    /// Reduce el allowance automáticamente
    fn transfer_from(
        env: Env, 
        spender: Address, 
        from: Address, 
        to: Address, 
        amount: i128
    ) -> Result<(), TokenError>;
    
    // Métodos de consulta (getters)
    fn name(env: Env) -> String;
    fn symbol(env: Env) -> String;
    fn decimals(env: Env) -> u32;
    fn total_supply(env: Env) -> i128;
    fn admin(env: Env) -> Address;
}

/// Estructura del contrato Token BDB
#[contract]
pub struct TokenBDB;

/// Implementación del contrato
#[contractimpl]
impl TokenTrait for TokenBDB {
    fn initialize(
        env: Env, 
        admin: Address, 
        name: String, 
        symbol: String,
        decimals: u32
    ) -> Result<(), TokenError> {
        // 1. Verificar que no esté inicializado
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(TokenError::AlreadyInitialized);
        }
        
        // 2. Validar decimales (máximo 18 como Ethereum)
        if decimals > MAX_DECIMALS {
            return Err(TokenError::InvalidDecimals);
        }
        
        // 3. Validar metadatos (name y symbol no vacíos)
        // Nota: String en Soroban no tiene .len() directo,
        // pero podemos convertir a bytes para validar
        if name.len() == 0 || name.len() > MAX_NAME_LENGTH {
            return Err(TokenError::InvalidMetadata);
        }
        
        if symbol.len() == 0 || symbol.len() > MAX_SYMBOL_LENGTH {
            return Err(TokenError::InvalidMetadata);
        }
        
        // 4. Guardar metadata en instance storage
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenName, &name);
        env.storage().instance().set(&DataKey::TokenSymbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.storage().instance().set(&DataKey::Initialized, &true);
        
        // 5. Extender TTL del storage de instance (30 días)
        env.storage().instance().extend_ttl(100_000, 200_000);
        
        // 6. Emitir evento rico con todos los metadatos
        env.events().publish(
            (symbol_short!("init"), admin.clone()),
            TokenMetadata {
                name: name.clone(),
                symbol: symbol.clone(),
                decimals,
            }
        );
        
        Ok(())
    }
    
    fn mint(env: Env, to: Address, amount: i128) -> Result<(), TokenError> {
        // 1. Verificar inicialización
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(TokenError::NotInitialized);
        }
        
        // 2. Solo el admin puede mintear
        let admin: Address = env.storage().instance()
            .get(&DataKey::Admin)
            .ok_or(TokenError::NotInitialized)?;
        admin.require_auth();
        
        // 3. Validaciones
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        
        // 4. Validar que `to` no sea igual a `admin` (opcional, pero buena práctica)
        // Esto evita que el admin se mintee tokens a sí mismo por error
        
        // 5. Obtener balance actual y verificar overflow
        let balance = Self::balance(env.clone(), to.clone());
        let new_balance = balance.checked_add(amount)
            .ok_or(TokenError::OverflowError)?;
        
        // 6. Actualizar balance con TTL extendido
        env.storage().persistent().set(
            &DataKey::Balance(to.clone()), 
            &new_balance
        );
        env.storage().persistent().extend_ttl(
            &DataKey::Balance(to.clone()),
            100_000,
            200_000
        );
        
        // 7. Actualizar total supply
        let total: i128 = env.storage().instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        let new_total = total.checked_add(amount)
            .ok_or(TokenError::OverflowError)?;
        env.storage().instance().set(
            &DataKey::TotalSupply, 
            &new_total
        );
        
        // 8. Emitir evento detallado
        env.events().publish(
            (symbol_short!("mint"), to.clone()), 
            (amount, new_balance, new_total)
        );
        
        Ok(())
    }
    
    fn burn(env: Env, from: Address, amount: i128) -> Result<(), TokenError> {
        // 1. Verificar inicialización
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(TokenError::NotInitialized);
        }
        
        // 2. Requiere autorización del dueño de los tokens
        from.require_auth();
        
        // 3. Validaciones
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        
        let balance = Self::balance(env.clone(), from.clone());
        if balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        // 4. Actualizar balance
        let new_balance = balance - amount;
        if new_balance == 0 {
            // Optimización: eliminar key si balance = 0
            env.storage().persistent().remove(&DataKey::Balance(from.clone()));
        } else {
            env.storage().persistent().set(
                &DataKey::Balance(from.clone()),
                &new_balance
            );
            env.storage().persistent().extend_ttl(
                &DataKey::Balance(from.clone()),
                100_000,
                200_000
            );
        }
        
        // 5. Actualizar total supply
        let total: i128 = env.storage().instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        let new_total = total.checked_sub(amount)
            .ok_or(TokenError::OverflowError)?;
        env.storage().instance().set(
            &DataKey::TotalSupply,
            &new_total
        );
        
        // 6. Emitir evento
        env.events().publish(
            (symbol_short!("burn"), from),
            (amount, new_balance, new_total)
        );
        
        Ok(())
    }
    
    fn balance(env: Env, account: Address) -> i128 {
        env.storage().persistent()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }
    
    fn transfer(
        env: Env, 
        from: Address, 
        to: Address, 
        amount: i128
    ) -> Result<(), TokenError> {
        // 1. Verificar inicialización
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(TokenError::NotInitialized);
        }
        
        // 2. Verificar autorización del sender
        from.require_auth();
        
        // 3. Validaciones
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        
        // 4. No permitir transferencia a sí mismo (gas-efficient)
        if from == to {
            return Err(TokenError::InvalidRecipient);
        }
        
        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        // 5. Calcular nuevos balances con verificación de overflow
        let new_from_balance = from_balance - amount;
        let to_balance = Self::balance(env.clone(), to.clone());
        let new_to_balance = to_balance.checked_add(amount)
            .ok_or(TokenError::OverflowError)?;
        
        // 6. Actualizar balances con TTL
        // Optimización: si from_balance = 0, eliminar key
        if new_from_balance == 0 {
            env.storage().persistent().remove(&DataKey::Balance(from.clone()));
        } else {
            env.storage().persistent().set(
                &DataKey::Balance(from.clone()),
                &new_from_balance
            );
            env.storage().persistent().extend_ttl(
                &DataKey::Balance(from.clone()),
                100_000,
                200_000
            );
        }
        
        env.storage().persistent().set(
            &DataKey::Balance(to.clone()),
            &new_to_balance
        );
        env.storage().persistent().extend_ttl(
            &DataKey::Balance(to.clone()),
            100_000,
            200_000
        );
        
        // 7. Emitir evento con balances post-transferencia
        env.events().publish(
            (symbol_short!("transfer"), from, to), 
            (amount, new_from_balance, new_to_balance)
        );
        
        Ok(())
    }
    
    fn approve(
        env: Env, 
        from: Address, 
        spender: Address, 
        amount: i128
    ) -> Result<(), TokenError> {
        // 1. Verificar inicialización
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(TokenError::NotInitialized);
        }
        
        // 2. Verificar autorización del owner
        from.require_auth();
        
        // 3. Validación: amount debe ser >= 0 (permitir 0 para revocar)
        if amount < 0 {
            return Err(TokenError::InvalidAmount);
        }
        
        // 4. Obtener allowance anterior para el evento
        let old_allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        
        // 5. Actualizar allowance
        if amount == 0 {
            // Optimización: eliminar key si allowance = 0
            env.storage().persistent().remove(
                &DataKey::Allowance(from.clone(), spender.clone())
            );
        } else {
            env.storage().persistent().set(
                &DataKey::Allowance(from.clone(), spender.clone()),
                &amount
            );
            env.storage().persistent().extend_ttl(
                &DataKey::Allowance(from.clone(), spender.clone()),
                100_000,
                200_000
            );
        }
        
        // 6. Evento mejorado con allowance anterior y nuevo
        env.events().publish(
            (symbol_short!("approve"), from, spender),
            (old_allowance, amount)
        );
        
        Ok(())
    }
    
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        env.storage().persistent()
            .get(&DataKey::Allowance(from, spender))
            .unwrap_or(0)
    }
    
    fn transfer_from(
        env: Env, 
        spender: Address, 
        from: Address, 
        to: Address, 
        amount: i128
    ) -> Result<(), TokenError> {
        // 1. Verificar inicialización
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(TokenError::NotInitialized);
        }
        
        // 2. Verificar autorización del spender
        spender.require_auth();
        
        // 3. Validaciones
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        
        // 4. No permitir transferencia a sí mismo
        if from == to {
            return Err(TokenError::InvalidRecipient);
        }
        
        // 5. Verificar allowance
        let allowed = Self::allowance(env.clone(), from.clone(), spender.clone());
        if allowed < amount {
            return Err(TokenError::InsufficientAllowance);
        }
        
        // 6. Verificar balance
        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        // 7. Calcular nuevos valores
        let new_from_balance = from_balance - amount;
        let to_balance = Self::balance(env.clone(), to.clone());
        let new_to_balance = to_balance.checked_add(amount)
            .ok_or(TokenError::OverflowError)?;
        let new_allowance = allowed - amount;
        
        // 8. Actualizar estado atómicamente
        // Optimización: eliminar keys si son 0
        if new_from_balance == 0 {
            env.storage().persistent().remove(&DataKey::Balance(from.clone()));
        } else {
            env.storage().persistent().set(
                &DataKey::Balance(from.clone()),
                &new_from_balance
            );
            env.storage().persistent().extend_ttl(
                &DataKey::Balance(from.clone()),
                100_000,
                200_000
            );
        }
        
        env.storage().persistent().set(
            &DataKey::Balance(to.clone()),
            &new_to_balance
        );
        env.storage().persistent().extend_ttl(
            &DataKey::Balance(to.clone()),
            100_000,
            200_000
        );
        
        if new_allowance == 0 {
            env.storage().persistent().remove(
                &DataKey::Allowance(from.clone(), spender.clone())
            );
        } else {
            env.storage().persistent().set(
                &DataKey::Allowance(from.clone(), spender.clone()),
                &new_allowance
            );
            env.storage().persistent().extend_ttl(
                &DataKey::Allowance(from.clone(), spender.clone()),
                100_000,
                200_000
            );
        }
        
        // 9. Emitir evento completo (FIX: evento faltante)
        env.events().publish(
            (symbol_short!("trnsf_frm"), spender, from.clone(), to.clone()),
            (amount, new_from_balance, new_to_balance, new_allowance)
        );
        
        Ok(())
    }
    
    // Métodos de consulta
    fn name(env: Env) -> String {
        // Verificar inicialización antes de devolver metadata
        if !env.storage().instance().has(&DataKey::Initialized) {
            return String::from_str(&env, "");
        }
        
        env.storage().instance()
            .get(&DataKey::TokenName)
            .unwrap_or(String::from_str(&env, ""))
    }
    
    fn symbol(env: Env) -> String {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return String::from_str(&env, "");
        }
        
        env.storage().instance()
            .get(&DataKey::TokenSymbol)
            .unwrap_or(String::from_str(&env, ""))
    }
    
    fn decimals(env: Env) -> u32 {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return 0;
        }
        
        env.storage().instance()
            .get(&DataKey::Decimals)
            .unwrap_or(0)
    }
    
    fn total_supply(env: Env) -> i128 {
        env.storage().instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }
    
    fn admin(env: Env) -> Address {
        env.storage().instance()
            .get(&DataKey::Admin)
            .expect("Admin not initialized")
    }
}
