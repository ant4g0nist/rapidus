use crate::parser;
use crate::vm;
use crate::vm::jsvalue::value::Value;
use std::fs::OpenOptions;
use std::io::Read;

/// Load the file ("test/{file_name}.js"), execute the script,
/// and compare returned value and the given answer.
/// ### Panic
/// Panic if the returned value was different from the answer.

pub fn test_file(file_name: &str, answer: String) {
    println!("{}", format!("test/{}.js", file_name));
    compare_scripts(load_file(file_name), answer);
}

/// Load the file ("test/{file_name}.js"), and execute the script.
/// ### Panic
/// Panic if the given code returned Err.
pub fn assert_file(file_name: &str) {
    println!("{}", format!("test/{}.js", file_name));
    execute_script(load_file(file_name));
}

fn load_file(file_name: &str) -> String {
    let mut file_body = String::new();
    match OpenOptions::new()
        .read(true)
        .open(format!("test/{}.js", file_name))
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
pub fn test_code(code: String, answer: String) {
    compare_scripts(code, answer);
}

/// Execute the given code, and normally terminates only when an runtime error is returned.
/// ### Panic
/// Panic if the code returned a parse error, or terminated without error.
pub fn runtime_error(text: &str) -> String {
    let mut vm = vm::vm::VM::new();

    let mut parser = parser::Parser::new(text.to_string());
    let node = parser.parse_all().unwrap();
    let mut iseq = vec![];

    let func_info = vm.compile(&node, &mut iseq, true, 0).unwrap();
    match vm.run_global(func_info, iseq) {
        Ok(()) => panic!(),
        Err(err) => return format!("{:?}", err),
    };
}

/// Execute the given code.
/// ### Panic
/// Panic if the given code returned Err.
pub fn execute_script(text: String) -> String {
    let mut vm = vm::vm::VM::new();

    let mut parser = parser::Parser::new(text);
    let node = parser.parse_all().unwrap();
    let mut iseq = vec![];

    let func_info = vm.compile(&node, &mut iseq, true, 0).unwrap();
    vm.run_global(func_info, iseq).unwrap();
    let val: Value = vm.stack.pop().unwrap_or(Value::undefined().into()).into();
    val.debug_string(true)
}

fn compare_scripts(text: String, answer: String) {
    let res_text = execute_script(text);
    println!("file: {}", res_text);

    let res_answer = execute_script(answer);
    println!("ans:  {}", res_answer);

    assert_eq!(res_text, res_answer);
}
