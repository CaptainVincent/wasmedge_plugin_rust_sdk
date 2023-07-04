use std::io::Write;

use wasmedge_plugin_sdk::{
    error::CoreError,
    memory::Memory,
    module::{PluginModule, SyncInstanceRef},
    types::{ValType, WasmVal},
};

#[derive(Debug)]
pub enum PluginError {
    ParamError,
    MemoryError,
    UTF8Error,
}

pub fn create_module() -> PluginModule<()> {
    fn to_uppercase<'a>(
        _inst_ref: &'a mut SyncInstanceRef,
        main_memory: &'a mut Memory,
        _data: &'a mut (),
        args: Vec<WasmVal>,
    ) -> Result<Vec<WasmVal>, CoreError> {
        fn to_uppercase_(
            main_memory: &mut Memory,
            data_ptr: &WasmVal,
            data_len: &WasmVal,
        ) -> Result<(), PluginError> {
            if let (WasmVal::I32(data_ptr), WasmVal::I32(data_len)) = (data_ptr, data_len) {
                let mut bytes = main_memory
                    .data_pointer_mut(*data_ptr as usize, *data_len as usize)
                    .ok_or(PluginError::MemoryError)?;

                let uppercase = std::str::from_utf8_mut(bytes)
                    .map_err(|_| PluginError::UTF8Error)?
                    .to_uppercase();

                let _ = bytes.write_all(uppercase.as_bytes());

                Ok(())
            } else {
                Err(PluginError::ParamError)
            }
        }

        match to_uppercase_(main_memory, &args[0], &args[1]) {
            Ok(_) => Ok(vec![WasmVal::I32(0)]),
            Err(PluginError::ParamError) => Ok(vec![WasmVal::I32(-1)]),
            Err(PluginError::MemoryError) => Ok(vec![WasmVal::I32(-2)]),
            Err(PluginError::UTF8Error) => Ok(vec![WasmVal::I32(-3)]),
        }
    }

    fn get_data<'a>(
        inst_ref: &'a mut SyncInstanceRef,
        main_memory: &'a mut Memory,
        _data: &'a mut (),
        _args: Vec<WasmVal>,
    ) -> Result<Vec<WasmVal>, CoreError> {
        let data = "a string return from plugin\0";
        let result = inst_ref.call("malloc", vec![WasmVal::I32(data.len() as i32)])?;
        if let Some(WasmVal::I32(data_ptr)) = result.first() {
            let data_ptr = *data_ptr;
            main_memory.write_bytes(data, data_ptr as u32)?;
            Ok(vec![WasmVal::I32(data_ptr)])
        } else {
            Ok(vec![WasmVal::I32(-1)])
        }
    }

    let mut module = PluginModule::create("memory_access_module", ()).unwrap();

    module
        .add_func(
            "to_uppercase",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            to_uppercase,
        )
        .unwrap();

    module
        .add_func("get_data", (vec![], vec![ValType::I32]), get_data)
        .unwrap();

    module
}

wasmedge_plugin_sdk::plugin::register_plugin!(
    plugin_name="memory_access_plugin",
    plugin_description="a demo plugin",
    version=(0,0,0,0),
    modules=[
        {"memory_access_module","a demo of module",create_module}
    ]
);
