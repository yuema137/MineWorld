//! `cli.md`: every visible subcommand and argument of `mineworld`, walked from the parser's own model
//! (`docs/DECISIONS.md` `DEP-11`, `DEP-49`: `clap-markdown` declined).

use std::fmt::Write as _;

use clap::{Arg, Command};

/// The page's body: one section per visible command, depth first, in the parser's order.
pub(super) fn page(out: &mut String, root: &Command) {
    for command in visible(root) {
        section(
            out,
            command,
            &format!("{} {}", root.get_name(), command.get_name()),
        );
    }
}

/// A command's subcommands a person types: not hidden, and not the parser's own `help`.
fn visible(command: &Command) -> impl Iterator<Item = &Command> {
    command
        .get_subcommands()
        .filter(|c| !c.is_hide_set() && c.get_name() != "help")
}

/// One command, then its own subcommands.
fn section(out: &mut String, command: &Command, path: &str) {
    let _ = writeln!(out, "## `{path}`\n");
    if let Some(about) = command.get_long_about().or_else(|| command.get_about()) {
        let _ = writeln!(out, "{}\n", about.to_string().trim());
    }
    let arguments: Vec<&Arg> = command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set() && !["help", "version"].contains(&arg.get_id().as_str()))
        .collect();
    if !arguments.is_empty() {
        out.push_str("| Argument | Required | Default | Description |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for arg in arguments {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                usage(arg),
                if arg.is_required_set() { "yes" } else { "no" },
                default(arg),
                help(arg),
            );
        }
        out.push('\n');
    }
    for child in visible(command) {
        section(out, child, &format!("{path} {}", child.get_name()));
    }
}

/// How an argument is written: `--save DIR`, `<world>`, `--town`.
fn usage(arg: &Arg) -> String {
    let value = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map_or_else(
            || arg.get_id().as_str().to_ascii_uppercase(),
            ToString::to_string,
        );
    let takes_value = arg.get_num_args().is_none_or(|range| range.takes_values())
        && !matches!(
            arg.get_action(),
            clap::ArgAction::SetTrue | clap::ArgAction::SetFalse | clap::ArgAction::Count
        );
    let repeat = if matches!(arg.get_action(), clap::ArgAction::Append) {
        " …"
    } else {
        ""
    };
    match arg.get_long() {
        Some(long) if takes_value => format!("`--{long} {value}`{repeat}"),
        Some(long) => format!("`--{long}`"),
        None => format!("`<{}>`{repeat}", arg.get_id().as_str()),
    }
}

/// An argument's default, or a dash.
fn default(arg: &Arg) -> String {
    let defaults: Vec<String> = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    if defaults.is_empty() {
        "—".to_owned()
    } else {
        format!("`{}`", defaults.join(" "))
    }
}

/// An argument's help text, with the environment variable it also reads.
fn help(arg: &Arg) -> String {
    let mut text = arg
        .get_long_help()
        .or_else(|| arg.get_help())
        .map(|help| help.to_string())
        .unwrap_or_default()
        .replace('\n', " ")
        .replace('|', "\\|");
    let text_ends = text.trim_end().ends_with(['.', ')']);
    if !text.trim().is_empty() && !text_ends {
        text = format!("{}.", text.trim_end());
    }
    if let Some(variable) = arg.get_env() {
        let _ = write!(text, " Also read from `{}`.", variable.to_string_lossy());
    }
    text.trim().to_owned()
}
