use crate::parser;
use crate::vm;
use crate::vm::jsvalue::value::Value;
use std::fs::OpenOptions;
use std::io::Read;

/// Load the file ("test/{file_name}.js"), execute the script,
/// and compare returned value and the given answer.
/// ### Panic
/// Panic if the returned value was different from the answer.

pub fn test_file(file_name: impl Into<String>, answer: impl Into<String>) {
    let file_name = file_name.into();
    println!("{}", format!("test/{}.js", file_name));
    compare_scripts(load_file(file_name), answer.into());
}

/// Load the file ("test/{file_name}.js"), and execute the script.
/// ### Panic
/// Panic if the given code returned Err.
pub fn assert_file(file_name: &str) {
    println!("{}", format!("test/{}.js", file_name));
    execute_script(load_file(file_name));
}

fn load_file(file_name: impl Into<String>) -> String {
    let mut file_body = String::new();
    match OpenOptions::new()
        .read(true)
        .open(format!("test/{}.js", file_name.into()))
    {
        Ok(mut file) => match file.read_to_string(&mut file_body).ok() {
            Some(x) => x,
            None => panic!("Couldn't read the file"),
        },
        Err(_) => panic!("file not found"),
    };
    file_body
}

/// Execute the given code, and compare returned value and the given answer.
/// ### Panic
/// Panic if the returned value was different from the answer.
pub fn test_code(code: impl Into<String>, answer: impl Into<String>) {
    compare_scripts(code.into(), answer.into());
}

/// Execute the given code, and normally terminates only when an runtime error is returned.
/// ### Panic
/// Panic if the code returned a parse error, or terminated without error.
pub fn runtime_error(text: &str) -> String {
    let mut vm = vm::vm::VM::new();

    let mut parser = parser::Parser::new("test", text);
    let node = parser.parse_all().unwrap();

    let func_info = vm.compile(&node, true).unwrap();
    match vm.run_global(func_info) {
        Ok(()) => panic!(),
        Err(err) => return format!("{:?}", err),
    };
}

/// Execute the given code.
/// ### Panic
/// Panic if the given code returned Err.
pub fn execute_script(text: String) -> String {
    let mut vm = vm::vm::VM::new();

    let mut parser = parser::Parser::new("test", text);
    let node = parser.parse_all().unwrap();
    let func_info = vm.compile(&node, true).unwrap();
    vm.run_global(func_info).unwrap();
    let val: Value = vm
        .current_context
        .stack
        .pop()
        .unwrap_or(Value::undefined().into())
        .into();
    val.debug_string(true)
}

fn compare_scripts(text: String, answer: String) {
    let res_text = execute_script(text);
    println!("file: {}", res_text);

    let res_answer = execute_script(answer);
    println!("ans:  {}", res_answer);

    assert_eq!(res_text, res_answer);
}
