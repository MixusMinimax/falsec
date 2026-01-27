use falsec_analyzer::Analyzer;
use falsec_interpreter::Interpreter;
use falsec_parser::Parser;
use falsec_types::Config;
use falsec_types::source::Program;

fn parse_program<'source>(program: &'source str, config: &Config) -> Program<'source> {
    let parser = Parser::new(program, config.clone());
    let commands: Result<Vec<_>, _> = parser.collect();
    Analyzer::new(commands.unwrap(), config.clone())
        .analyze()
        .unwrap()
}

#[test]
fn test_a() {
    let config = Config::default();
    let code = include_str!("samples/a.f");
    let program = parse_program(code, &config);
    let mut out = Vec::new();
    Interpreter::new(&b"L123"[..], &mut out, program, config)
        .run()
        .unwrap();
    assert_eq!(out, b"123");
}
