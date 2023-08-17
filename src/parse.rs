use std::process::Command;

pub fn emacs_parse_org_document<C>(file_contents: C) -> Result<String, Box<dyn std::error::Error>>
where
    C: AsRef<str>,
{
    let escaped_file_contents = escape_elisp_string(file_contents);
    let elisp_script = format!(
        r#"(progn
     (erase-buffer)
     (insert "{escaped_file_contents}")
     (org-mode)
     (message "%s" (pp-to-string (org-element-parse-buffer)))
)"#,
        escaped_file_contents = escaped_file_contents
    );
    let mut cmd = Command::new("emacs");
    let proc = cmd
        .arg("-q")
        .arg("--no-site-file")
        .arg("--no-splash")
        .arg("--batch")
        .arg("--eval")
        .arg(elisp_script);
    let out = proc.output()?;
    out.status.exit_ok()?;
    let org_sexp = out.stderr;
    Ok(String::from_utf8(org_sexp)?)
}

fn escape_elisp_string<C>(file_contents: C) -> String
where
    C: AsRef<str>,
{
    let source = file_contents.as_ref();
    let source_len = source.len();
    // We allocate a string 10% larger than the source to account for escape characters. Without this, we would have more allocations during processing.
    let mut output = String::with_capacity(source_len + (source_len / 10));
    for c in source.chars() {
        match c {
            '"' | '\\' => {
                output.push('\\');
                output.push(c);
            }
            _ => {
                output.push(c);
            }
        }
    }
    output
}
