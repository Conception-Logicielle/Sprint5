use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn extract_title_and_abstract(file_path: &Path) -> io::Result<(String, String, String, usize, usize)> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    let total_lines = lines.len();

    let file_name = file_path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace(" ", "_");

    let mut title_lines = Vec::new();
    let mut abstract_lines = Vec::new();
    let mut abstract_started = false;

    // 1. Détecte le titre sur les premières lignes non vides
    for line in lines.iter().take(15) {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        if trimmed.to_lowercase().contains("abstract") {
            break;
        }

        // Exclure les emails ou adresses
        if trimmed.contains("@") || trimmed.contains("http") || trimmed.contains("www") {
            break;
        }

        title_lines.push(trimmed.to_string());

        // On coupe après 3 lignes pour éviter d'absorber tout le début
        if title_lines.len() >= 3 {
            break;
        }
    }

    let title = title_lines.join(" ").replace("  ", " ");

    let mut next_line_is_abstract = false;

    for line in &lines {
        let l = line.trim();

        // Début de l'abstract
        if !abstract_started && l.to_lowercase().starts_with("abstract") {
            abstract_started = true;

            if l.to_lowercase().trim() == "abstract" {
                next_line_is_abstract = true;
                continue;
            }

            // Cas inline : "Abstract — résumé..."
            let cleaned = l
                .trim_start_matches(|c: char| c.is_alphabetic() || c == ':' || c == '—' || c == '-' || c == ' ')
                .trim();
            if !cleaned.is_empty() {
                abstract_lines.push(cleaned.to_string());
            }
            continue;
        }

        // Ligne juste après un "Abstract" seul
        if next_line_is_abstract {
            if l.is_empty() {
                next_line_is_abstract = false;
                continue;
            }
            abstract_lines.push(l.to_string());
            next_line_is_abstract = false;
            continue;
        }

        if abstract_started {
            if l.is_empty()
                || l.starts_with("1 ")
                || l.starts_with("1.")
                || l.to_lowercase().starts_with("introduction")
            {
                break;
            }
            abstract_lines.push(l.to_string());
        }
    }


    // Mise en forme de l'abstract
    let mut abstract_text = abstract_lines
        .join(" ")
        .replace("- ", "")
        .replace("\n", " ")
        .replace("  ", " ");

    abstract_text = abstract_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let abstract_line_count = abstract_lines.len();

    Ok((
        file_name,
        title.trim().to_string(),
        abstract_text.trim().to_string(),
        total_lines,
        abstract_line_count,
    ))
}


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_folder> <output_folder>", args[0]);
        std::process::exit(1);
    }

    let input_folder = Path::new(&args[1]);
    let output_folder = Path::new(&args[2]);

    fs::create_dir_all(output_folder)?;

    let output_file_path = output_folder.join("resumes.txt");
    let mut output_file = File::create(output_file_path)?;

    let mut total_duration = Duration::new(0, 0);

    for entry in fs::read_dir(input_folder)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let start = Instant::now();

            if let Ok((filename, title, abstract_text, total_lines, abstract_line_count)) =
                extract_title_and_abstract(&path)
            {
                let duration = start.elapsed();
                total_duration += duration;

                writeln!(
                    output_file,
                    "==============================\n\
🗂 Fichier        : {}\n\
📝 Titre          : {}\n\
📄 Résumé         : {}\n\
📊 Lignes totales : {}\n\
✂️ Lignes résumé  : {}\n\
🔠 Longueur texte : {} caractères\n\
⏱ Temps analyse  : {} ms\n",
                    filename,
                    title,
                    abstract_text,
                    total_lines,
                    abstract_line_count,
                    abstract_text.len(),
                    duration.as_millis()
                )?;
            }
        }
    }

    println!(
        "✅ Résumé généré avec succès ! Temps total de traitement : {} ms",
        total_duration.as_millis()
    );
    Ok(())
}
