use core::str::FromStr;
use core::str;

const BUFFER_SIZE: usize = 20;
type BufferType = [u8; BUFFER_SIZE]; 
struct Buffer {
    buffer: BufferType,
    next_position: usize,
    last_position: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            next_position: 0,
            last_position: 0,
        }
    }
}

impl Buffer
{
    pub fn add_byte(&mut self, data: u8) {
        self.buffer[self.next_position] = data;
        
        self.last_position = self.next_position;
        self.next_position += 1;

        if self.next_position >= BUFFER_SIZE {
            self.next_position = 0;  
        }
    }

    pub fn last_byte(&self) -> u8 {
        self.buffer[self.last_position]
    }

    pub fn create_arranged_buffer(&self) -> BufferType {
        let mut parse_buffer: BufferType = [0; BUFFER_SIZE];

        let left = &self.buffer[..self.next_position];
        let right = &self.buffer[self.next_position..];

        // Arrange into the parse buffer.
        for (global_buffer, parse_buffer) in right.iter().chain(left.iter())
            .zip(parse_buffer.iter_mut()) {
                *parse_buffer = *global_buffer;
        }
        parse_buffer
    }

    pub fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|x| *x = 0);
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Command {
    Stop,
    Left { speed: i32},
    Right { speed: i32},
}

impl Command {
    pub fn parse_from<'a, I>(mut command: I) -> Option<Self>
    where
        I: Iterator<Item = &'a str>,
    {
        match command.next() {
            Some("stop") => Some(Command::Stop),
            Some("left") => Some(Command::Left{ speed: Command::with_value(command)?}),
            Some("right") => Some(Command::Right{ speed: Command::with_value(command)?}),
            _ => None,
        }
    }

    fn with_value<'a, I>(mut command: I) -> Option<i32>
    where
        I: Iterator<Item = &'a str>,
    {
        i32::from_str(command.next()?).ok()
    }
}

pub struct SerialCommands
{
    buffer: Buffer,
}

impl Default for SerialCommands {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
        }
    }
}

const ASCII_CR: u8 = b'\r';
impl SerialCommands
{
    pub fn add_character(&mut self, data: u8) {
        self.buffer.add_byte(data);
    }

    pub fn get_command(&mut self) -> Option<Command> {
        if self.buffer.last_byte() == ASCII_CR {
            let parse_buffer = self.buffer.create_arranged_buffer();

            let command = self.parse_commands(&parse_buffer);
            if command.is_some()
            {
                self.buffer.reset();
                return command;
            }
        }
        None
    }

    fn parse_commands(&self, buffer: &[u8]) -> Option<Command>{
        if let Ok(parse_buffer) = str::from_utf8(buffer) {
            let mut command_parts = parse_buffer.split_terminator(|c:char| 
                !c.is_ascii_digit() && 
                !c.is_ascii_alphanumeric() && 
                !c.is_ascii_punctuation());

            while let Some(_) = command_parts.next()
            {
                let command = Command::parse_from(command_parts.clone());
                if command.is_some()
                {
                    return command
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer() {
        let mut buffer = Buffer::default();
        
        for data in b"0123456789_abcdefghi_012345" {
            buffer.add_byte(*data);
        }
    
        assert_eq!(b'5', buffer.last_byte());
        assert_eq!(Ok("789_abcdefghi_012345"), str::from_utf8(&buffer.create_arranged_buffer()));
    }

    #[test]
    fn more_than_buffer_size_no_panic() {
        let mut serial_commands = SerialCommands::default();
        
        for data in 0..100 {
            serial_commands.add_character(data);
        }
    }

    #[test]
    fn command_parsing() {
        let data = "left 100".split_whitespace();
        let command = Command::parse_from(data);
        assert_eq!(Some(Command::Left{speed:100}), command);

        let data = "right -5".split_whitespace();
        let command = Command::parse_from(data);
        assert_eq!(Some(Command::Right{speed:-5}), command);
    }

    #[test]
    fn parse_single_command() {
        // Register for the expected command
        let mut serial_commands = SerialCommands::default();
        
        for data in b"stop\r" {
            serial_commands.add_character(*data);
        }

        assert_eq!(Some(Command::Stop), serial_commands.get_command());
        assert_eq!(None, serial_commands.get_command());
    }

    #[test]
    fn parse_command_leading_chars() {
        let mut serial_commands = SerialCommands::default();
        for data in b"le _ 1 stop\r" {
            serial_commands.add_character(*data);
        }

        assert_eq!(Some(Command::Stop), serial_commands.get_command());
        assert_eq!(None, serial_commands.get_command());
    }

    #[test]
    fn parse_command_with_value() {
        // Register for the expected command
        let mut serial_commands = SerialCommands::default();
        
        for data in b"left 100\r" {
            serial_commands.add_character(*data);
        }

        assert_eq!(Some(Command::Left{speed: 100}), serial_commands.get_command());
    }
}
