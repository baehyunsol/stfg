use ragit_cli::{
    ArgCount,
    ArgParser,
    ArgType,
    Span,
    get_closest_string,
};
use stfg::{Error, from_git, to_git};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if let Err(e) = run(&args) {
        match e {
            Error::CliError { message, span } => {
                eprintln!(
                    "cli error: {message}{}",
                    if let Some(span) = span {
                        format!("\n\n{}", ragit_cli::underline_span(&span))
                    } else {
                        String::new()
                    },
                );
            },
            _ => {
                eprintln!("{e:?}");
            },
        }

        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), Error> {
    match args.get(1).map(|arg| arg.as_str()) {
        Some("to-git" | "from-sql") => {
            let parsed_args = ArgParser::new()
                .arg_flag("--output", ArgType::String)
                .short_flag(&["--output"])
                .args(ArgType::String, ArgCount::Exact(1))
                .parse(&args, 2)?;

            let input = parsed_args.get_args_exact(1)?[0].clone();
            let output = parsed_args.arg_flags.get("--output").unwrap().to_string();
            to_git(&input, &output)?;
        },
        Some("from-git" | "to-sql") => {
            let parsed_args = ArgParser::new()
                .arg_flag("--output", ArgType::String)
                .short_flag(&["--output"])
                .args(ArgType::String, ArgCount::Exact(1))
                .parse(&args, 2)?;

            let input = parsed_args.get_args_exact(1)?[0].clone();
            let output = parsed_args.arg_flags.get("--output").unwrap().to_string();
            from_git(&output, &input)?;
        },
        Some(invalid_command) => {
            let similar_command = get_closest_string(
                &[
                    "to-git",
                    "to-sql",
                    "from-git",
                    "from-sql",
                ].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                invalid_command,
            );

            return Err(Error::CliError {
                message: format!(
                    "`{invalid_command}` is an invalid command. {}",
                    if let Some(similar_command) = similar_command {
                        format!("There is a similar command: `{similar_command}`.")
                    } else {
                        String::new()
                    },
                ),
                span: Span::NthArg(0).render(&args, 1),
            });
        },
        _ => todo!(),
    }

    Ok(())
}
