/// DTMF (Dual-Tone Multi-Frequency) detection and handling
use std::time::{Duration, Instant};

/// DTMF digit representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DtmfDigit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Star,
    Pound,
}

impl DtmfDigit {
    /// Convert to character representation
    pub fn to_char(&self) -> char {
        match self {
            DtmfDigit::Zero => '0',
            DtmfDigit::One => '1',
            DtmfDigit::Two => '2',
            DtmfDigit::Three => '3',
            DtmfDigit::Four => '4',
            DtmfDigit::Five => '5',
            DtmfDigit::Six => '6',
            DtmfDigit::Seven => '7',
            DtmfDigit::Eight => '8',
            DtmfDigit::Nine => '9',
            DtmfDigit::Star => '*',
            DtmfDigit::Pound => '#',
        }
    }

    /// Parse from character
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(DtmfDigit::Zero),
            '1' => Some(DtmfDigit::One),
            '2' => Some(DtmfDigit::Two),
            '3' => Some(DtmfDigit::Three),
            '4' => Some(DtmfDigit::Four),
            '5' => Some(DtmfDigit::Five),
            '6' => Some(DtmfDigit::Six),
            '7' => Some(DtmfDigit::Seven),
            '8' => Some(DtmfDigit::Eight),
            '9' => Some(DtmfDigit::Nine),
            '*' => Some(DtmfDigit::Star),
            '#' => Some(DtmfDigit::Pound),
            _ => None,
        }
    }

    /// Get DTMF frequencies (low and high)
    pub fn frequencies(&self) -> (u32, u32) {
        match self {
            DtmfDigit::One => (697, 1209),
            DtmfDigit::Two => (697, 1336),
            DtmfDigit::Three => (697, 1477),
            DtmfDigit::Four => (770, 1209),
            DtmfDigit::Five => (770, 1336),
            DtmfDigit::Six => (770, 1477),
            DtmfDigit::Seven => (852, 1209),
            DtmfDigit::Eight => (852, 1336),
            DtmfDigit::Nine => (852, 1477),
            DtmfDigit::Star => (941, 1209),
            DtmfDigit::Zero => (941, 1336),
            DtmfDigit::Pound => (941, 1477),
        }
    }
}

/// DTMF event from SIP INFO or RFC 2833
#[derive(Debug, Clone)]
pub struct DtmfEvent {
    pub digit: DtmfDigit,
    pub duration: Duration,
    pub timestamp: Instant,
}

impl DtmfEvent {
    pub fn new(digit: DtmfDigit, duration: Duration) -> Self {
        Self {
            digit,
            duration,
            timestamp: Instant::now(),
        }
    }

    /// Create from RFC 2833 payload type and event code
    pub fn from_rfc2833(event_code: u8, duration_ms: u16) -> Option<Self> {
        let digit = match event_code {
            0 => DtmfDigit::Zero,
            1 => DtmfDigit::One,
            2 => DtmfDigit::Two,
            3 => DtmfDigit::Three,
            4 => DtmfDigit::Four,
            5 => DtmfDigit::Five,
            6 => DtmfDigit::Six,
            7 => DtmfDigit::Seven,
            8 => DtmfDigit::Eight,
            9 => DtmfDigit::Nine,
            10 => DtmfDigit::Star,
            11 => DtmfDigit::Pound,
            _ => return None,
        };

        Some(Self::new(digit, Duration::from_millis(duration_ms as u64)))
    }
}

/// DTMF detector with buffer and timeout
pub struct DtmfDetector {
    buffer: Vec<DtmfDigit>,
    max_buffer_size: usize,
    digit_timeout: Duration,
    last_digit_time: Option<Instant>,
}

impl DtmfDetector {
    /// Create new DTMF detector
    pub fn new(max_buffer_size: usize, digit_timeout: Duration) -> Self {
        Self {
            buffer: Vec::new(),
            max_buffer_size,
            digit_timeout,
            last_digit_time: None,
        }
    }

    /// Create with default settings (20 digits, 5 second timeout)
    pub fn default_settings() -> Self {
        Self::new(20, Duration::from_secs(5))
    }

    /// Process a DTMF event
    pub fn process_event(&mut self, event: DtmfEvent) {
        // Check if timeout has occurred since last digit
        if let Some(last_time) = self.last_digit_time {
            if event.timestamp.duration_since(last_time) > self.digit_timeout {
                // Timeout - clear buffer
                self.buffer.clear();
            }
        }

        // Add digit to buffer
        if self.buffer.len() < self.max_buffer_size {
            self.buffer.push(event.digit);
        }

        self.last_digit_time = Some(event.timestamp);
    }

    /// Get current buffer as string
    pub fn get_buffer(&self) -> String {
        self.buffer.iter().map(|d| d.to_char()).collect()
    }

    /// Clear buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.last_digit_time = None;
    }

    /// Get last N digits
    pub fn get_last_digits(&self, n: usize) -> String {
        let start = if self.buffer.len() > n {
            self.buffer.len() - n
        } else {
            0
        };
        self.buffer[start..].iter().map(|d| d.to_char()).collect()
    }

    /// Check if buffer matches a pattern
    pub fn matches(&self, pattern: &str) -> bool {
        self.get_buffer() == pattern
    }

    /// Check if buffer ends with a pattern
    pub fn ends_with(&self, pattern: &str) -> bool {
        self.get_buffer().ends_with(pattern)
    }

    /// Get buffer length
    pub fn buffer_length(&self) -> usize {
        self.buffer.len()
    }

    /// Check if timeout has occurred
    pub fn is_timeout(&self) -> bool {
        if let Some(last_time) = self.last_digit_time {
            Instant::now().duration_since(last_time) > self.digit_timeout
        } else {
            false
        }
    }
}

/// DTMF parser for SIP INFO body
pub struct DtmfParser;

impl DtmfParser {
    /// Parse DTMF from SIP INFO application/dtmf-relay body
    /// Format: "Signal=1\r\nDuration=160\r\n"
    pub fn parse_sip_info(body: &str) -> Option<DtmfEvent> {
        let mut signal: Option<char> = None;
        let mut duration_ms: u16 = 100; // Default duration

        for line in body.lines() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "Signal" => signal = value.chars().next(),
                "Duration" => {
                    if let Ok(d) = value.parse::<u16>() {
                        duration_ms = d;
                    }
                }
                _ => {}
            }
        }

        if let Some(c) = signal {
            if let Some(digit) = DtmfDigit::from_char(c) {
                return Some(DtmfEvent::new(digit, Duration::from_millis(duration_ms as u64)));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtmf_digit_conversion() {
        assert_eq!(DtmfDigit::Zero.to_char(), '0');
        assert_eq!(DtmfDigit::Nine.to_char(), '9');
        assert_eq!(DtmfDigit::Star.to_char(), '*');
        assert_eq!(DtmfDigit::Pound.to_char(), '#');

        assert_eq!(DtmfDigit::from_char('0'), Some(DtmfDigit::Zero));
        assert_eq!(DtmfDigit::from_char('5'), Some(DtmfDigit::Five));
        assert_eq!(DtmfDigit::from_char('*'), Some(DtmfDigit::Star));
        assert_eq!(DtmfDigit::from_char('#'), Some(DtmfDigit::Pound));
        assert_eq!(DtmfDigit::from_char('x'), None);
    }

    #[test]
    fn test_dtmf_frequencies() {
        assert_eq!(DtmfDigit::One.frequencies(), (697, 1209));
        assert_eq!(DtmfDigit::Five.frequencies(), (770, 1336));
        assert_eq!(DtmfDigit::Nine.frequencies(), (852, 1477));
        assert_eq!(DtmfDigit::Zero.frequencies(), (941, 1336));
    }

    #[test]
    fn test_dtmf_detector() {
        let mut detector = DtmfDetector::default_settings();

        let event1 = DtmfEvent::new(DtmfDigit::One, Duration::from_millis(100));
        let event2 = DtmfEvent::new(DtmfDigit::Two, Duration::from_millis(100));
        let event3 = DtmfEvent::new(DtmfDigit::Three, Duration::from_millis(100));

        detector.process_event(event1);
        detector.process_event(event2);
        detector.process_event(event3);

        assert_eq!(detector.get_buffer(), "123");
        assert_eq!(detector.buffer_length(), 3);
        assert!(detector.matches("123"));
        assert!(detector.ends_with("23"));

        detector.clear_buffer();
        assert_eq!(detector.get_buffer(), "");
        assert_eq!(detector.buffer_length(), 0);
    }

    #[test]
    fn test_dtmf_parser() {
        let body = "Signal=5\r\nDuration=160\r\n";
        let event = DtmfParser::parse_sip_info(body).unwrap();

        assert_eq!(event.digit, DtmfDigit::Five);
        assert_eq!(event.duration, Duration::from_millis(160));
    }

    #[test]
    fn test_dtmf_rfc2833() {
        let event = DtmfEvent::from_rfc2833(5, 160).unwrap();
        assert_eq!(event.digit, DtmfDigit::Five);
        assert_eq!(event.duration, Duration::from_millis(160));

        let star_event = DtmfEvent::from_rfc2833(10, 100).unwrap();
        assert_eq!(star_event.digit, DtmfDigit::Star);

        let pound_event = DtmfEvent::from_rfc2833(11, 100).unwrap();
        assert_eq!(pound_event.digit, DtmfDigit::Pound);

        let invalid = DtmfEvent::from_rfc2833(99, 100);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_get_last_digits() {
        let mut detector = DtmfDetector::default_settings();

        for digit in ['1', '2', '3', '4', '5'] {
            let d = DtmfDigit::from_char(digit).unwrap();
            detector.process_event(DtmfEvent::new(d, Duration::from_millis(100)));
        }

        assert_eq!(detector.get_last_digits(3), "345");
        assert_eq!(detector.get_last_digits(10), "12345"); // More than available
        assert_eq!(detector.get_last_digits(0), "");
    }
}
