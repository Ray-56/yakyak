/// IVR flow engine for executing IVR logic
use super::dtmf::{DtmfDetector, DtmfEvent};
use super::menu::{IvrMenu, IvrMenuSystem, MenuAction};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// IVR session state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IvrState {
    /// Initial state
    Start,
    /// Playing greeting
    PlayingGreeting,
    /// Waiting for digit input
    WaitingForInput,
    /// Processing input
    ProcessingInput,
    /// Playing audio
    PlayingAudio(String),
    /// Transferring call
    Transferring(String),
    /// Invalid input
    InvalidInput,
    /// Timeout
    Timeout,
    /// Completed
    Completed,
}

/// IVR flow session
pub struct IvrSession {
    pub session_id: String,
    pub current_menu_id: Option<String>,
    pub state: IvrState,
    pub dtmf_detector: DtmfDetector,
    pub retry_count: u32,
    pub menu_stack: Vec<String>, // For GoBack action
    pub variables: HashMap<String, String>, // Session variables
}

impl IvrSession {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_menu_id: None,
            state: IvrState::Start,
            dtmf_detector: DtmfDetector::default_settings(),
            retry_count: 0,
            menu_stack: Vec::new(),
            variables: HashMap::new(),
        }
    }

    /// Set a session variable
    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// Get a session variable
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Push menu to stack
    pub fn push_menu(&mut self, menu_id: String) {
        if let Some(current) = &self.current_menu_id {
            self.menu_stack.push(current.clone());
        }
        self.current_menu_id = Some(menu_id);
    }

    /// Pop menu from stack
    pub fn pop_menu(&mut self) -> Option<String> {
        self.menu_stack.pop()
    }
}

/// IVR flow definition
#[derive(Debug, Clone)]
pub struct IvrFlow {
    pub id: String,
    pub name: String,
    pub start_menu_id: String,
    pub menu_system: Arc<RwLock<IvrMenuSystem>>,
}

impl IvrFlow {
    pub fn new(id: String, name: String, start_menu_id: String, menu_system: IvrMenuSystem) -> Self {
        Self {
            id,
            name,
            start_menu_id,
            menu_system: Arc::new(RwLock::new(menu_system)),
        }
    }

    /// Get menu system
    pub async fn get_menu_system(&self) -> tokio::sync::RwLockReadGuard<'_, IvrMenuSystem> {
        self.menu_system.read().await
    }
}

/// IVR flow engine
pub struct IvrFlowEngine {
    sessions: Arc<RwLock<HashMap<String, IvrSession>>>,
}

impl IvrFlowEngine {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new IVR session
    pub async fn start_session(&self, session_id: String, flow: &IvrFlow) -> Result<IvrSession, String> {
        let mut session = IvrSession::new(session_id.clone());
        session.current_menu_id = Some(flow.start_menu_id.clone());
        session.state = IvrState::PlayingGreeting;

        info!("Started IVR session {} with menu {}", session_id, flow.start_menu_id);

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        Ok(session)
    }

    /// Process DTMF event for a session
    pub async fn process_dtmf(
        &self,
        session_id: &str,
        event: DtmfEvent,
        flow: &IvrFlow,
    ) -> Result<MenuAction, String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Add digit to detector
        session.dtmf_detector.process_event(event.clone());
        let digit = event.digit.to_char();

        debug!("Processing DTMF digit '{}' for session {}", digit, session_id);

        // Get current menu
        let current_menu_id = session.current_menu_id.clone()
            .ok_or_else(|| "No current menu".to_string())?;

        let menu_system = flow.menu_system.read().await;
        let menu = menu_system.get_menu(&current_menu_id)
            .ok_or_else(|| format!("Menu {} not found", current_menu_id))?;

        // Check if digit is valid
        if !menu.is_valid_digit(digit) {
            session.retry_count += 1;
            session.state = IvrState::InvalidInput;

            if session.retry_count >= menu.max_retries {
                warn!("Max retries exceeded for session {}", session_id);
                session.state = IvrState::Completed;
                return Ok(MenuAction::Hangup);
            }

            return Err("Invalid digit".to_string());
        }

        // Get menu item and action
        let item = menu.get_item(digit).unwrap();
        let action = item.action.clone();

        session.retry_count = 0; // Reset retry count on valid input
        session.dtmf_detector.clear_buffer();

        // Process action
        match &action {
            MenuAction::GotoMenu(menu_id) => {
                info!("Going to menu {} from session {}", menu_id, session_id);
                session.push_menu(menu_id.clone());
                session.state = IvrState::PlayingGreeting;
            }
            MenuAction::GoBack => {
                if let Some(prev_menu) = session.pop_menu() {
                    info!("Going back to menu {} from session {}", prev_menu, session_id);
                    session.current_menu_id = Some(prev_menu);
                    session.state = IvrState::PlayingGreeting;
                } else {
                    warn!("No previous menu to go back to");
                    session.state = IvrState::Completed;
                    return Ok(MenuAction::Hangup);
                }
            }
            MenuAction::Repeat => {
                info!("Repeating menu for session {}", session_id);
                session.state = IvrState::PlayingGreeting;
            }
            MenuAction::Transfer(destination) => {
                info!("Transferring session {} to {}", session_id, destination);
                session.state = IvrState::Transferring(destination.clone());
            }
            MenuAction::Hangup => {
                info!("Hanging up session {}", session_id);
                session.state = IvrState::Completed;
            }
            MenuAction::PlayAudio(file) => {
                info!("Playing audio {} for session {}", file, session_id);
                session.state = IvrState::PlayingAudio(file.clone());
            }
            _ => {
                session.state = IvrState::ProcessingInput;
            }
        }

        Ok(action)
    }

    /// Handle timeout for a session
    pub async fn handle_timeout(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.retry_count += 1;
        session.state = IvrState::Timeout;

        warn!("Timeout for session {} (retry {})", session_id, session.retry_count);

        Ok(())
    }

    /// End a session
    pub async fn end_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        info!("Ended IVR session {}", session_id);
    }

    /// Get session
    pub async fn get_session(&self, session_id: &str) -> Option<IvrSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Count active sessions
    pub async fn count_sessions(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for IvrFlowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::ivr::dtmf::DtmfDigit;
    use crate::infrastructure::ivr::menu::IvrMenuBuilder;
    use std::time::Duration;

    fn create_test_menu_system() -> IvrMenuSystem {
        let mut system = IvrMenuSystem::new();

        let main_menu = IvrMenuBuilder::new(
            "main".to_string(),
            "Main Menu".to_string(),
            "main.wav".to_string(),
        )
        .add_item('1', "Sales".to_string(), MenuAction::Transfer("sip:sales@example.com".to_string()))
        .add_item('2', "Support".to_string(), MenuAction::GotoMenu("support".to_string()))
        .add_item('0', "Repeat".to_string(), MenuAction::Repeat)
        .add_item('*', "Go Back".to_string(), MenuAction::GoBack)
        .build();

        let support_menu = IvrMenuBuilder::new(
            "support".to_string(),
            "Support Menu".to_string(),
            "support.wav".to_string(),
        )
        .add_item('1', "Technical".to_string(), MenuAction::Transfer("sip:tech@example.com".to_string()))
        .add_item('*', "Go Back".to_string(), MenuAction::GoBack)
        .build();

        system.add_menu(main_menu);
        system.add_menu(support_menu);

        system
    }

    #[tokio::test]
    async fn test_start_session() {
        let menu_system = create_test_menu_system();
        let flow = IvrFlow::new(
            "test_flow".to_string(),
            "Test Flow".to_string(),
            "main".to_string(),
            menu_system,
        );

        let engine = IvrFlowEngine::new();
        let session = engine.start_session("session1".to_string(), &flow).await.unwrap();

        assert_eq!(session.session_id, "session1");
        assert_eq!(session.current_menu_id, Some("main".to_string()));
        assert_eq!(session.state, IvrState::PlayingGreeting);
        assert_eq!(engine.count_sessions().await, 1);
    }

    #[tokio::test]
    async fn test_process_dtmf() {
        let menu_system = create_test_menu_system();
        let flow = IvrFlow::new(
            "test_flow".to_string(),
            "Test Flow".to_string(),
            "main".to_string(),
            menu_system,
        );

        let engine = IvrFlowEngine::new();
        engine.start_session("session1".to_string(), &flow).await.unwrap();

        // Press '1' (Sales)
        let event = DtmfEvent::new(DtmfDigit::One, Duration::from_millis(100));
        let action = engine.process_dtmf("session1", event, &flow).await.unwrap();

        match action {
            MenuAction::Transfer(dest) => {
                assert_eq!(dest, "sip:sales@example.com");
            }
            _ => panic!("Expected Transfer action"),
        }
    }

    #[tokio::test]
    async fn test_menu_navigation() {
        let menu_system = create_test_menu_system();
        let flow = IvrFlow::new(
            "test_flow".to_string(),
            "Test Flow".to_string(),
            "main".to_string(),
            menu_system,
        );

        let engine = IvrFlowEngine::new();
        engine.start_session("session1".to_string(), &flow).await.unwrap();

        // Press '2' (Go to Support menu)
        let event = DtmfEvent::new(DtmfDigit::Two, Duration::from_millis(100));
        engine.process_dtmf("session1", event, &flow).await.unwrap();

        let session = engine.get_session("session1").await.unwrap();
        assert_eq!(session.current_menu_id, Some("support".to_string()));
        assert_eq!(session.menu_stack.len(), 1);
        assert_eq!(session.menu_stack[0], "main");

        // Press '*' (Go back)
        let event = DtmfEvent::new(DtmfDigit::Star, Duration::from_millis(100));
        engine.process_dtmf("session1", event, &flow).await.unwrap();

        let session = engine.get_session("session1").await.unwrap();
        assert_eq!(session.current_menu_id, Some("main".to_string()));
        assert_eq!(session.menu_stack.len(), 0);
    }

    #[tokio::test]
    async fn test_invalid_digit() {
        let menu_system = create_test_menu_system();
        let flow = IvrFlow::new(
            "test_flow".to_string(),
            "Test Flow".to_string(),
            "main".to_string(),
            menu_system,
        );

        let engine = IvrFlowEngine::new();
        engine.start_session("session1".to_string(), &flow).await.unwrap();

        // Press '9' (invalid digit)
        let event = DtmfEvent::new(DtmfDigit::Nine, Duration::from_millis(100));
        let result = engine.process_dtmf("session1", event, &flow).await;

        assert!(result.is_err());

        let session = engine.get_session("session1").await.unwrap();
        assert_eq!(session.retry_count, 1);
        assert_eq!(session.state, IvrState::InvalidInput);
    }

    #[tokio::test]
    async fn test_session_variables() {
        let mut session = IvrSession::new("test".to_string());

        session.set_variable("caller".to_string(), "alice".to_string());
        session.set_variable("language".to_string(), "en".to_string());

        assert_eq!(session.get_variable("caller"), Some(&"alice".to_string()));
        assert_eq!(session.get_variable("language"), Some(&"en".to_string()));
        assert_eq!(session.get_variable("nonexistent"), None);
    }
}
