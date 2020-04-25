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
    pub fn add_character(&mut self, data: u8) {
        self.buffer[self.next_position] = data;
        
        self.last_position = self.next_position;
        self.next_position += 1;

        if self.next_position >= BUFFER_SIZE {
            self.next_position = 0;  
        }
    }

    pub fn last_char(&self) -> u8 {
        self.buffer[self.last_position]
    }

    pub fn create_arranged_buffer(&self) -> BufferType {
        let mut parse_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let left = &self.buffer[..self.last_position];
        let right = &self.buffer[self.last_position..];

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
    Left(u32),
    Right(u32),
}

pub struct SerialCommands
{
    buffer: Buffer,
    //command: Option<Command>,
}

impl Default for SerialCommands {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            //command: None,
        }
    }
}

enum CommandValueType {
    IntType,
    NoType,
}

struct CommandOption {
    label: &'static str,
    value_type: CommandValueType,
}

const CARRIAGE_RETURN_VALUE: u8 = b'\r';
const COMMANDS_AVAILABLE: [CommandOption; 2] = [
    CommandOption{label:"left", value_type:CommandValueType::NoType},
    CommandOption{label:"right", value_type:CommandValueType::NoType},
];

//const LEFT: &str = "left";
//const RIGHT: &str = "right";
impl SerialCommands
    //where CB: FnMut(Command) -> ()
{
    pub fn add_character(&mut self, data: u8) {
        self.buffer.add_character(data);
    }

    pub fn get_command(&mut self) -> Option<Command> {
        if self.buffer.last_char() == CARRIAGE_RETURN_VALUE {
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
            let command_parts = parse_buffer.split_terminator(
                |c:char| !c.is_ascii_digit() && !c.is_ascii_alphanumeric());
            for command in command_parts {
                for available_commands in &COMMANDS_AVAILABLE
                {
                    if available_commands.label == command
                    {
                        return Some(Command::Left(0));
                    }
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
    fn more_than_buffer_size_no_panic() {
        let mut serial_commands = SerialCommands::default();
        
        for data in 0..100 {
            serial_commands.add_character(data);
        }
    }
    #[test]
    fn parse_command() {
        // Register for the expected command
        let mut serial_commands = SerialCommands::default();
        
        for data in b"left\r" {
            serial_commands.add_character(*data);
        }

        assert_eq!(Some(Command::Left(0)), serial_commands.get_command());
        assert_eq!(None, serial_commands.get_command());
    }

    // #[test]
    // fn check_arranged_buffer() {
    //     // Register for the expected command
    //     let mut serial_commands = SerialCommands::default();
        
    //     for data in b"left\r" {
    //         serial_commands.add_character(*data);
    //     }

    //     assert_eq!(b"left\r", serial_commands.get_buffer());
    // }


    #[test]
    fn parse_command_leading_chars() {
        // Register for the expected command
        let mut serial_commands = SerialCommands::default();
        
        for data in b"le _ 1 left\r" {
            serial_commands.add_character(*data);
        }

        assert_eq!(Some(Command::Left(0)), serial_commands.get_command());
        assert_eq!(None, serial_commands.get_command());
    }


    #[test]
    fn debug_stuff() {
        let parse_buffer = "left\r";
        //let mut command_parts = parse_buffer.split_terminator(' ');
         let command_parts = parse_buffer.split(
             |c:char| !c.is_ascii_digit() && !c.is_ascii_alphanumeric());
    
        assert_eq!("left", command_parts.collect::<String>());
    }

    // #[test]
    // fn parse_command_with_value() {
    //     // Register for the expected command
    //     let mut serial_commands = SerialCommands::default();
        
    //     for data in b"left 100\r" {
    //         serial_commands.add_character(*data);
    //     }

    //     assert_eq!(Some(Command::Left(100)), serial_commands.get_command());
    // }
}
