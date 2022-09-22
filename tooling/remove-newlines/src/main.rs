fn main() -> Result<(), Box<dyn std::error::Error>> {
    for input_path in std::env::args().skip(1) {
        let input_path = std::path::PathBuf::from(input_path);

        let mut output_path = 
            input_path
                .file_name()
                .ok_or("no filename")?
                .to_os_string();
        output_path.push(".");
        output_path.push("o");
        output_path.push("u");
        output_path.push("t");
        let output_path = std::path::PathBuf::from(output_path);

        if output_path.exists() {
            return Err("output path exists".into());
        }
        eprintln!("reading from: {}", input_path.display());
        let input = std::fs::read_to_string(input_path)?;

        let mut output = String::with_capacity(input.len());

        for c in input.chars() {
            if c == '\n' {
                continue
            }
            output.push(c);
        }

        // Reduce chance of TOCTOU error.
        if output_path.exists() {
            return Err("output path exists".into());
        }
        eprintln!("writing to: {}", output_path.display());
        std::fs::write(output_path, output)?;
    }

    Ok(())
}
