use crate::command::{CommandRunner, CommandSpec, process_runner::ProcessCommandRunner};

#[test]
fn process_runner_executes_command_and_captures_output() {
    let runner = ProcessCommandRunner;

    let command = CommandSpec {
        program: "printf".into(),
        args: vec!["hello".into()],
    };

    let output = runner.run(&command, &std::env::temp_dir()).unwrap();

    assert_eq!(output.status, Some(0));
    assert_eq!(output.stdout, "hello");
    assert!(output.stderr.is_empty());
}
