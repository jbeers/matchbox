use matchbox_compiler::{compile_with_treeshaking, parser};
use matchbox_vm::vm::VM;

fn compile_source(source: &str) -> matchbox_vm::Chunk {
    let ast = parser::parse(source, Some("host_lifecycle_test.bx")).unwrap();
    let mut chunk = compile_with_treeshaking(
        "host_lifecycle_test.bx",
        &ast,
        source,
        Vec::new(),
        false,
        false,
        &[],
        &[],
    )
    .unwrap();
    chunk.reconstruct_functions();
    chunk
}

#[test]
fn host_method_call_preserves_fibers_spawned_by_run_async() {
    let source = r#"
        class Application {
            function onApplicationStart() {
                runAsync( function() {
                    sleep( 1000 );
                    return "done";
                } );
            }
        }
    "#;

    let mut vm = VM::new();
    let chunk = compile_source(source);
    vm.interpret_chunk_borrowed(&chunk).unwrap();
    let application = vm.construct_global_class("Application", Vec::new()).unwrap();

    vm.call_method_value(application, "onApplicationStart", Vec::new())
        .unwrap();

    vm.pump_until_blocked().unwrap();

    assert_eq!(
        vm.fibers.len(),
        1,
        "runAsync fiber should remain queued for the host scheduler after the completed lifecycle fiber is cleaned up"
    );
}

#[test]
fn pump_until_blocked_does_not_resume_sleeping_fiber_early() {
    let source = r#"
        class Application {
            function onApplicationStart() {
                runAsync( function() {
                    sleep( 1000 );
                    variables.done = true;
                } );
            }
        }
    "#;

    let mut vm = VM::new();
    let chunk = compile_source(source);
    vm.interpret_chunk_borrowed(&chunk).unwrap();
    let application = vm.construct_global_class("Application", Vec::new()).unwrap();

    vm.call_method_value(application, "onApplicationStart", Vec::new())
        .unwrap();

    vm.pump_until_blocked().unwrap();
    vm.pump_until_blocked().unwrap();

    assert_eq!(
        vm.get_global("done").map(|value| value.as_bool()),
        None,
        "sleeping runAsync fiber should not resume until its wait time has elapsed"
    );
    assert_eq!(vm.fibers.len(), 1);
}

#[test]
fn application_scope_is_mutable_from_application_start() {
    let source = r#"
        class Application {
            function onApplicationStart() {
                application.automaticJiggleEnabled = true;
            }
        }
    "#;

    let mut vm = VM::new();
    let chunk = compile_source(source);
    vm.interpret_chunk_borrowed(&chunk).unwrap();
    let application = vm.construct_global_class("Application", Vec::new()).unwrap();
    vm.insert_empty_struct_global("application");

    vm.call_method_value(application, "onApplicationStart", Vec::new())
        .unwrap();

    let enabled = vm
        .get_global_struct_member("application", "automaticJiggleEnabled")
        .unwrap();

    assert_eq!(enabled.as_bool(), true);
}

#[test]
fn current_task_chunk_completes_with_background_fiber_alive() {
    let application_source = r#"
        class Application {
            function onApplicationStart() {
                runAsync( function() {
                    while ( true ) {
                        sleep( 5000 );
                    }
                } );
            }
        }
    "#;
    let route_source = r#"
        routeCompleted = true;
        "ok";
    "#;

    let mut vm = VM::new();
    let application_chunk = compile_source(application_source);
    vm.interpret_chunk_borrowed(&application_chunk).unwrap();
    let application = vm.construct_global_class("Application", Vec::new()).unwrap();
    vm.insert_empty_struct_global("application");
    vm.call_method_value(application, "onApplicationStart", Vec::new())
        .unwrap();
    vm.pump_until_blocked().unwrap();

    assert_eq!(vm.fibers.len(), 1);

    let route_chunk = compile_source(route_source);
    vm.interpret_chunk_borrowed_current_task(&route_chunk).unwrap();

    assert_eq!(
        vm.get_global("routeCompleted").map(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        vm.fibers.len(),
        1,
        "background fiber should remain alive after route execution"
    );
}
