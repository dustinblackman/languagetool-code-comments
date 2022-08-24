#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

mod cache;
mod commands;
mod lt;
mod parse;

use anyhow::Result;
use colored::*;
use fstrings::*;
use lazy_static::lazy_static;
use std::io;

lazy_static! {
    static ref SUPPORTED_LANGS_HELP: String = {
        let supported_langs = env!("LTCC_LANGS")
            .split(',')
            .collect::<Vec<&str>>()
            .join("\n  - ")
            .green();

        let header = "SUPPORTED LANGUAGES:".yellow();
        return f!("{header}\n  - {supported_langs}");
    };
}

fn build_cli() -> clap::Command<'static> {
    return clap::Command::new("languagetool-code-comments")
        .about("Integrates the LanguageTool API to parse, spell check, and correct the grammar of your code comments!")
        .after_help(SUPPORTED_LANGS_HELP.as_str())
        .version(env!("VERGEN_GIT_SEMVER"))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            clap::Command::new("check")
                .about("Parses source code comments from the provided file and passes them to LanguageTool, returning grammar and spelling mistakes if any.")
                .arg(
                    clap::Arg::new("url")
                        .long("url")
                        .short('u')
                        .help("LanguageTool API url.")
                        .env("LTCC_URL")
                        .default_value("https://api.languagetool.org")
                        .takes_value(true)
                        .multiple_values(false),
                )
                .arg(
                    clap::Arg::new("file")
                        .long("file")
                        .short('f')
                        .help("Path to source code file.")
                        .value_hint(clap::ValueHint::FilePath)
                        .takes_value(true)
                        .multiple_values(false),
                )
                .arg(
                    clap::Arg::new("concurrency")
                        .long("concurrency")
                        .short('c')
                        .default_value("10")
                        .help("Maximum amount of requests to make to LanguageTools in parallel.")
                        .takes_value(true)
                        .multiple_values(false),
                )
                .arg(
                    clap::Arg::new("language")
                        .long("language")
                        .short('l')
                        .default_value("auto")
                        .help("Written language of code comment blocks. Setting this to a language code (en-US, fr-FR, es-MX) will speed up requests to LanguageTool.")
                        .takes_value(true)
                        .multiple_values(false),
                ),

        )
        .subcommand(
            clap::Command::new("completion")
                .about("Generates shell completions")
                .arg(
                    clap::Arg::new("shell")
                        .short('s')
                        .long("shell")
                        .help("Which shell to generate completions for.")
                        .possible_values(clap_complete::Shell::possible_values())
                        .required(true),
                ),
        )
        .subcommand(
            clap::Command::new("cache")
                .about("Functionality around the LanguageTools result cache.")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    clap::Command::new("path").about("Outputs the cache directories path")
                )
                .subcommand(
                    clap::Command::new("delete").about("Deletes the entire cache directory")
                )
        );
}

fn print_completions<G: clap_complete::Generator>(gen: G, app: &mut clap::Command) {
    clap_complete::generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

async fn parse_cli() -> Result<()> {
    let matches = build_cli().get_matches();
    match matches.subcommand() {
        Some(("check", run_matches)) => {
            let filepath = run_matches.get_one::<String>("file").unwrap().to_string();
            let languagetool_api_url = run_matches.get_one::<String>("url").unwrap().to_string();
            let language = run_matches
                .get_one::<String>("language")
                .unwrap()
                .to_string();
            let concurrency = run_matches
                .get_one::<String>("concurrency")
                .unwrap()
                .parse::<usize>()?;

            let res =
                commands::check(filepath, languagetool_api_url, concurrency, language).await?;
            println!("{}", serde_json::to_string(&res)?);
        }
        Some(("completion", run_matches)) => {
            if let Ok(generator) = run_matches.value_of_t::<clap_complete::Shell>("shell") {
                eprintln!("Generating completion file for {}...", generator);
                let mut app = build_cli();
                print_completions(generator, &mut app);
            }
        }
        Some(("cache", args)) => match args.subcommand() {
            Some(("delete", _)) => {
                cache::delete_cache().await?;
            }
            Some(("path", _)) => {
                println!("{}", cache::get_dir_path().await?.to_str().unwrap());
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    return Ok(());
}

#[tokio::main]
async fn main() {
    parse_cli().await.unwrap();
}
