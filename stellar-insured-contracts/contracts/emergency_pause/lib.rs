use soroban_sdk::{Vec as SorobanVec};
use core::cmp::min;

const PAUSE_TIMELOCK: Symbol = Symbol::short("PAUSE_TIMELOCK");
const RESUME_VOTES: Symbol = Symbol::short("RESUME_VOTES");
const REQUIRED_VOTES: Symbol = Symbol::short("REQUIRED_VOTES");

#[contracttype]
#[derive(Clone, Debug)]
pub struct RecoveryProcedure {
    pub steps: SorobanVec<Symbol>,
    pub completed: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StagedResumption {
    pub stages: SorobanVec<Symbol>,
    pub current_stage: u32,
    pub total_stages: u32,
    pub started: bool,
}
#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol};

const ADMIN: Symbol = Symbol::short("ADMIN");
const EMERGENCY_PAUSE: Symbol = Symbol::short("EMERGENCY");
const PAUSE_REASON: Symbol = Symbol::short("PAUSE_REASON");
const PAUSE_TIMESTAMP: Symbol = Symbol::short("PAUSE_TIME");
const MAX_DURATION: Symbol = Symbol::short("MAX_DURATION");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum EmergencyPauseError {
    Unauthorized = 1,
    AlreadyPaused = 2,
    NotPaused = 3,
    InvalidDuration = 4,
    NotInitialized = 5,
    DurationExceeded = 6,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyPauseState {
    pub is_paused: bool,
    pub reason: Symbol,
    pub pause_timestamp: u64,
    pub max_duration_seconds: u64,
    pub paused_by: Address,
}

#[contract]
pub struct EmergencyPauseContract;

impl EmergencyPauseContract {
    pub fn initialize(env: &Env, admin: &Address) -> Result<(), EmergencyPauseError> {
        if env.storage().persistent().has(&ADMIN) {
            return Err(EmergencyPauseError::NotInitialized);
        }

        admin.require_auth();
        env.storage().persistent().set(&ADMIN, admin);

        Ok(())
    }

    pub fn activate_emergency_pause(
        env: &Env,
        admin: &Address,
        reason: Symbol,
        max_duration_seconds: u64,
    ) -> Result<(), EmergencyPauseError> {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().persistent()
            .get(&ADMIN)
            .ok_or(EmergencyPauseError::NotInitialized)?;

        if stored_admin != *admin {
            return Err(EmergencyPauseError::Unauthorized);
        }

        if env.storage().persistent().has(&EMERGENCY_PAUSE) {
            return Err(EmergencyPauseError::AlreadyPaused);
        }

        if max_duration_seconds == 0 || max_duration_seconds > 86400 * 30 { // Max 30 days
            return Err(EmergencyPauseError::InvalidDuration);
        }

        // Set timelock for pause (e.g., 10s enforced delay)
        let now = env.ledger().timestamp();
        env.storage().persistent().set(&PAUSE_TIMELOCK, &(now + 10));

        let pause_state = EmergencyPauseState {
            is_paused: true,
            reason,
            pause_timestamp: env.ledger().timestamp(),
            max_duration_seconds,
            paused_by: admin.clone(),
        };

        env.storage().persistent().set(&EMERGENCY_PAUSE, &pause_state);

        // Initialize staged resumption and recovery
        let stages = SorobanVec::from_array(&env, &[Symbol::short("Stage1"), Symbol::short("Stage2"), Symbol::short("Stage3")]);
        let staged = StagedResumption { stages, current_stage: 0, total_stages: 3, started: false };
        env.storage().persistent().set(&Symbol::short("STAGED_RESUME"), &staged);

        let recovery = RecoveryProcedure { steps: SorobanVec::from_array(&env, &[Symbol::short("CheckFunds"), Symbol::short("NotifyUsers"), Symbol::short("Audit")]), completed: false };
        env.storage().persistent().set(&Symbol::short("RECOVERY"), &recovery);

        // Set required votes for emergency governance (e.g., 3)
        env.storage().persistent().set(&REQUIRED_VOTES, &3u32);
        env.storage().persistent().set(&RESUME_VOTES, &SorobanVec::<Address>::new(&env));

        Ok(())
    }

    pub fn deactivate_emergency_pause(env: &Env, admin: &Address) -> Result<(), EmergencyPauseError> {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().persistent()
            .get(&ADMIN)
            .ok_or(EmergencyPauseError::NotInitialized)?;

        if stored_admin != *admin {
            return Err(EmergencyPauseError::Unauthorized);
        }

        if !env.storage().persistent().has(&EMERGENCY_PAUSE) {
            return Err(EmergencyPauseError::NotPaused);
        }

        // Enforce timelock before unpausing
        let now = env.ledger().timestamp();
        let timelock: u64 = env.storage().persistent().get(&PAUSE_TIMELOCK).unwrap_or(0);
        if now < timelock {
            return Err(EmergencyPauseError::DurationExceeded);
        }

        env.storage().persistent().remove(&EMERGENCY_PAUSE);

        // Reset staged resumption and recovery
        env.storage().persistent().remove(&Symbol::short("STAGED_RESUME"));
        env.storage().persistent().remove(&Symbol::short("RECOVERY"));
        env.storage().persistent().remove(&RESUME_VOTES);

        Ok(())
    }

    // Emergency governance voting for resumption
    pub fn vote_resume(env: &Env, voter: &Address) -> Result<u32, EmergencyPauseError> {
        voter.require_auth();
        let mut votes: SorobanVec<Address> = env.storage().persistent().get(&RESUME_VOTES).unwrap_or(SorobanVec::new(&env));
        if votes.contains(voter) {
            return Ok(votes.len());
        }
        votes.push_back(voter.clone());
        env.storage().persistent().set(&RESUME_VOTES, &votes);
        let required: u32 = env.storage().persistent().get(&REQUIRED_VOTES).unwrap_or(3);
        if votes.len() >= required {
            // Allow admin to deactivate pause
            // (actual unpause must still be called by admin)
        }
        Ok(votes.len())
    }

    // Advance staged resumption
    pub fn advance_stage(env: &Env, admin: &Address) -> Result<u32, EmergencyPauseError> {
        admin.require_auth();
        let mut staged: StagedResumption = env.storage().persistent().get(&Symbol::short("STAGED_RESUME")).unwrap();
        if staged.current_stage + 1 < staged.total_stages {
            staged.current_stage += 1;
            staged.started = true;
            env.storage().persistent().set(&Symbol::short("STAGED_RESUME"), &staged);
            Ok(staged.current_stage)
        } else {
            staged.current_stage = staged.total_stages;
            staged.started = false;
            env.storage().persistent().set(&Symbol::short("STAGED_RESUME"), &staged);
            Ok(staged.current_stage)
        }
    }

    // Complete recovery procedure
    pub fn complete_recovery(env: &Env, admin: &Address) -> Result<(), EmergencyPauseError> {
        admin.require_auth();
        let mut recovery: RecoveryProcedure = env.storage().persistent().get(&Symbol::short("RECOVERY")).unwrap();
        recovery.completed = true;
        env.storage().persistent().set(&Symbol::short("RECOVERY"), &recovery);
        Ok(())
    }

    pub fn is_emergency_paused(env: &Env) -> Result<bool, EmergencyPauseError> {
        if let Some(pause_state) = env.storage().persistent().get::<_, EmergencyPauseState>(&EMERGENCY_PAUSE) {
            let current_time = env.ledger().timestamp();
            
            // Auto-expire if duration exceeded
            if current_time > pause_state.pause_timestamp + pause_state.max_duration_seconds {
                env.storage().persistent().remove(&EMERGENCY_PAUSE);
                return Ok(false);
            }
            
            Ok(pause_state.is_paused)
        } else {
            Ok(false)
        }
    }

    pub fn get_emergency_pause_state(env: &Env) -> Result<EmergencyPauseState, EmergencyPauseError> {
        env.storage().persistent()
            .get(&EMERGENCY_PAUSE)
            .ok_or(EmergencyPauseError::NotPaused)
    }
}
