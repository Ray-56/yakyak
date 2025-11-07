/// Audio file management and organization
use crate::domain::audio::wav::{WavFile, WavError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Supported languages for audio prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// English
    En,
    /// Spanish
    Es,
    /// French
    Fr,
    /// German
    De,
    /// Chinese
    Zh,
    /// Japanese
    Ja,
    /// Korean
    Ko,
    /// Portuguese
    Pt,
    /// Russian
    Ru,
    /// Arabic
    Ar,
}

impl Language {
    /// Parse language from string code
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" | "english" => Some(Language::En),
            "es" | "spanish" => Some(Language::Es),
            "fr" | "french" => Some(Language::Fr),
            "de" | "german" => Some(Language::De),
            "zh" | "chinese" => Some(Language::Zh),
            "ja" | "japanese" => Some(Language::Ja),
            "ko" | "korean" => Some(Language::Ko),
            "pt" | "portuguese" => Some(Language::Pt),
            "ru" | "russian" => Some(Language::Ru),
            "ar" | "arabic" => Some(Language::Ar),
            _ => None,
        }
    }

    /// Get language code
    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
            Language::Zh => "zh",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Pt => "pt",
            Language::Ru => "ru",
            Language::Ar => "ar",
        }
    }

    /// Get language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Es => "Spanish",
            Language::Fr => "French",
            Language::De => "German",
            Language::Zh => "Chinese",
            Language::Ja => "Japanese",
            Language::Ko => "Korean",
            Language::Pt => "Portuguese",
            Language::Ru => "Russian",
            Language::Ar => "Arabic",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::En
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Audio file information
#[derive(Debug, Clone)]
pub struct AudioFileInfo {
    /// File identifier (e.g., "welcome", "goodbye", "ivr_main_menu")
    pub id: String,
    /// File path
    pub path: PathBuf,
    /// Language
    pub language: Language,
    /// Description
    pub description: Option<String>,
    /// Duration in seconds
    pub duration: f64,
    /// File size in bytes
    pub size: u64,
}

/// Audio file manager for organizing audio files
pub struct AudioFileManager {
    /// Audio files indexed by (language, id)
    files: HashMap<(Language, String), Arc<WavFile>>,
    /// Audio file metadata
    metadata: HashMap<(Language, String), AudioFileInfo>,
    /// Base directory for audio files
    base_dir: PathBuf,
    /// Default language
    default_language: Language,
}

impl AudioFileManager {
    /// Create new audio file manager
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            files: HashMap::new(),
            metadata: HashMap::new(),
            base_dir: base_dir.as_ref().to_path_buf(),
            default_language: Language::En,
        }
    }

    /// Set default language
    pub fn set_default_language(&mut self, language: Language) {
        self.default_language = language;
    }

    /// Get default language
    pub fn default_language(&self) -> Language {
        self.default_language
    }

    /// Register audio file
    /// File path is relative to base_dir or absolute
    pub fn register<P: AsRef<Path>>(
        &mut self,
        id: &str,
        language: Language,
        path: P,
        description: Option<String>,
    ) -> Result<(), WavError> {
        let file_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.base_dir.join(path)
        };

        // Load the WAV file
        let wav_file = WavFile::from_file(&file_path)?;
        let duration = wav_file.duration();

        // Get file size
        let size = std::fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let info = AudioFileInfo {
            id: id.to_string(),
            path: file_path,
            language,
            description,
            duration,
            size,
        };

        let key = (language, id.to_string());
        self.files.insert(key.clone(), Arc::new(wav_file));
        self.metadata.insert(key, info);

        Ok(())
    }

    /// Register audio file with automatic language detection from path
    /// Expected path format: <base_dir>/<lang>/<id>.wav
    pub fn register_auto<P: AsRef<Path>>(
        &mut self,
        path: P,
        description: Option<String>,
    ) -> Result<(), WavError> {
        let file_path = path.as_ref();

        // Extract language and id from path
        let components: Vec<&str> = file_path
            .iter()
            .filter_map(|s| s.to_str())
            .collect();

        if components.len() < 2 {
            return Err(WavError::InvalidFormat(
                "Path must contain language and filename".to_string(),
            ));
        }

        let lang_str = components[components.len() - 2];
        let filename = components[components.len() - 1];

        let language = Language::from_code(lang_str)
            .ok_or_else(|| WavError::InvalidFormat(format!("Unknown language: {}", lang_str)))?;

        let id = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| WavError::InvalidFormat("Invalid filename".to_string()))?;

        self.register(id, language, file_path, description)
    }

    /// Get audio file by ID and language
    pub fn get(&self, id: &str, language: Language) -> Option<Arc<WavFile>> {
        let key = (language, id.to_string());
        self.files.get(&key).cloned()
    }

    /// Get audio file by ID, using default language
    pub fn get_default(&self, id: &str) -> Option<Arc<WavFile>> {
        self.get(id, self.default_language)
    }

    /// Get audio file by ID with fallback to default language
    pub fn get_with_fallback(&self, id: &str, language: Language) -> Option<Arc<WavFile>> {
        self.get(id, language)
            .or_else(|| self.get(id, self.default_language))
    }

    /// Get audio file metadata
    pub fn get_info(&self, id: &str, language: Language) -> Option<&AudioFileInfo> {
        let key = (language, id.to_string());
        self.metadata.get(&key)
    }

    /// List all audio files for a language
    pub fn list_by_language(&self, language: Language) -> Vec<&AudioFileInfo> {
        self.metadata
            .iter()
            .filter(|((lang, _), _)| *lang == language)
            .map(|(_, info)| info)
            .collect()
    }

    /// List all audio file IDs
    pub fn list_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .metadata
            .keys()
            .map(|(_, id)| id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// List all available languages
    pub fn list_languages(&self) -> Vec<Language> {
        let mut langs: Vec<Language> = self
            .metadata
            .keys()
            .map(|(lang, _)| *lang)
            .collect();
        langs.sort_by_key(|l| l.code());
        langs.dedup();
        langs
    }

    /// Check if audio file exists
    pub fn exists(&self, id: &str, language: Language) -> bool {
        let key = (language, id.to_string());
        self.files.contains_key(&key)
    }

    /// Remove audio file
    pub fn remove(&mut self, id: &str, language: Language) {
        let key = (language, id.to_string());
        self.files.remove(&key);
        self.metadata.remove(&key);
    }

    /// Clear all audio files
    pub fn clear(&mut self) {
        self.files.clear();
        self.metadata.clear();
    }

    /// Get total number of audio files
    pub fn count(&self) -> usize {
        self.files.len()
    }

    /// Load all audio files from a directory structure
    /// Expected structure: <base_dir>/<lang>/*.wav
    pub fn load_directory(&mut self) -> Result<usize, WavError> {
        let mut count = 0;

        if !self.base_dir.exists() {
            return Err(WavError::IoError(format!(
                "Directory not found: {}",
                self.base_dir.display()
            )));
        }

        // Iterate through language directories
        for entry in std::fs::read_dir(&self.base_dir)
            .map_err(|e| WavError::IoError(e.to_string()))?
        {
            let entry = entry.map_err(|e| WavError::IoError(e.to_string()))?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let lang_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if let Some(language) = Language::from_code(lang_name) {
                // Load all WAV files in this language directory
                for audio_entry in std::fs::read_dir(&path)
                    .map_err(|e| WavError::IoError(e.to_string()))?
                {
                    let audio_entry = audio_entry.map_err(|e| WavError::IoError(e.to_string()))?;
                    let audio_path = audio_entry.path();

                    if audio_path.extension().and_then(|s| s.to_str()) == Some("wav") {
                        if self.register_auto(&audio_path, None).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }
}

impl Default for AudioFileManager {
    fn default() -> Self {
        Self::new("audio")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Some(Language::En));
        assert_eq!(Language::from_code("EN"), Some(Language::En));
        assert_eq!(Language::from_code("english"), Some(Language::En));
        assert_eq!(Language::from_code("es"), Some(Language::Es));
        assert_eq!(Language::from_code("unknown"), None);
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::En.code(), "en");
        assert_eq!(Language::Es.code(), "es");
        assert_eq!(Language::Fr.code(), "fr");
    }

    #[test]
    fn test_language_name() {
        assert_eq!(Language::En.name(), "English");
        assert_eq!(Language::Es.name(), "Spanish");
        assert_eq!(Language::Zh.name(), "Chinese");
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::En), "English");
        assert_eq!(format!("{}", Language::Ja), "Japanese");
    }

    #[test]
    fn test_manager_creation() {
        let manager = AudioFileManager::new("/var/audio");
        assert_eq!(manager.count(), 0);
        assert_eq!(manager.default_language(), Language::En);
    }

    #[test]
    fn test_manager_set_default_language() {
        let mut manager = AudioFileManager::new("/var/audio");
        manager.set_default_language(Language::Es);
        assert_eq!(manager.default_language(), Language::Es);
    }

    #[test]
    fn test_manager_list_operations() {
        let manager = AudioFileManager::new("/var/audio");
        assert_eq!(manager.list_ids().len(), 0);
        assert_eq!(manager.list_languages().len(), 0);
    }
}
