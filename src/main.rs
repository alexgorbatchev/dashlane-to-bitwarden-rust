use std::io;

fn main() {
    std::process::exit(i32::from(dashlane_to_bitwarden::entry(
        std::env::args_os(),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
        cli_command_kit::Mode::from_env(),
    )));
}
