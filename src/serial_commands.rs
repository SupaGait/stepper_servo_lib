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

const CARRIAGE_RETURN_VALUE: u8 = b'\r';
const LEFT: &str = "left";
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
            right
                .iter()
                .chain(left.iter())
                .zip(parse_buffer.iter_mut())
                .for_each(|(data, parse_buffer)| *parse_buffer = *data);

            if let Ok(parse_buffer) = str::from_utf8(&parse_buffer) {
                let command_parts = parse_buffer.split_terminator(' ');
                for command in command_parts {
                    if let Some(_) = command.matches(LEFT).next() {
                        self.command = Some(Command::Left(0));
                    }
                }
            }

            // Clear the buffer
            self.buffer.iter_mut().for_each(|x| *x = 0);
        }
    }

    //fn match_command(&mut self, parse_buffer: &[u8]) {
        //let a = parse_buffer.iter().rmatches(char::is_numeric).collect();
    //}
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
