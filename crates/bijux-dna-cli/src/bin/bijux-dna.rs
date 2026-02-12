fn main() {
    eprintln!(
        "warning: `bijux-dna` is deprecated; use `bijux dna ...` (compatibility shim will be removed in a future release)"
    );
    let mut args = vec!["bijux".to_string(), "dna".to_string()];
    args.extend(std::env::args().skip(1));
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("failed to resolve current directory: {err}");
            std::process::exit(70);
        }
    };
    if let Err(err) = bijux_dna::commands::run_with_args(&arg_refs, &cwd) {
        eprintln!("{err}");
        std::process::exit(70);
    }
}
