mod convert;
mod input;
pub mod model;
pub use convert::convert;

use anyhow::{Context, Result, bail};
use cli_command_kit::{
    HelpOptions, Mode,
    clap::{Arg, ArgAction, ArgMatches, Command, value_parser},
};
use std::{
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn paths(command: Command) -> Command {
    command
        .arg(
            Arg::new("input")
                .value_name("INPUT")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .help("Dashlane CSV directory or ZIP archive"),
        )
        .arg(
            Arg::new("output")
                .value_name("OUTPUT")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .help("Bitwarden JSON output file"),
        )
}

pub fn command() -> Command {
    paths(
        Command::new("dashlane-to-bitwarden")
            .about("Convert local Dashlane exports into Bitwarden JSON"),
    )
    .subcommand_negates_reqs(true)
    .args_conflicts_with_subcommands(true)
    .arg(
        Arg::new("version")
            .long("version")
            .short('v')
            .action(ArgAction::SetTrue)
            .exclusive(true)
            .help("Print the raw version"),
    )
    .subcommand(
        Command::new("export")
            .about("Manage exported vault files")
            .subcommand_required(true)
            .subcommand(paths(
                Command::new("convert").about("Convert a Dashlane export to Bitwarden JSON"),
            )),
    )
}

pub fn help_options() -> HelpOptions {
    HelpOptions {
        skill: Some(
            cli_command_kit::SkillSource::new("alexgorbatchev/dashlane-to-bitwarden", "main")
                .expect("the fixed skill source names a GitHub repository"),
        ),
        ..HelpOptions::default()
    }
}

/// Parse every source category before atomically replacing the destination.
pub fn convert_to_file(input: &Path, output: &Path) -> Result<model::Export> {
    let export = convert(input)?;
    if let Ok(target) = output.canonicalize() {
        let is_source = if input.is_dir() {
            input::FILES.iter().any(|name| {
                input
                    .join(name)
                    .canonicalize()
                    .is_ok_and(|path| path == target)
            })
        } else {
            input.canonicalize().is_ok_and(|path| path == target)
        };
        if is_source {
            bail!("output would replace a source export file");
        }
    }
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let shown = cli_command_kit::display_path(output);
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "create output directory {}",
            cli_command_kit::display_path(parent)
        )
    })?;
    let mut file =
        tempfile::NamedTempFile::new_in(parent).context("create private output staging file")?;
    serde_json::to_writer_pretty(&mut file, &export).context("write Bitwarden export")?;
    file.write_all(b"\n").context("write export newline")?;
    file.as_file()
        .sync_all()
        .context("sync output staging file")?;
    file.persist(output)
        .map_err(|error| error.error)
        .with_context(|| format!("replace output {shown}"))?;
    Ok(export)
}

fn perform(
    matches: &ArgMatches,
    options: &HelpOptions,
    output: &mut impl Write,
    mode: Mode,
) -> Result<()> {
    if cli_command_kit::write_skill(&command(), matches, options, output)? {
        return Ok(());
    }
    if matches.get_flag("version") {
        writeln!(output, "{}", env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }
    let matches = match matches.subcommand() {
        Some(("export", parent)) => parent
            .subcommand_matches("convert")
            .expect("required convert command"),
        _ => matches,
    };
    let input = matches.get_one::<PathBuf>("input").expect("required input");
    let path = matches
        .get_one::<PathBuf>("output")
        .expect("required output");
    let export = convert_to_file(input, path)?;
    let mut counts = [0_usize; 4];
    for item in &export.items {
        counts[item.kind_index()] += 1;
    }
    let shown = cli_command_kit::display_path(path);
    if mode == Mode::Agent {
        serde_json::to_writer(
            &mut *output,
            &serde_json::json!({"output": shown, "items": export.items.len(), "logins": counts[0], "cards": counts[2], "identities": counts[3], "secure_notes": counts[1]}),
        )?;
        writeln!(output)?;
    } else {
        writeln!(
            output,
            "Created: {shown}\nLogins: {}\nCards: {}\nIdentities: {}\nSecure Notes: {}",
            counts[0], counts[2], counts[3], counts[1]
        )?;
    }
    Ok(())
}

pub fn entry(
    args: impl IntoIterator<Item = OsString>,
    output: &mut impl Write,
    diagnostics: &mut impl Write,
    mode: Mode,
) -> u8 {
    let options = help_options();
    let root =
        cli_command_kit::setup(command(), mode, &options).expect("valid static command tree");
    let result = match root.try_get_matches_from(args) {
        Ok(matches) => perform(&matches, &options, output, mode),
        Err(error) if !error.use_stderr() => output
            .write_all(error.to_string().as_bytes())
            .map_err(Into::into),
        Err(error) => {
            let _ = write!(diagnostics, "{error}");
            return 1;
        }
    };
    match result {
        Ok(()) => 0,
        Err(error) => {
            let prefix = if mode == Mode::Agent {
                "ERR:"
            } else {
                "[ERROR]"
            };
            let _ = writeln!(diagnostics, "{prefix} {error:#}");
            1
        }
    }
}
