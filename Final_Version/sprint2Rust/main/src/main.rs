use std::alloc::System;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn detect_column_cutoff(lines: &[String]) -> usize {
    use std::collections::HashMap;
    let mut histogram = HashMap::new();
    let mut total_lines = 0;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        total_lines += 1;

        let chars: Vec<char> = line.chars().collect();
        let mut space_count = 0;

        for (i, &c) in chars.iter().enumerate().skip(40) {
            if c == ' ' {
                space_count += 1;
            } else {
                if space_count >= 4 {
                    *histogram.entry(i).or_insert(0) += 1;
                }
                space_count = 0;
            }
        }
    }

    // Si aucune coupure claire détectée → monocolonne
    if histogram.is_empty() {
        println!("Pas de coupure détectée, document considéré comme monocolonne.");
        return usize::MAX;
    }

    let (cutoff, count) = histogram
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .unwrap();

    let ratio = count as f32 / total_lines as f32;

    // Si la coupure ne concerne qu'une minorité de lignes → probablement bruit
    if ratio < 0.15 {
        println!(
            "Coupure détectée à {} caractères mais trop rare ({} lignes sur {}), ignorée.",
            cutoff, count, total_lines
        );
        return usize::MAX;
    }

    println!(
        "✅ Coupure détectée à {} caractères ({} occurrences sur {}).",
        cutoff, count, total_lines
    );
    cutoff
}






fn keep_left_column(lines: &[String], dynamic_width: usize) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let trimmed = line.trim_end();
            trimmed
                .chars()
                .take(dynamic_width)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}


fn extract_title_and_abstract(file_path: &Path) -> io::Result<(String, String, String, usize, usize)> {
    println!("Traitement du fichier : {:?}", file_path);
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let raw_lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    let cutoff = detect_column_cutoff(&raw_lines);
    let lines = keep_left_column(&raw_lines, cutoff);
    let total_lines = lines.len();

    let file_name = file_path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace(" ", "_");

    let mut title_lines = Vec::new();
    let mut abstract_lines = Vec::new();
    let mut abstract_started = false;

    for line in lines.iter().take(15) {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        if trimmed.to_lowercase().contains("abstract") {
            break;
        }

        if trimmed.contains("@") || trimmed.contains("http") || trimmed.contains("www") {
            break;
        }

        title_lines.push(trimmed.to_string());

        if title_lines.len() >= 3 {
            break;
        }
    }

    let title = title_lines.join(" ").replace("  ", " ");

    let mut next_line_is_abstract = false;

    for line in &lines {
        let l = line.trim();

        if !abstract_started && l.to_lowercase().starts_with("abstract") {
            abstract_started = true;

            if l.to_lowercase().trim() == "abstract" {
                next_line_is_abstract = true;
                continue;
            }

            let cleaned = l
                .trim_start_matches(|c: char| c.is_alphabetic() || c == ':' || c == '—' || c == '-' || c == ' ')
                .trim();
            if !cleaned.is_empty() {
                abstract_lines.push(cleaned.to_string());
            }
            continue;
        }

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
            if  l.starts_with("1 ")
                || l.starts_with("1.")
                || l.to_lowercase().starts_with("introduction")
                || l.starts_with("I.")
                || l.starts_with("I")
            {
                break;
            }
            abstract_lines.push(l.to_string());
        }
    }


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
    println!("------------------------");

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
Fichier        : {}\n\
Titre          : {}\n\
Résumé         : {}\n\
Lignes totales : {}\n\
Lignes résumé  : {}\n\
Longueur texte : {} caractères\n\
Temps analyse  : {} ms\n",
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
