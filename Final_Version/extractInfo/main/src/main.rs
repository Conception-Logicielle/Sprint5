use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use rayon::prelude::*;
use regex::Regex;

#[derive(Debug)]
struct ArticleData {
    filename: String,
    title: String,
    authors: String,
    abstract_text: String,
    introduction: String,
    body: String,
    conclusion: String,
    discussion: String,
    bibliography: String,
}

/// Toutes les regex utilisées sont compilées une seule fois ici.
struct RegexSet {
    metadata: Regex,
    likely_author: Regex,
    name_pair: Regex,
    multiple_names: Regex,
}

impl RegexSet {
    fn new() -> Self {
        Self {
            metadata: Regex::new(r"(?i)(conference|volume|doi|issn|copyright|journal|published|©)").unwrap(),
            likely_author: Regex::new(r"(?ix)^((?:[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)(?:[\d†*]*)\s*(?:,\s*|\s+and\s+|\s+et\s+)?)+$").unwrap(),
            name_pair: Regex::new(r"^[A-Z][a-z]+(\s+[A-Z][a-z]{2,})([\d†*∗°]*)$").unwrap(),
            multiple_names: Regex::new(r"(?i)([A-Z][a-z]+(?:\s+[A-Z]\.?)?\s+[A-Z][a-z]+(?:\d*)\s*(,|and|et)\s*){1,}").unwrap(),
        }
    }
}

/// Détecte le titre de l'article dans les premières lignes du fichier.
fn extract_title(lines: &[String], regex: &RegexSet) -> (String, usize) {
    let mut title_lines = Vec::new();
    let mut started = false;
    let mut last_index = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if started {
                break;
            }
            continue;
        }
        if regex.metadata.is_match(trimmed) {
            continue;
        }
        let words: Vec<_> = trimmed.split_whitespace().collect();
        let uppercase_count = words.iter().filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)).count();

        if !started {
            if uppercase_count >= words.len() / 2 && words.len() >= 2 {
                started = true;
                title_lines.push(trimmed.to_string());
                last_index = i + 1;
            }
        } else {
            if regex.likely_author.is_match(trimmed)
                || regex.name_pair.is_match(trimmed)
                || trimmed.contains('\\')
                || regex.multiple_names.is_match(trimmed) {
                break;
            }
            title_lines.push(trimmed.to_string());
            last_index = i + 1;
        }
    }

    (title_lines.join(" ").trim().to_string(), last_index)
}


/// Nettoie une chaîne de caractères contenant des noms ou affiliations.
fn clean_string(input: &str) -> String {
    let to_remove = ["1", "2", "†", "*", "∗", "\\", "  "];
    let mut cleaned = input.to_string();
    for item in to_remove {
        cleaned = cleaned.replace(item, " ");
    }
    cleaned.trim().to_string()
}

/// Extraite les auteurs de l'article.
fn extract_authors(lines: &[String], start_after_title: usize) -> String {
    let mut author_lines = Vec::new();
    let mut started = false;

    for line in lines.iter().skip(start_after_title) {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        if trimmed.is_empty() {
            continue;
        }

        let breaks_section = [
            "abstract", "university", "laboratoire", "school", "@",
            "technologies", "street", "avenue", "parkway"
        ];

        if breaks_section.iter().any(|kw| lower.contains(kw))
            || trimmed.chars().all(|c| c.is_numeric()) {
            break;
        }

        let contains_names = trimmed
            .split(|c: char| c == ',' || c == '\\' || c == '/' || c == ';')
            .filter(|part| {
                let part = part.trim();
                part.split_whitespace().count() >= 2
                    && part.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            })
            .count();

        if contains_names >= 1 {
            author_lines.push(trimmed.to_string());
            started = true;
        } else if started {
            break;
        }
    }

    clean_string(&author_lines.join(" "))
}

/// Extraite le résumé de l'article.
fn extract_abstract(lines: &[String]) -> String {
    const START_KEYWORDS: &[&str] = &["abstract", "résumé", "summary", "executive summary"];
    const STOP_KEYWORDS: &[&str] = &["introduction", "keywords", "index terms", "1.", "i.", "1 introduction"];

    let mut abstract_lines = Vec::new();
    let mut in_abstract = false;
    let mut empty_line_seen = false;
    let mut fallback: Option<String> = None;

    for line in lines {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Détection explicite
        if !in_abstract && START_KEYWORDS.iter().any(|kw| {
            lower.starts_with(&format!("{kw}—")) ||
                lower.starts_with(&format!("{kw}:")) ||
                lower == *kw
        }) {
            in_abstract = true;

            let mut cleaned = trimmed.to_string();
            for kw in START_KEYWORDS {
                for sep in ["—", ":", "-"] {
                    let prefix = format!("{kw}{sep}");
                    if cleaned.to_lowercase().starts_with(&prefix) {
                        cleaned = cleaned[prefix.len()..].trim_start().to_string();
                        break;
                    }
                }
            }

            if cleaned.split_whitespace().count() > 2 {
                abstract_lines.push(cleaned);
            }

            continue;
        }

        if in_abstract {
            if trimmed.is_empty() {
                empty_line_seen = true;
                continue;
            }

            if empty_line_seen && STOP_KEYWORDS.iter().any(|kw| lower.starts_with(kw)) {
                break;
            }

            if lower.contains("@") || lower.contains("university") || lower.contains("institute") {
                continue;
            }

            abstract_lines.push(trimmed.to_string());
            empty_line_seen = false;
        }

        // fallback pour paragraphe long
        if !in_abstract && fallback.is_none() {
            if trimmed.len() > 100 && !lower.contains("introduction") {
                fallback = Some(trimmed.to_string());
            }
        }
    }

    if !abstract_lines.is_empty() {
        abstract_lines.join(" ").replace("  ", " ").trim().to_string()
    } else {
        fallback.unwrap_or_default()
    }
}

/// Extrait l'introduction à partir des lignes, après le résumé
fn extract_introduction(lines: &[String], abstract_text: &str) -> String {
    use regex::Regex;

    let intro_regex = Regex::new(r"(?i)^(\d+\.?|[ivxlc]+\.?)?\s*introduction\s*$").unwrap();
    let end_regex = Regex::new(r"(?i)(^\d+\.?|^[ivxlc]+\.?)?\s*(related work|background|method|methods|approach|system|architecture|model|design|experiment|results|evaluation|data|training|implementation|algorithm)s?\s*$").unwrap();
    let plan_phrase_regex = Regex::new(r"(?i)the rest of (the )?paper is organized|we (now )?describe the structure of the paper").unwrap();

    let mut intro_lines = Vec::new();
    let mut in_intro = false;
    let mut abstract_end_index = 0;

    // Estime la position de fin de l'abstract dans le texte brut
    if let Some(first_line) = abstract_text.lines().next() {
        for (i, line) in lines.iter().enumerate() {
            if line.contains(first_line.trim()) {
                abstract_end_index = i;
                break;
            }
        }
    }

    for (i, line) in lines.iter().enumerate().skip(abstract_end_index) {
        let trimmed = line.trim();

        if !in_intro {
            if intro_regex.is_match(trimmed) {
                in_intro = true;
                continue;
            }
        } else {
            if trimmed.is_empty() {
                continue;
            }
            if end_regex.is_match(trimmed) || plan_phrase_regex.is_match(trimmed) {
                break;
            }
            intro_lines.push(trimmed.to_string());
        }
    }

    intro_lines.join(" ").replace("  ", " ").trim().to_string()
}


/// Extrait les champs disponibles de l'article.
fn extract_article_fields(path: &Path, regex: &RegexSet) -> io::Result<ArticleData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().filter_map(Result::ok).collect::<Vec<_>>();

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().replace(' ', "_"))
        .unwrap_or_else(|| "unknown_file".to_string());

    let (title, title_end_index) = extract_title(&lines, regex);
    let authors = extract_authors(&lines, title_end_index);
    let abstract_text = extract_abstract(&lines);
    let introduction = extract_introduction(&lines, &abstract_text);

    Ok(ArticleData {
        filename,
        title,
        authors,
        abstract_text,
        introduction,
        body: String::new(),
        conclusion: String::new(),
        discussion: String::new(),
        bibliography: String::new(),
    })
}

/// Écrit les articles en XML.
fn write_combined_xml(path: &Path, articles: &[ArticleData]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "<articles>")?;

    for article in articles {
        writeln!(
            file,
            "\t<article>\n\
            \t\t<preamble>{}</preamble>\n\
            \t\t<titre>{}</titre>\n\
            \t\t<auteur>{}</auteur>\n\
            \t\t<abstract>{}</abstract>\n\
            \t\t<introduction>{}</introduction>\n\
            \t\t<corps>{}</corps>\n\
            \t\t<conclusion>{}</conclusion>\n\
            \t\t<discussion>{}</discussion>\n\
            \t\t<biblio>{}</biblio>\n\
            \t</article>",
            article.filename,
            article.title,
            article.authors,
            article.abstract_text,
            article.introduction,
            article.body,
            article.conclusion,
            article.discussion,
            article.bibliography
        )?;
    }

    writeln!(file, "</articles>")?;
    Ok(())
}

/// Écrit les résumés textuels.
fn write_txt_summaries(path: &Path, articles: &[ArticleData], duration_total: u128) -> io::Result<()> {
    let mut file = File::create(path)?;

    for article in articles {
        let total_len = article.abstract_text.len()
            + article.introduction.len()
            + article.body.len()
            + article.discussion.len()
            + article.conclusion.len();

        writeln!(
            file,
            "==============================\n\
             Fichier        : {}\n\
             Titre          : {}\n\
             Auteurs        : {}\n\
             Résumé         : {}\n\
             Introduction   : {}\n\
             Développement  : {}\n\
             Discussion     : {}\n\
             Conclusion     : {}\n\
             Références     : {}\n\
             Longueur texte : {} caractères\n",
            article.filename,
            article.title,
            article.authors,
            article.abstract_text,
            article.introduction,
            article.body,
            article.discussion,
            article.conclusion,
            article.bibliography,
            total_len
        )?;
    }

    writeln!(file, "==============================")?;
    writeln!(file, "Traitement terminé en {} ms", duration_total)?;
    Ok(())
}

/// Fonction principale.
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_folder> <output_folder> <mode: txt|xml>", args[0]);
        std::process::exit(1);
    }

    let input_folder = Path::new(&args[1]);
    let output_folder = Path::new(&args[2]);
    let mode = &args[3].to_lowercase();
    fs::create_dir_all(output_folder)?;

    let start_all = Instant::now();
    let regex = RegexSet::new();

    let entries: Vec<_> = fs::read_dir(input_folder)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("txt"))
        .collect();

    let articles: Vec<_> = entries
        .par_iter()
        .filter_map(|path| extract_article_fields(path, &regex).ok())
        .collect();

    match mode.as_str() {
        "xml" => write_combined_xml(&output_folder.join("articles.xml"), &articles)?,
        "txt" => {
            let elapsed = start_all.elapsed().as_millis();
            write_txt_summaries(&output_folder.join("resumes.txt"), &articles, elapsed)?;
        }
        _ => {
            eprintln!("Mode invalide : {}. Utilisez 'txt' ou 'xml'.", mode);
            std::process::exit(1);
        }
    }

    println!(
        "Extraction réussie en mode {}. Temps total : {} ms",
        mode,
        start_all.elapsed().as_millis()
    );

    Ok(())
}
