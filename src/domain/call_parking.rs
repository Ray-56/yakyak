//! Call Parking domain model
//!
//! Provides call parking functionality allowing users to place calls on hold
//! in a shared parking slot that can be retrieved from any extension.
//! Includes timeout and callback features if the call is not retrieved.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Call parking slot state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParkingSlotState {
    /// Slot is available for parking
    Available,
    /// Slot has a parked call
    Occupied,
    /// Slot is reserved but call not yet parked
    Reserved,
}

/// Call parking timeout action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Call back the parker
    CallbackParker,
    /// Call back the original caller
    CallbackCaller,
    /// Transfer to operator/attendant
    TransferOperator,
    /// Disconnect the call
    Disconnect,
    /// Send to voicemail
    Voicemail,
}

impl TimeoutAction {
    pub fn description(&self) -> &str {
        match self {
            TimeoutAction::CallbackParker => "Call back parker",
            TimeoutAction::CallbackCaller => "Call back caller",
            TimeoutAction::TransferOperator => "Transfer to operator",
            TimeoutAction::Disconnect => "Disconnect call",
            TimeoutAction::Voicemail => "Send to voicemail",
        }
    }
}

/// Parked call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkedCall {
    /// Unique identifier for this parked call
    pub id: Uuid,
    /// Call ID from the original call
    pub call_id: String,
    /// Parking slot number
    pub slot_number: u32,
    /// Extension that parked the call
    pub parker_extension: String,
    /// Original caller information
    pub caller_id: String,
    /// Original callee information
    pub callee_id: String,
    /// When the call was parked
    pub parked_at: DateTime<Utc>,
    /// When the parking will timeout
    pub timeout_at: DateTime<Utc>,
    /// Action to take on timeout
    pub timeout_action: TimeoutAction,
    /// Whether timeout callback has been attempted
    pub timeout_attempted: bool,
    /// Number of retrieval attempts
    pub retrieval_attempts: u32,
    /// Optional custom announcement for the slot
    pub announcement: Option<String>,
}

impl ParkedCall {
    pub fn new(
        call_id: String,
        slot_number: u32,
        parker_extension: String,
        caller_id: String,
        callee_id: String,
        timeout_seconds: i64,
        timeout_action: TimeoutAction,
    ) -> Self {
        let now = Utc::now();
        let timeout_at = now + Duration::seconds(timeout_seconds);

        Self {
            id: Uuid::new_v4(),
            call_id,
            slot_number,
            parker_extension,
            caller_id,
            callee_id,
            parked_at: now,
            timeout_at,
            timeout_action,
            timeout_attempted: false,
            retrieval_attempts: 0,
            announcement: None,
        }
    }

    /// Check if the parking has timed out
    pub fn is_timed_out(&self) -> bool {
        Utc::now() >= self.timeout_at
    }

    /// Get remaining time until timeout
    pub fn remaining_seconds(&self) -> i64 {
        let now = Utc::now();
        if now >= self.timeout_at {
            0
        } else {
            (self.timeout_at - now).num_seconds()
        }
    }

    /// Get duration parked in seconds
    pub fn parked_duration_seconds(&self) -> i64 {
        (Utc::now() - self.parked_at).num_seconds()
    }
}

/// Call parking slot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingSlot {
    /// Slot number (e.g., 700-799)
    pub number: u32,
    /// Slot state
    pub state: ParkingSlotState,
    /// Currently parked call (if occupied)
    pub parked_call: Option<ParkedCall>,
    /// Display name for the slot
    pub name: Option<String>,
    /// Whether the slot is enabled
    pub enabled: bool,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
}

impl ParkingSlot {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            state: ParkingSlotState::Available,
            parked_call: None,
            name: None,
            enabled: true,
            last_used: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Park a call in this slot
    pub fn park(&mut self, call: ParkedCall) -> Result<(), String> {
        if !self.enabled {
            return Err("Parking slot is disabled".to_string());
        }

        if self.state != ParkingSlotState::Available {
            return Err(format!("Parking slot {} is not available", self.number));
        }

        self.state = ParkingSlotState::Occupied;
        self.parked_call = Some(call);
        self.last_used = Some(Utc::now());

        Ok(())
    }

    /// Retrieve the parked call
    pub fn retrieve(&mut self) -> Result<ParkedCall, String> {
        if self.state != ParkingSlotState::Occupied {
            return Err(format!("Parking slot {} has no parked call", self.number));
        }

        let mut call = self
            .parked_call
            .take()
            .ok_or_else(|| "No call in slot".to_string())?;

        call.retrieval_attempts += 1;
        self.state = ParkingSlotState::Available;

        Ok(call)
    }

    /// Clear the slot (for timeout or error)
    pub fn clear(&mut self) {
        self.state = ParkingSlotState::Available;
        self.parked_call = None;
    }

    /// Check if the parked call has timed out
    pub fn check_timeout(&self) -> bool {
        if let Some(ref call) = self.parked_call {
            call.is_timed_out()
        } else {
            false
        }
    }
}

/// Parking lot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingLotConfig {
    /// Parking lot identifier
    pub id: Uuid,
    /// Parking lot name
    pub name: String,
    /// Start of parking slot range
    pub slot_start: u32,
    /// End of parking slot range
    pub slot_end: u32,
    /// Default timeout in seconds
    pub default_timeout_seconds: i64,
    /// Default timeout action
    pub default_timeout_action: TimeoutAction,
    /// Parking slot assignment strategy
    pub assignment_strategy: SlotAssignmentStrategy,
    /// Whether to play announcement with slot number
    pub play_announcement: bool,
    /// Custom announcement audio file
    pub announcement_file: Option<String>,
    /// Whether the lot is enabled
    pub enabled: bool,
}

impl ParkingLotConfig {
    pub fn new(name: String, slot_start: u32, slot_end: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            slot_start,
            slot_end,
            default_timeout_seconds: 300, // 5 minutes default
            default_timeout_action: TimeoutAction::CallbackParker,
            assignment_strategy: SlotAssignmentStrategy::Sequential,
            play_announcement: true,
            announcement_file: None,
            enabled: true,
        }
    }

    pub fn with_timeout(mut self, seconds: i64, action: TimeoutAction) -> Self {
        self.default_timeout_seconds = seconds;
        self.default_timeout_action = action;
        self
    }

    pub fn with_strategy(mut self, strategy: SlotAssignmentStrategy) -> Self {
        self.assignment_strategy = strategy;
        self
    }

    /// Get number of slots in this lot
    pub fn slot_count(&self) -> u32 {
        self.slot_end - self.slot_start + 1
    }

    /// Check if a slot number belongs to this lot
    pub fn contains_slot(&self, slot_number: u32) -> bool {
        slot_number >= self.slot_start && slot_number <= self.slot_end
    }
}

/// Parking slot assignment strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotAssignmentStrategy {
    /// Assign slots sequentially (701, 702, 703...)
    Sequential,
    /// Assign slots in random order
    Random,
    /// Assign least recently used slot
    LeastRecentlyUsed,
    /// Assign first available slot
    FirstAvailable,
}

/// Call parking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingStatistics {
    /// Total slots in system
    pub total_slots: usize,
    /// Currently occupied slots
    pub occupied_slots: usize,
    /// Available slots
    pub available_slots: usize,
    /// Total calls parked (lifetime)
    pub total_parked_calls: u64,
    /// Total calls retrieved successfully
    pub total_retrieved_calls: u64,
    /// Total timeout callbacks
    pub total_timeouts: u64,
    /// Average parking duration in seconds
    pub average_parking_duration: i64,
    /// Calls by timeout action
    pub timeouts_by_action: HashMap<String, u64>,
}

impl ParkingStatistics {
    pub fn new() -> Self {
        Self {
            total_slots: 0,
            occupied_slots: 0,
            available_slots: 0,
            total_parked_calls: 0,
            total_retrieved_calls: 0,
            total_timeouts: 0,
            average_parking_duration: 0,
            timeouts_by_action: HashMap::new(),
        }
    }
}

impl Default for ParkingStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Call parking manager
pub struct CallParkingManager {
    /// Parking lots configuration
    lots: Arc<Mutex<HashMap<Uuid, ParkingLotConfig>>>,
    /// Parking slots
    slots: Arc<Mutex<HashMap<u32, ParkingSlot>>>,
    /// Call ID to slot number mapping
    call_to_slot: Arc<Mutex<HashMap<String, u32>>>,
    /// Statistics counters
    total_parked: Arc<Mutex<u64>>,
    total_retrieved: Arc<Mutex<u64>>,
    total_timeouts: Arc<Mutex<u64>>,
    timeouts_by_action: Arc<Mutex<HashMap<String, u64>>>,
    /// Parking duration history for averaging
    parking_durations: Arc<Mutex<VecDeque<i64>>>,
}

impl CallParkingManager {
    pub fn new() -> Self {
        Self {
            lots: Arc::new(Mutex::new(HashMap::new())),
            slots: Arc::new(Mutex::new(HashMap::new())),
            call_to_slot: Arc::new(Mutex::new(HashMap::new())),
            total_parked: Arc::new(Mutex::new(0)),
            total_retrieved: Arc::new(Mutex::new(0)),
            total_timeouts: Arc::new(Mutex::new(0)),
            timeouts_by_action: Arc::new(Mutex::new(HashMap::new())),
            parking_durations: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
        }
    }

    /// Create a new parking lot
    pub fn create_lot(&self, config: ParkingLotConfig) -> Result<Uuid, String> {
        let lot_id = config.id;

        // Validate slot range
        if config.slot_start >= config.slot_end {
            return Err("Invalid slot range: start must be less than end".to_string());
        }

        if config.slot_count() > 1000 {
            return Err("Slot range too large (max 1000 slots per lot)".to_string());
        }

        // Check for overlapping slots
        let slots = self.slots.lock().unwrap();
        for slot_num in config.slot_start..=config.slot_end {
            if slots.contains_key(&slot_num) {
                return Err(format!("Slot {} already exists in another lot", slot_num));
            }
        }
        drop(slots);

        // Create slots for the lot
        let mut slots = self.slots.lock().unwrap();
        for slot_num in config.slot_start..=config.slot_end {
            slots.insert(slot_num, ParkingSlot::new(slot_num));
        }

        // Store lot configuration
        let mut lots = self.lots.lock().unwrap();
        lots.insert(lot_id, config);

        Ok(lot_id)
    }

    /// Get parking lot configuration
    pub fn get_lot(&self, lot_id: Uuid) -> Option<ParkingLotConfig> {
        let lots = self.lots.lock().unwrap();
        lots.get(&lot_id).cloned()
    }

    /// Delete a parking lot
    pub fn delete_lot(&self, lot_id: Uuid) -> Result<(), String> {
        let mut lots = self.lots.lock().unwrap();
        let config = lots
            .remove(&lot_id)
            .ok_or_else(|| "Parking lot not found".to_string())?;

        // Remove all slots in the lot
        let mut slots = self.slots.lock().unwrap();
        for slot_num in config.slot_start..=config.slot_end {
            slots.remove(&slot_num);
        }

        Ok(())
    }

    /// Park a call in an available slot
    pub fn park_call(
        &self,
        call_id: String,
        parker_extension: String,
        caller_id: String,
        callee_id: String,
        preferred_slot: Option<u32>,
    ) -> Result<u32, String> {
        // Find an available slot
        let slot_number = if let Some(slot_num) = preferred_slot {
            // Use specific slot if requested
            let slots = self.slots.lock().unwrap();
            if slots.get(&slot_num).map(|s| s.enabled && s.state == ParkingSlotState::Available).unwrap_or(false) {
                slot_num
            } else {
                return Err(format!("Preferred slot {} is not available", slot_num));
            }
        } else {
            // Find next available slot
            self.find_available_slot()?
        };

        // Find the lot configuration for timeout settings
        let lots = self.lots.lock().unwrap();
        let lot_config = lots
            .values()
            .find(|lot| lot.contains_slot(slot_number))
            .ok_or_else(|| "Slot does not belong to any parking lot".to_string())?;

        let timeout_seconds = lot_config.default_timeout_seconds;
        let timeout_action = lot_config.default_timeout_action;
        drop(lots);

        // Create parked call
        let parked_call = ParkedCall::new(
            call_id.clone(),
            slot_number,
            parker_extension,
            caller_id,
            callee_id,
            timeout_seconds,
            timeout_action,
        );

        // Park the call in the slot
        let mut slots = self.slots.lock().unwrap();
        let slot = slots
            .get_mut(&slot_number)
            .ok_or_else(|| "Parking slot not found".to_string())?;

        slot.park(parked_call)?;

        // Map call ID to slot
        let mut call_to_slot = self.call_to_slot.lock().unwrap();
        call_to_slot.insert(call_id, slot_number);

        // Update statistics
        let mut total_parked = self.total_parked.lock().unwrap();
        *total_parked += 1;

        Ok(slot_number)
    }

    /// Retrieve a parked call from a slot
    pub fn retrieve_call(&self, slot_number: u32) -> Result<ParkedCall, String> {
        let mut slots = self.slots.lock().unwrap();
        let slot = slots
            .get_mut(&slot_number)
            .ok_or_else(|| format!("Parking slot {} not found", slot_number))?;

        let parked_call = slot.retrieve()?;

        // Update call to slot mapping
        let mut call_to_slot = self.call_to_slot.lock().unwrap();
        call_to_slot.remove(&parked_call.call_id);

        // Update statistics
        let mut total_retrieved = self.total_retrieved.lock().unwrap();
        *total_retrieved += 1;

        // Record parking duration
        let mut durations = self.parking_durations.lock().unwrap();
        durations.push_back(parked_call.parked_duration_seconds());
        if durations.len() > 1000 {
            durations.pop_front();
        }

        Ok(parked_call)
    }

    /// Find slot number for a parked call ID
    pub fn find_slot_by_call_id(&self, call_id: &str) -> Option<u32> {
        let call_to_slot = self.call_to_slot.lock().unwrap();
        call_to_slot.get(call_id).copied()
    }

    /// Get information about a parking slot
    pub fn get_slot(&self, slot_number: u32) -> Option<ParkingSlot> {
        let slots = self.slots.lock().unwrap();
        slots.get(&slot_number).cloned()
    }

    /// List all parking slots
    pub fn list_slots(&self) -> Vec<ParkingSlot> {
        let slots = self.slots.lock().unwrap();
        let mut slot_list: Vec<_> = slots.values().cloned().collect();
        slot_list.sort_by_key(|s| s.number);
        slot_list
    }

    /// List occupied slots
    pub fn list_occupied_slots(&self) -> Vec<ParkingSlot> {
        let slots = self.slots.lock().unwrap();
        let mut occupied: Vec<_> = slots
            .values()
            .filter(|s| s.state == ParkingSlotState::Occupied)
            .cloned()
            .collect();
        occupied.sort_by_key(|s| s.number);
        occupied
    }

    /// Check for timed out calls and process them
    pub fn process_timeouts(&self) -> Vec<(u32, ParkedCall)> {
        let mut slots = self.slots.lock().unwrap();
        let mut timed_out = Vec::new();

        for (slot_num, slot) in slots.iter_mut() {
            if slot.check_timeout() {
                if let Some(mut parked_call) = slot.parked_call.take() {
                    if !parked_call.timeout_attempted {
                        parked_call.timeout_attempted = true;
                        timed_out.push((*slot_num, parked_call));
                        slot.clear();

                        // Update statistics
                        let mut total_timeouts = self.total_timeouts.lock().unwrap();
                        *total_timeouts += 1;
                    }
                }
            }
        }

        // Update call to slot mapping
        if !timed_out.is_empty() {
            let mut call_to_slot = self.call_to_slot.lock().unwrap();
            for (_, call) in &timed_out {
                call_to_slot.remove(&call.call_id);

                // Record timeout by action
                let mut timeouts_by_action = self.timeouts_by_action.lock().unwrap();
                let action_name = format!("{:?}", call.timeout_action);
                *timeouts_by_action.entry(action_name).or_insert(0) += 1;
            }
        }

        timed_out
    }

    /// Find an available parking slot using the configured strategy
    fn find_available_slot(&self) -> Result<u32, String> {
        let slots = self.slots.lock().unwrap();
        let lots = self.lots.lock().unwrap();

        // Find first enabled lot
        let lot = lots
            .values()
            .find(|lot| lot.enabled)
            .ok_or_else(|| "No enabled parking lots available".to_string())?;

        // Find available slot based on strategy
        let available_slots: Vec<_> = slots
            .values()
            .filter(|s| {
                s.enabled
                    && s.state == ParkingSlotState::Available
                    && lot.contains_slot(s.number)
            })
            .collect();

        if available_slots.is_empty() {
            return Err("No available parking slots".to_string());
        }

        let slot = match lot.assignment_strategy {
            SlotAssignmentStrategy::Sequential | SlotAssignmentStrategy::FirstAvailable => {
                available_slots.iter().min_by_key(|s| s.number)
            }
            SlotAssignmentStrategy::LeastRecentlyUsed => {
                available_slots.iter().min_by_key(|s| {
                    s.last_used.unwrap_or_else(|| DateTime::<Utc>::MIN_UTC)
                })
            }
            SlotAssignmentStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..available_slots.len());
                available_slots.get(idx)
            }
        };

        slot.map(|s| s.number)
            .ok_or_else(|| "Failed to find available slot".to_string())
    }

    /// Get parking statistics
    pub fn get_statistics(&self) -> ParkingStatistics {
        let slots = self.slots.lock().unwrap();
        let total_parked = self.total_parked.lock().unwrap();
        let total_retrieved = self.total_retrieved.lock().unwrap();
        let total_timeouts = self.total_timeouts.lock().unwrap();
        let timeouts_by_action = self.timeouts_by_action.lock().unwrap();
        let durations = self.parking_durations.lock().unwrap();

        let mut stats = ParkingStatistics::new();
        stats.total_slots = slots.len();
        stats.occupied_slots = slots
            .values()
            .filter(|s| s.state == ParkingSlotState::Occupied)
            .count();
        stats.available_slots = stats.total_slots - stats.occupied_slots;
        stats.total_parked_calls = *total_parked;
        stats.total_retrieved_calls = *total_retrieved;
        stats.total_timeouts = *total_timeouts;
        stats.timeouts_by_action = timeouts_by_action.clone();

        if !durations.is_empty() {
            let sum: i64 = durations.iter().sum();
            stats.average_parking_duration = sum / durations.len() as i64;
        }

        stats
    }

    /// Clear all parking slots (for testing or emergency)
    pub fn clear_all_slots(&self) {
        let mut slots = self.slots.lock().unwrap();
        for slot in slots.values_mut() {
            slot.clear();
        }

        let mut call_to_slot = self.call_to_slot.lock().unwrap();
        call_to_slot.clear();
    }

    /// List all parking lots
    pub fn list_lots(&self) -> Vec<ParkingLotConfig> {
        let lots = self.lots.lock().unwrap();
        lots.values().cloned().collect()
    }
}

impl Default for CallParkingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_action_description() {
        assert_eq!(TimeoutAction::CallbackParker.description(), "Call back parker");
        assert_eq!(TimeoutAction::Disconnect.description(), "Disconnect call");
    }

    #[test]
    fn test_parked_call_creation() {
        let call = ParkedCall::new(
            "call-123".to_string(),
            701,
            "alice".to_string(),
            "1001".to_string(),
            "1002".to_string(),
            300,
            TimeoutAction::CallbackParker,
        );

        assert_eq!(call.call_id, "call-123");
        assert_eq!(call.slot_number, 701);
        assert_eq!(call.parker_extension, "alice");
        assert_eq!(call.timeout_action, TimeoutAction::CallbackParker);
        assert!(!call.is_timed_out());
        assert!(call.remaining_seconds() > 290);
    }

    #[test]
    fn test_parking_slot_park_and_retrieve() {
        let mut slot = ParkingSlot::new(700);

        let call = ParkedCall::new(
            "call-123".to_string(),
            700,
            "alice".to_string(),
            "1001".to_string(),
            "1002".to_string(),
            300,
            TimeoutAction::CallbackParker,
        );

        // Park call
        slot.park(call.clone()).unwrap();
        assert_eq!(slot.state, ParkingSlotState::Occupied);
        assert!(slot.parked_call.is_some());

        // Retrieve call
        let retrieved = slot.retrieve().unwrap();
        assert_eq!(retrieved.call_id, "call-123");
        assert_eq!(slot.state, ParkingSlotState::Available);
        assert!(slot.parked_call.is_none());
    }

    #[test]
    fn test_parking_slot_double_park_fails() {
        let mut slot = ParkingSlot::new(700);

        let call1 = ParkedCall::new(
            "call-1".to_string(),
            700,
            "alice".to_string(),
            "1001".to_string(),
            "1002".to_string(),
            300,
            TimeoutAction::CallbackParker,
        );

        slot.park(call1).unwrap();

        let call2 = ParkedCall::new(
            "call-2".to_string(),
            700,
            "bob".to_string(),
            "1003".to_string(),
            "1004".to_string(),
            300,
            TimeoutAction::CallbackParker,
        );

        let result = slot.park(call2);
        assert!(result.is_err());
    }

    #[test]
    fn test_parking_lot_config() {
        let lot = ParkingLotConfig::new("Main Lot".to_string(), 700, 799);

        assert_eq!(lot.name, "Main Lot");
        assert_eq!(lot.slot_count(), 100);
        assert!(lot.contains_slot(750));
        assert!(!lot.contains_slot(800));
        assert!(!lot.contains_slot(699));
    }

    #[test]
    fn test_create_parking_lot() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        let lot_id = manager.create_lot(config).unwrap();

        assert!(lot_id != Uuid::nil());

        let slots = manager.list_slots();
        assert_eq!(slots.len(), 11); // 700-710 inclusive
    }

    #[test]
    fn test_park_and_retrieve_call() {
        let manager = CallParkingManager::new();

        // Create parking lot
        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        manager.create_lot(config).unwrap();

        // Park a call
        let slot_num = manager
            .park_call(
                "call-123".to_string(),
                "alice".to_string(),
                "1001".to_string(),
                "1002".to_string(),
                None,
            )
            .unwrap();

        assert_eq!(slot_num, 700); // First available slot

        // Verify slot is occupied
        let slot = manager.get_slot(slot_num).unwrap();
        assert_eq!(slot.state, ParkingSlotState::Occupied);

        // Retrieve the call
        let retrieved = manager.retrieve_call(slot_num).unwrap();
        assert_eq!(retrieved.call_id, "call-123");
        assert_eq!(retrieved.parker_extension, "alice");

        // Verify slot is now available
        let slot_after = manager.get_slot(slot_num).unwrap();
        assert_eq!(slot_after.state, ParkingSlotState::Available);
    }

    #[test]
    fn test_park_preferred_slot() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        manager.create_lot(config).unwrap();

        // Park with preferred slot
        let slot_num = manager
            .park_call(
                "call-123".to_string(),
                "alice".to_string(),
                "1001".to_string(),
                "1002".to_string(),
                Some(705),
            )
            .unwrap();

        assert_eq!(slot_num, 705);
    }

    #[test]
    fn test_multiple_parked_calls() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 705);
        manager.create_lot(config).unwrap();

        // Park multiple calls
        let slot1 = manager
            .park_call(
                "call-1".to_string(),
                "alice".to_string(),
                "1001".to_string(),
                "1002".to_string(),
                None,
            )
            .unwrap();

        let slot2 = manager
            .park_call(
                "call-2".to_string(),
                "bob".to_string(),
                "1003".to_string(),
                "1004".to_string(),
                None,
            )
            .unwrap();

        assert_eq!(slot1, 700);
        assert_eq!(slot2, 701);

        let occupied = manager.list_occupied_slots();
        assert_eq!(occupied.len(), 2);
    }

    #[test]
    fn test_find_slot_by_call_id() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        manager.create_lot(config).unwrap();

        let slot_num = manager
            .park_call(
                "call-123".to_string(),
                "alice".to_string(),
                "1001".to_string(),
                "1002".to_string(),
                None,
            )
            .unwrap();

        let found_slot = manager.find_slot_by_call_id("call-123");
        assert_eq!(found_slot, Some(slot_num));

        let not_found = manager.find_slot_by_call_id("call-999");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_parking_statistics() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        manager.create_lot(config).unwrap();

        // Park and retrieve some calls
        let slot1 = manager
            .park_call(
                "call-1".to_string(),
                "alice".to_string(),
                "1001".to_string(),
                "1002".to_string(),
                None,
            )
            .unwrap();

        manager
            .park_call(
                "call-2".to_string(),
                "bob".to_string(),
                "1003".to_string(),
                "1004".to_string(),
                None,
            )
            .unwrap();

        manager.retrieve_call(slot1).unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_slots, 11);
        assert_eq!(stats.occupied_slots, 1);
        assert_eq!(stats.available_slots, 10);
        assert_eq!(stats.total_parked_calls, 2);
        assert_eq!(stats.total_retrieved_calls, 1);
    }

    #[test]
    fn test_timeout_detection() {
        let call = ParkedCall::new(
            "call-123".to_string(),
            700,
            "alice".to_string(),
            "1001".to_string(),
            "1002".to_string(),
            -10, // Negative timeout = already expired
            TimeoutAction::CallbackParker,
        );

        assert!(call.is_timed_out());
        assert_eq!(call.remaining_seconds(), 0);
    }

    #[test]
    fn test_clear_all_slots() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        manager.create_lot(config).unwrap();

        // Park multiple calls
        manager
            .park_call(
                "call-1".to_string(),
                "alice".to_string(),
                "1001".to_string(),
                "1002".to_string(),
                None,
            )
            .unwrap();

        manager
            .park_call(
                "call-2".to_string(),
                "bob".to_string(),
                "1003".to_string(),
                "1004".to_string(),
                None,
            )
            .unwrap();

        let occupied_before = manager.list_occupied_slots();
        assert_eq!(occupied_before.len(), 2);

        manager.clear_all_slots();

        let occupied_after = manager.list_occupied_slots();
        assert_eq!(occupied_after.len(), 0);
    }

    #[test]
    fn test_delete_parking_lot() {
        let manager = CallParkingManager::new();

        let config = ParkingLotConfig::new("Test Lot".to_string(), 700, 710);
        let lot_id = manager.create_lot(config).unwrap();

        assert_eq!(manager.list_slots().len(), 11);

        manager.delete_lot(lot_id).unwrap();

        assert_eq!(manager.list_slots().len(), 0);
    }

    #[test]
    fn test_overlapping_lots_rejected() {
        let manager = CallParkingManager::new();

        let config1 = ParkingLotConfig::new("Lot 1".to_string(), 700, 750);
        manager.create_lot(config1).unwrap();

        let config2 = ParkingLotConfig::new("Lot 2".to_string(), 740, 790);
        let result = manager.create_lot(config2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_disabled_slot_cannot_park() {
        let mut slot = ParkingSlot::new(700).disabled();

        let call = ParkedCall::new(
            "call-123".to_string(),
            700,
            "alice".to_string(),
            "1001".to_string(),
            "1002".to_string(),
            300,
            TimeoutAction::CallbackParker,
        );

        let result = slot.park(call);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("disabled"));
    }
}
