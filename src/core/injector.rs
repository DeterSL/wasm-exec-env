use wasmtime::{Store, Memory};

pub struct Injector<'a, T>
where
    T: 'static,
{
    store: &'a mut Store<T>,
    memory: Memory,
}

impl<'a, T> Injector<'a, T>
where
    T: 'static,
{
    pub fn new(store: &'a mut Store<T>, memory: Memory) -> Self {
        Injector { store, memory }
    }

    pub fn inject_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        /*let mem_data = self.memory.data_mut(self.store);*/
        /*if offset + data.len() > mem_data.len() {*/
            /*return Err(format!(*/
                /*"Memory access out of bounds: offset {} + data length {} exceeds memory size {}",*/
                /*offset,*/
                /*data.len(),*/
                /*mem_data.len()*/
            /*));*/
        /*}*/
        /*mem_data[offset..offset + data.len()].copy_from_slice(data);*/
        Ok(())
    }

    pub fn inject_string(&mut self, offset: usize, s: &str) -> Result<(), String> {
        self.inject_bytes(offset, s.as_bytes())
    }
}
