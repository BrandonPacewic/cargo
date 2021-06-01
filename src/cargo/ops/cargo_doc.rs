use crate::core::{Shell, Workspace};
use crate::ops;
use crate::util::config::PathAndArgs;
use crate::util::CargoResult;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Strongly typed options for the `cargo doc` command.
#[derive(Debug)]
pub struct DocOptions {
    /// Whether to attempt to open the browser after compiling the docs
    pub open_result: bool,
    /// Options to pass through to the compiler
    pub compile_opts: ops::CompileOptions,
}

#[derive(Deserialize)]
struct CargoDocConfig {
    /// Browser to use to open docs. If this is unset, the value of the environment variable
    /// `BROWSER` will be used.
    browser: Option<PathAndArgs>,
}

/// Main method for `cargo doc`.
pub fn doc(ws: &Workspace<'_>, options: &DocOptions) -> CargoResult<()> {
    let compilation = ops::compile(ws, &options.compile_opts)?;

    if options.open_result {
        let name = &compilation.root_crate_names[0];
        let kind = options.compile_opts.build_config.single_requested_kind()?;
        let path = compilation.root_output[&kind]
            .with_file_name("doc")
            .join(&name)
            .join("index.html");
        if path.exists() {
            let config_browser = {
                let cfg = ws.config().get::<CargoDocConfig>("doc")?;

                cfg.browser
                    .map(|path_args| (path_args.path.resolve_program(ws.config()), path_args.args))
            };

            let mut shell = ws.config().shell();
            shell.status("Opening", path.display())?;
            open_docs(&path, &mut shell, config_browser)?;
        }
    }

    Ok(())
}

fn open_docs(
    path: &Path,
    shell: &mut Shell,
    config_browser: Option<(PathBuf, Vec<String>)>,
) -> CargoResult<()> {
    let browser =
        config_browser.or_else(|| Some((PathBuf::from(std::env::var_os("BROWSER")?), Vec::new())));

    match browser {
        Some((browser, initial_args)) => {
            if let Err(e) = Command::new(&browser).args(initial_args).arg(path).status() {
                shell.warn(format!(
                    "Couldn't open docs with {}: {}",
                    browser.to_string_lossy(),
                    e
                ))?;
            }
        }
        None => {
            if let Err(e) = opener::open(&path) {
                let e = e.into();
                crate::display_warning_with_error("couldn't open docs", &e, shell);
            }
        }
    };

    Ok(())
}
