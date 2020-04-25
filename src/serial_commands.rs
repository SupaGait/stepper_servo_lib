use core::str;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Command {
    Left(u32),
    Right(u32),
}

const BUFFER_SIZE: usize = 20;
type BufferType = [u8; BUFFER_SIZE];
pub struct SerialCommands
    //where CB: FnMut(Command) -> ()
{
    buffer: BufferType,
    position: usize,
    command: Option<Command>,
    //callback: CB,
}

impl Default for SerialCommands {
    fn default() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            position: 0,
            command: None,
            //callback,
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
        self.store_character(data);
    }

    pub fn get_command(&mut self) -> Option<Command>
    {
        let command = self.command.as_ref()?;
        let command = command.clone();
        self.command = None;

        Some(command)
    }

    fn store_character(&mut self, data: u8) {
        self.buffer[self.position] = data;
        self.check_for_command();

        // Increment buffer
        self.position += 1;

        if self.position >= BUFFER_SIZE {
            self.position = 0;  
        }
    }

    fn check_for_command(&mut self) {
        if self.buffer[self.position] == CARRIAGE_RETURN_VALUE {
            let mut parse_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

            let left = &self.buffer[..self.position];
            let right = &self.buffer[self.position..];

            // Arrange into the parse buffer.
            for (global_buffer, parse_buffer) in right.iter().chain(left.iter())
                .zip(parse_buffer.iter_mut()) {
                    *parse_buffer = *global_buffer;
            }

            // Parse
            self.parse_commands(&parse_buffer);

            // Clear the buffer
            self.buffer.iter_mut().for_each(|x| *x = 0);
        }
    }

    pub fn get_buffer(&self) ->&[u8] {
        &self.buffer
    }

    fn parse_commands(&mut self, buffer: &[u8]) {
        if let Ok(parse_buffer) = str::from_utf8(buffer) {
            let command_parts = parse_buffer.split_terminator(
                |c:char| !c.is_ascii_digit() && !c.is_ascii_alphanumeric());
            for command in command_parts {
                for available_commands in &COMMANDS_AVAILABLE
                {
                    if available_commands.label == command
                    {
                        self.command = Some(Command::Left(0));
                    }
                }
            }
        }

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
