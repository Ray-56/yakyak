/// IVR menu system
use super::dtmf::DtmfDigit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Action to take when menu item is selected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MenuAction {
    /// Play audio file
    PlayAudio(String),
    /// Transfer to extension
    Transfer(String),
    /// Go to another menu
    GotoMenu(String),
    /// Go to voicemail
    Voicemail(String),
    /// Dial by extension
    DialExtension,
    /// Repeat current menu
    Repeat,
    /// Return to previous menu
    GoBack,
    /// Hang up
    Hangup,
    /// Custom action
    Custom(String),
}

/// IVR menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrMenuItem {
    pub digit: char,
    pub label: String,
    pub action: MenuAction,
}

impl IvrMenuItem {
    pub fn new(digit: char, label: String, action: MenuAction) -> Self {
        Self {
            digit,
            label,
            action,
        }
    }
}

/// IVR menu configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrMenu {
    pub id: String,
    pub name: String,
    pub greeting_file: String,
    pub items: Vec<IvrMenuItem>,
    pub timeout_seconds: u32,
    pub max_retries: u32,
    pub invalid_sound: Option<String>,
    pub timeout_sound: Option<String>,
}

impl IvrMenu {
    pub fn new(id: String, name: String, greeting_file: String) -> Self {
        Self {
            id,
            name,
            greeting_file,
            items: Vec::new(),
            timeout_seconds: 5,
            max_retries: 3,
            invalid_sound: Some("ivr/invalid.wav".to_string()),
            timeout_sound: Some("ivr/timeout.wav".to_string()),
        }
    }

    /// Add menu item
    pub fn add_item(&mut self, item: IvrMenuItem) {
        self.items.push(item);
    }

    /// Get menu item by digit
    pub fn get_item(&self, digit: char) -> Option<&IvrMenuItem> {
        self.items.iter().find(|item| item.digit == digit)
    }

    /// Get all available digits
    pub fn get_available_digits(&self) -> Vec<char> {
        self.items.iter().map(|item| item.digit).collect()
    }

    /// Check if digit is valid
    pub fn is_valid_digit(&self, digit: char) -> bool {
        self.items.iter().any(|item| item.digit == digit)
    }
}

/// IVR menu builder for easy construction
pub struct IvrMenuBuilder {
    menu: IvrMenu,
}

impl IvrMenuBuilder {
    pub fn new(id: String, name: String, greeting_file: String) -> Self {
        Self {
            menu: IvrMenu::new(id, name, greeting_file),
        }
    }

    pub fn timeout(mut self, seconds: u32) -> Self {
        self.menu.timeout_seconds = seconds;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.menu.max_retries = retries;
        self
    }

    pub fn add_item(mut self, digit: char, label: String, action: MenuAction) -> Self {
        self.menu.add_item(IvrMenuItem::new(digit, label, action));
        self
    }

    pub fn build(self) -> IvrMenu {
        self.menu
    }
}

/// IVR menu system manager
pub struct IvrMenuSystem {
    menus: HashMap<String, IvrMenu>,
}

impl IvrMenuSystem {
    pub fn new() -> Self {
        Self {
            menus: HashMap::new(),
        }
    }

    /// Add menu to the system
    pub fn add_menu(&mut self, menu: IvrMenu) {
        self.menus.insert(menu.id.clone(), menu);
    }

    /// Get menu by ID
    pub fn get_menu(&self, id: &str) -> Option<&IvrMenu> {
        self.menus.get(id)
    }

    /// Remove menu
    pub fn remove_menu(&mut self, id: &str) -> Option<IvrMenu> {
        self.menus.remove(id)
    }

    /// List all menu IDs
    pub fn list_menu_ids(&self) -> Vec<String> {
        self.menus.keys().cloned().collect()
    }

    /// Load menus from JSON configuration
    pub fn load_from_json(&mut self, json: &str) -> Result<(), String> {
        let menus: Vec<IvrMenu> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        for menu in menus {
            self.add_menu(menu);
        }

        Ok(())
    }

    /// Create a default main menu
    pub fn create_default_main_menu() -> IvrMenu {
        IvrMenuBuilder::new(
            "main".to_string(),
            "Main Menu".to_string(),
            "ivr/main_menu.wav".to_string(),
        )
        .timeout(10)
        .max_retries(3)
        .add_item(
            '1',
            "Sales".to_string(),
            MenuAction::Transfer("sip:sales@example.com".to_string()),
        )
        .add_item(
            '2',
            "Support".to_string(),
            MenuAction::Transfer("sip:support@example.com".to_string()),
        )
        .add_item(
            '3',
            "Directory".to_string(),
            MenuAction::GotoMenu("directory".to_string()),
        )
        .add_item(
            '9',
            "Operator".to_string(),
            MenuAction::Transfer("sip:operator@example.com".to_string()),
        )
        .add_item(
            '0',
            "Repeat".to_string(),
            MenuAction::Repeat,
        )
        .add_item(
            '*',
            "Go Back".to_string(),
            MenuAction::GoBack,
        )
        .build()
    }
}

impl Default for IvrMenuSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_menu() {
        let menu = IvrMenu::new(
            "test".to_string(),
            "Test Menu".to_string(),
            "test.wav".to_string(),
        );

        assert_eq!(menu.id, "test");
        assert_eq!(menu.name, "Test Menu");
        assert_eq!(menu.greeting_file, "test.wav");
        assert_eq!(menu.timeout_seconds, 5);
        assert_eq!(menu.max_retries, 3);
    }

    #[test]
    fn test_menu_items() {
        let mut menu = IvrMenu::new(
            "test".to_string(),
            "Test Menu".to_string(),
            "test.wav".to_string(),
        );

        menu.add_item(IvrMenuItem::new(
            '1',
            "Option 1".to_string(),
            MenuAction::PlayAudio("option1.wav".to_string()),
        ));

        menu.add_item(IvrMenuItem::new(
            '2',
            "Option 2".to_string(),
            MenuAction::Transfer("sip:extension@example.com".to_string()),
        ));

        assert_eq!(menu.items.len(), 2);
        assert!(menu.is_valid_digit('1'));
        assert!(menu.is_valid_digit('2'));
        assert!(!menu.is_valid_digit('3'));

        let item = menu.get_item('1').unwrap();
        assert_eq!(item.label, "Option 1");
    }

    #[test]
    fn test_menu_builder() {
        let menu = IvrMenuBuilder::new(
            "main".to_string(),
            "Main Menu".to_string(),
            "main.wav".to_string(),
        )
        .timeout(10)
        .max_retries(5)
        .add_item('1', "Sales".to_string(), MenuAction::Transfer("100".to_string()))
        .add_item('2', "Support".to_string(), MenuAction::Transfer("200".to_string()))
        .build();

        assert_eq!(menu.timeout_seconds, 10);
        assert_eq!(menu.max_retries, 5);
        assert_eq!(menu.items.len(), 2);
    }

    #[test]
    fn test_menu_system() {
        let mut system = IvrMenuSystem::new();

        let menu1 = IvrMenu::new(
            "menu1".to_string(),
            "Menu 1".to_string(),
            "menu1.wav".to_string(),
        );

        let menu2 = IvrMenu::new(
            "menu2".to_string(),
            "Menu 2".to_string(),
            "menu2.wav".to_string(),
        );

        system.add_menu(menu1);
        system.add_menu(menu2);

        assert!(system.get_menu("menu1").is_some());
        assert!(system.get_menu("menu2").is_some());
        assert!(system.get_menu("menu3").is_none());

        let ids = system.list_menu_ids();
        assert_eq!(ids.len(), 2);

        system.remove_menu("menu1");
        assert!(system.get_menu("menu1").is_none());
    }

    #[test]
    fn test_default_main_menu() {
        let menu = IvrMenuSystem::create_default_main_menu();

        assert_eq!(menu.id, "main");
        assert!(menu.is_valid_digit('1'));
        assert!(menu.is_valid_digit('2'));
        assert!(menu.is_valid_digit('3'));
        assert!(menu.is_valid_digit('9'));
        assert!(menu.is_valid_digit('0'));
        assert!(menu.is_valid_digit('*'));
        assert!(!menu.is_valid_digit('#'));
    }

    #[test]
    fn test_menu_json_serialization() {
        let menu = IvrMenuBuilder::new(
            "test".to_string(),
            "Test".to_string(),
            "test.wav".to_string(),
        )
        .add_item('1', "Option 1".to_string(), MenuAction::Hangup)
        .build();

        let json = serde_json::to_string(&menu).unwrap();
        let deserialized: IvrMenu = serde_json::from_str(&json).unwrap();

        assert_eq!(menu.id, deserialized.id);
        assert_eq!(menu.items.len(), deserialized.items.len());
    }
}
