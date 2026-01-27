use crate::error::AnalyzerError;
use falsec_types::Config;
use falsec_types::source::{Command, Lambda, LambdaCommand, Program, Span};
use falsec_util::string_id;
use std::borrow::Cow;
use std::collections::HashMap;

pub mod error;

pub struct Analyzer<'source> {
    program: Vec<(Command<'source>, Span<'source>)>,
    #[allow(dead_code)] // TODO: will be used later
    config: Config,
}

impl<'source> Analyzer<'source> {
    pub fn new(program: Vec<(Command<'source>, Span<'source>)>, config: Config) -> Self {
        Self { program, config }
    }

    pub fn analyze(self) -> Result<Program<'source>, AnalyzerError> {
        let lambdas = Self::extract_lambdas(self.program, HashMap::new(), 0)?;
        let strings = lambdas
            .values()
            .try_fold(Default::default(), Self::extract_strings)?;
        Ok(Program {
            main_id: 0,
            lambdas,
            strings,
        })
        // todo: code optimization, dead code elimination, etc.
    }

    fn extract_lambdas(
        program: Lambda<'source>,
        mut lambdas: HashMap<u64, Lambda<'source>>,
        id: u64,
    ) -> Result<HashMap<u64, Lambda<'source>>, AnalyzerError> {
        let mut lambda = Lambda::new();
        let mut current_id = id + 1;
        for (command, span) in program {
            lambda.push((
                match command {
                    Command::Lambda(LambdaCommand::LambdaDefinition(inner)) => {
                        let c = lambdas.len();
                        lambdas = Self::extract_lambdas(inner, lambdas, current_id)?;
                        let com = Command::Lambda(LambdaCommand::LambdaReference(current_id));
                        current_id += (lambdas.len() - c) as u64;
                        com
                    }
                    Command::Lambda(LambdaCommand::LambdaReference(..)) => {
                        return Err(AnalyzerError::invalid_input(
                            "Lambda references are not allowed in lambda definitions",
                        ));
                    }
                    command => command,
                },
                span,
            ));
        }
        lambdas.insert(id, lambda);
        Ok(lambdas)
    }

    fn extract_strings(
        mut strings: HashMap<u64, Cow<'source, str>>,
        lambda: &Lambda<'source>,
    ) -> Result<HashMap<u64, Cow<'source, str>>, AnalyzerError> {
        for (command, _) in lambda {
            if let Command::StringLiteral(s) = command {
                strings.entry(string_id(s)).or_insert_with(|| s.clone());
            }
        }
        Ok(strings)
    }
}

#[cfg(test)]
mod tests {
    use crate::Analyzer;
    use falsec_types::source::{Command, LambdaCommand, Pos, Span};
    use std::borrow::Cow;

    fn insert_dummy_spans(commands: Vec<Command>) -> Vec<(Command, Span)> {
        commands
            .into_iter()
            .map(|command| (command, Span::new(Pos::at_start(), Pos::at_start(), "")))
            .collect()
    }

    #[test]
    fn only_main() {
        let program = insert_dummy_spans(vec![
            Command::IntLiteral(123),
            Command::StringLiteral(Cow::Borrowed("Hello, World!")),
            Command::Dup,
            Command::WriteInt,
        ]);
        let analyzed = Analyzer::new(program.clone(), Default::default())
            .analyze()
            .unwrap();
        assert_eq!(analyzed.main_id, 0);
        assert_eq!(analyzed.lambdas.len(), 1);
        assert_eq!(analyzed.lambdas.get(&0).unwrap(), &program);
    }

    #[test]
    fn one_lambda() {
        let program = insert_dummy_spans(vec![
            Command::Comment(Cow::Borrowed("This is a lambda")),
            Command::Lambda(LambdaCommand::LambdaDefinition(insert_dummy_spans(vec![
                Command::IntLiteral(123),
                Command::StringLiteral(Cow::Borrowed("Hello, World!")),
                Command::Dup,
                Command::WriteInt,
            ]))),
        ]);
        let analyzed = Analyzer::new(program.clone(), Default::default())
            .analyze()
            .unwrap();
        assert_eq!(analyzed.main_id, 0);
        assert_eq!(analyzed.lambdas.len(), 2);
        assert_eq!(
            analyzed.lambdas.get(&0).unwrap(),
            &insert_dummy_spans(vec![
                Command::Comment(Cow::Borrowed("This is a lambda")),
                Command::Lambda(LambdaCommand::LambdaReference(1)),
            ])
        );
        assert_eq!(
            analyzed.lambdas.get(&1).unwrap(),
            &insert_dummy_spans(vec![
                Command::IntLiteral(123),
                Command::StringLiteral(Cow::Borrowed("Hello, World!")),
                Command::Dup,
                Command::WriteInt,
            ])
        );
    }

    #[test]
    fn test_nested() {
        let program = insert_dummy_spans(vec![
            Command::Lambda(LambdaCommand::LambdaDefinition(insert_dummy_spans(vec![
                Command::IntLiteral(123),
                Command::Lambda(LambdaCommand::LambdaDefinition(insert_dummy_spans(vec![
                    Command::CharLiteral('x'),
                    Command::WriteChar,
                ]))),
                Command::Drop,
            ]))),
            Command::Lambda(LambdaCommand::LambdaDefinition(insert_dummy_spans(vec![
                Command::IntLiteral(5),
                Command::WriteInt,
            ]))),
        ]);
        let analyzed = Analyzer::new(program.clone(), Default::default())
            .analyze()
            .unwrap();
        assert_eq!(analyzed.main_id, 0);
        assert_eq!(analyzed.lambdas.len(), 4);
        assert_eq!(
            &analyzed.lambdas[&0],
            &insert_dummy_spans(vec![
                Command::Lambda(LambdaCommand::LambdaReference(1)),
                Command::Lambda(LambdaCommand::LambdaReference(3)),
            ])
        );
        assert_eq!(
            &analyzed.lambdas[&1],
            &insert_dummy_spans(vec![
                Command::IntLiteral(123),
                Command::Lambda(LambdaCommand::LambdaReference(2)),
                Command::Drop,
            ])
        );
        assert_eq!(
            &analyzed.lambdas[&2],
            &insert_dummy_spans(vec![Command::CharLiteral('x'), Command::WriteChar])
        );
        assert_eq!(
            &analyzed.lambdas[&3],
            &insert_dummy_spans(vec![Command::IntLiteral(5), Command::WriteInt])
        );
    }
}
