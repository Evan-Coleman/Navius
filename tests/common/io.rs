use std::io::{self, Cursor, Read, Write};

/// MockIO provides a simulated IO environment for testing CLI interactions
/// without requiring real terminal input.
pub struct MockIO {
    /// Simulated input data
    pub input: Cursor<Vec<u8>>,
    /// Captured output data
    pub output: Vec<u8>,
}

impl MockIO {
    /// Create a new MockIO with the given input data
    pub fn new(input_data: &[u8]) -> Self {
        Self {
            input: Cursor::new(input_data.to_vec()),
            output: Vec::new(),
        }
    }

    /// Create a MockIO with input from a string
    pub fn with_string(input: &str) -> Self {
        Self::new(input.as_bytes())
    }

    /// Create a MockIO with simulated keypress/arrow input
    pub fn with_key_sequence(keys: &[Key]) -> Self {
        let mut data = Vec::new();
        for key in keys {
            let bytes = key.as_bytes();
            data.extend_from_slice(&bytes);
        }
        Self::new(&data)
    }

    /// Get the captured output as a string
    pub fn get_output_as_string(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }

    /// Check if the captured output contains a specific string
    pub fn output_contains(&self, text: &str) -> bool {
        self.get_output_as_string().contains(text)
    }

    /// Add more input to the mock
    pub fn add_input(&mut self, input: &[u8]) {
        let position = self.input.position();
        let mut current_data = self.input.get_ref().clone();
        current_data.extend_from_slice(input);
        self.input = Cursor::new(current_data);
        self.input.set_position(position);
    }

    /// Add string input to the mock
    pub fn add_string_input(&mut self, input: &str) {
        self.add_input(input.as_bytes());
    }

    /// Clear the captured output
    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}

impl Read for MockIO {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for MockIO {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Special key representations for testing terminal input
#[derive(Debug, Clone)]
pub enum Key {
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Char(char),
}

impl Key {
    /// Convert the key to its byte representation
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Key::Enter => vec![b'\n'],
            Key::Escape => vec![27],
            Key::Space => vec![b' '],
            Key::Tab => vec![b'\t'],
            Key::Backspace => vec![127],
            Key::ArrowUp => vec![27, 91, 65],
            Key::ArrowDown => vec![27, 91, 66],
            Key::ArrowLeft => vec![27, 91, 68],
            Key::ArrowRight => vec![27, 91, 67],
            Key::Char(c) => vec![*c as u8],
        }
    }
}

/// Simulates pressing arrow keys a certain number of times
pub fn press_arrow_key(key: Key, count: usize) -> Vec<Key> {
    match key {
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            vec![key; count]
        }
        _ => panic!("Not an arrow key: {:?}", key),
    }
}

/// Creates a key sequence for selecting an option in a menu
pub fn select_option(current_position: usize, target_position: usize) -> Vec<Key> {
    let mut keys = Vec::new();

    if current_position < target_position {
        // Need to move down
        keys.extend(press_arrow_key(
            Key::ArrowDown,
            target_position - current_position,
        ));
    } else if current_position > target_position {
        // Need to move up
        keys.extend(press_arrow_key(
            Key::ArrowUp,
            current_position - target_position,
        ));
    }

    // Add Enter to select the option
    keys.push(Key::Enter);

    keys
}
