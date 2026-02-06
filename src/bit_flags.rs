use std::collections::HashMap;

pub struct BitFlagContainer {
    flags: u64,
    map: HashMap<String, u8>,
    next_index: u8,
}

#[derive(Debug)]
pub enum FlagError {
    NotFound(String),
    Full,
}

impl BitFlagContainer {
    pub fn new() -> Self {
        Self {
            flags: 0,
            map: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn create_flag(&mut self, key: &str, value: bool) -> Result<(), FlagError> {
        if self.next_index >= 64 {
            return Err(FlagError::Full);
        }

        let index = self.next_index;
        self.next_index += 1;

        self.map.insert(key.to_string(), index);
        self.set_value(index, value);

        Ok(())
    }

    pub fn set_flag(&mut self, key: &str, value: bool) -> Result<(), FlagError> {
        let index = self.get_index(key)?;
        self.set_value(index, value);
        Ok(())
    }

    pub fn get_flag(&self, key: &str) -> Result<bool, FlagError> {
        let index = self.get_index(key)?;
        Ok(self.get_value(index))
    }

    fn get_index(&self, key: &str) -> Result<u8, FlagError> {
        self.map
            .get(key)
            .copied()
            .ok_or_else(|| FlagError::NotFound(key.to_string()))
    }

    fn set_value(&mut self, index: u8, value: bool) {
        let mask = 1u64 << index;
        if value {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }

    fn get_value(&self, index: u8) -> bool {
        (self.flags & (1u64 << index)) != 0
    }
}
