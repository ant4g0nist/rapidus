extern crate rapidus;
use rapidus::parser;
use rapidus::{vm, vm::exec_context, vm::jsvalue::value::Value, vm::vm::VM};

extern crate libc;

extern crate rustyline;

extern crate clap;
use clap::{App, Arg};

const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let app = App::new("Rapidus")
        .version(VERSION_STR)
        .author("uint256_t")
        .about("A toy JavaScript engine")
        .arg(
            Arg::with_name("debug")
                .help("Show useful information for debugging")
                .long("debug"),
        )
        .arg(
            Arg::with_name("trace")
                .help("Trace bytecode execution for debugging")
                .long("trace"),
        )
        .arg(Arg::with_name("file").help("Input file name").index(1));
    let app_matches = app.clone().get_matches();
    let is_debug = app_matches.is_present("debug");
    let is_trace = app_matches.is_present("trace");
    let file_name = match app_matches.value_of("file") {
        Some(file_name) => file_name,
        None => {
            repl(is_trace);
            return;
        }
    };

    let mut parser = match parser::Parser::load_module(file_name.clone()) {
        Ok(ok) => ok,
        Err(_) => return,
    };

    let node = match parser.parse_all() {
        Ok(ok) => ok,
        Err(err) => {
            parser.handle_error(&err);
            return;
        }
    };
    if is_debug {
        println!("Parser:");
        println!("{:?}", node);
    };

    let mut vm = VM::new();
    if is_trace {
        vm = vm.trace();
    }
    let mut iseq = vec![];
    let global_info = match vm.compile(&node, &mut iseq, false, 0) {
        Ok(ok) => ok,
        Err(vm::codegen::Error { msg, token_pos, .. }) => {
            parser.show_error_at(token_pos, msg);
            return;
        }
    };

    if is_debug {
        println!("Codegen:");
        rapidus::bytecode_gen::show_inst_seq(&iseq, &vm.constant_table);
    };

    let script_info = parser.into_script_info();
    vm.script_info.push((0, script_info));
    if let Err(e) = vm.run_global(global_info, iseq) {
        vm.show_error_message(e);
    }

    if is_debug {
        for (i, val_boxed) in vm.current_context.stack.iter().enumerate() {
            let val: Value = (*val_boxed).into();
            println!("stack remaining: [{}]: {:?}", i, val);
        }
    }
}

fn repl(is_trace: bool) {
    let mut rl = rustyline::Editor::<()>::new();
    let mut vm = VM::new();
    if is_trace {
        vm = vm.trace();
    }
    let mut global_context: Option<exec_context::ExecContext> = None;

    loop {
        let mut parser;

        let line = if let Ok(line) = rl.readline("> ") {
            line
        } else {
            break;
        };

        rl.add_history_entry(line.clone());

        let mut lines = line + "\n";

        loop {
            parser = parser::Parser::new("REPL", lines.clone());
            match parser.parse_all() {
                Ok(node) => {
                    // compile and execute
                    let mut iseq = vec![];
                    let global_info = match vm.compile(&node, &mut iseq, true, 0) {
                        Ok(ok) => ok,
                        Err(vm::codegen::Error { msg, token_pos, .. }) => {
                            parser.show_error_at(token_pos, msg);
                            break;
                        }
                    };

                    match global_context {
                        Some(ref mut context) => {
                            context.bytecode = iseq;
                            context.exception_table = global_info.exception_table.clone();
                            context.append_from_function_info(
                                &mut vm.factory.memory_allocator,
                                &global_info,
                            )
                        }
                        None => global_context = Some(vm.create_global_context(global_info, iseq)),
                    }

                    let script_info = parser.into_script_info();
                    vm.script_info = vec![(0, script_info)];
                    vm.current_context = global_context.clone().unwrap();
                    match vm.run() {
                        Ok(val) => println!("{}", val.debug_string(true)),
                        Err(e) => {
                            let val = e.to_value(&mut vm.factory);
                            if val.is_error_object() {
                                println!(
                                    "Error: {}",
                                    val.get_property_by_str_key("message").to_string()
                                );
                            } else {
                                println!("Thrown: {}", val.to_string())
                            };
                        }
                    }
                    break;
                }
                Err(parser::Error::UnexpectedEOF(_)) => match rl.readline("... ") {
                    Ok(line) => {
                        rl.add_history_entry(line.clone());
                        lines += line.as_str();
                        lines += "\n";
                        continue;
                    }
                    Err(_) => break,
                },
                Err(e) => {
                    parser.handle_error(&e);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rapidus::test::{assert_file, execute_script, runtime_error, test_code, test_file};

    #[test]
    fn vm_test() {
        execute_script("for(var i = 0; i < 4; i++){ i }".to_string());
        // TODO: Following tests should run. Fix them ASAP.

        // assert_file("letconst".to_string());
        // assert_file("nested_block".to_string());
        // assert_file("nested_block2".to_string());
        // test_code("'true'*3".to_string(), "'truetruetrue'".to_string());
        // test_code("(100).toString(15)".to_string(), "'6a'".to_string());
        // test_file(
        //     "label".to_string(),
        //     "[ 0, 0, 0, 1, 0, 2, 1, 0, 2, 0, 3, 0, 3, 1, 4, 1, 4, 2, 0 ]".to_string(),
        // );

        //test_file(
        //    "qsort".to_string(),
        //    "[ 0, 0, 1, 3, 5, 7, 7, 10, 11, 12, 14, 14, 16, 17, 19 ]".to_string(),
        //);
        // test_file("arguments1".to_string(), "[[1,2,3,4,4],[1,2,[3,4]],[5,6,7,undefined,3],[5,6,[7]],[8,9,undefined,undefined,2],[8,9,undefined],[10,undefined,undefined,undefined,1],[10,undefined,undefined]]".to_string());
        // test_file(
        //     "arguments2".to_string(),
        //     "[10,15,20,25,15,10,'OK',20,25,'OK',10,'NG',20,25,'NG']".to_string(),
        // );
    }

    #[test]
    fn string_test1() {
        test_code("'死して屍拾う者なし'[4]", "'拾'");
    }

    #[test]
    fn string_test2() {
        test_code("'死して屍拾う者なし'.length", "9");
    }
    #[test]
    fn operator_test() {
        test_code("+(5>3)+60%7+(3>=5)+!!5+(-6)", "0");
    }

    #[test]
    fn operator_test2() {
        assert_file("operator");
    }

    #[test]
    fn this_test() {
        test_file("this", "[1,101,124]");
    }

    #[test]
    fn prototype_test() {
        assert_file("prototypes");
    }

    #[test]
    fn accessor_property() {
        assert_file("accessor_property");
    }

    #[test]
    fn trinity() {
        assert_file("trinity");
    }

    #[test]
    fn closure() {
        assert_file("closure");
    }

    #[test]
    fn trycatch() {
        assert_file("trycatch");
    }

    #[test]
    fn r#typeof() {
        assert_file("typeof");
    }

    #[test]
    fn exotic_cmp() {
        assert_file("exotic_cmp")
    }

    #[test]
    fn r#while() {
        assert_file("while")
    }

    #[test]
    fn r#for() {
        assert_file("for")
    }

    #[test]
    fn r#if() {
        assert_file("if")
    }

    #[test]
    fn arrow_function() {
        test_code("let f = (x) => { return x * x }; f(5)", "25");
        test_code("let f = x => { return x * x }; f(6)", "36");
    }

    #[test]
    fn symbol() {
        assert_file("symbol")
    }

    #[test]
    fn array() {
        assert_file("array")
    }

    #[test]
    fn env() {
        assert_file("env")
    }

    #[test]
    fn new_call_member() {
        assert_file("new_call_member")
    }

    #[test]
    fn assert() {
        assert_file("assert")
    }

    #[test]
    fn fibo() {
        assert_file("fibo")
    }

    #[test]
    fn fact() {
        assert_file("fact")
    }

    #[test]
    fn test_module() {
        assert_file("test_module_caller")
    }

    #[test]
    fn runtime_error1() {
        runtime_error("let a = {}; a.b.c");
    }

    #[test]
    fn runtime_error2() {
        runtime_error("let a = {}; a.b.c = 5");
    }

    #[test]
    fn runtime_error3() {
        runtime_error("let a = {}; a(5)");
    }

    #[test]
    fn runtime_error4() {
        runtime_error("let a = {}; a.b(5)");
    }

    #[test]
    fn runtime_error5() {
        runtime_error("let a = {}; a(5)");
    }
}
