use core::str;

pub enum Command {
    Left(u32),
    Right(u32),
}

const BUFFER_SIZE: usize = 10;
type BufferType = [u8; BUFFER_SIZE];
pub struct SerialCommands {
    buffer: BufferType,
    position: usize,
}

const CARRIAGE_RETURN_VALUE: u8 = 13;
impl SerialCommands {
    pub fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            position: 0,
        }
    }

    pub fn add_character(&mut self, data: u8) {
        self.store_character(data);
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

            // Find the last integer
            let integer_count = parse_buffer
                .iter()
                .rev()
                .skip(1) // Skipp CR.
                .take_while(|data| data.is_ascii_digit())
                .count();

            //Parse the last integer part
            let parse_buffer =
                str::from_utf8(&parse_buffer[BUFFER_SIZE - 1 - integer_count..BUFFER_SIZE - 1])
                    .unwrap();
            let _number: u32 = parse_buffer.parse().unwrap();

            // Clear the buffer
            self.buffer.iter_mut().for_each(|x| *x = 0);
        }
    }

    // const LEFT: &str = "left";
    // const RIGHT &str = "right";
    fn match_command(&mut self, parse_buffer: &[u8]) {
        //let a = parse_buffer.iter().rmatches(char::is_numeric).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_more_characters_than_buffer_size() {
        // let callbak = |command|
        // {
        // };

        let mut serial_commands = SerialCommands::new();
        for data in [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', '1', '2', '3', '4', '5', '6', '7',
        ]
        .iter()
        {
            serial_commands.add_character(*data as u8);
        }
    }
    //assert_eq!(bad_add(1, 2), 3);
}
