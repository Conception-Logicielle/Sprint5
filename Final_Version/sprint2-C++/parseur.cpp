#include <filesystem>
#include <fstream>
#include <iostream>
#include <regex>
#include <sstream>
#include <algorithm>

// Parser d'articles scientifiques en C++
// Utilisation : ./parser <dossier_input_txt>
// Le programme lit chaque fichier .txt (déjà converti depuis PDF),
// extrait le nom de fichier (espaces remplacés par '_'), le titre,
// et le résumé (abstract), puis génère un fichier _parsed.txt
// dans le sous-dossier "parsed_texts" du dossier d'entrée.

int main(int argc, char** argv) {
    using namespace std;
    namespace fs = std::filesystem;

    if (argc != 2) {
        cerr << "Usage: " << argv[0] << " <dossier_input_txt>\n";
        return 1;
    }
    fs::path input_dir = argv[1];
    if (!fs::exists(input_dir) || !fs::is_directory(input_dir)) {
        cerr << "Dossier invalide : " << input_dir << "\n";
        return 1;
    }

    fs::path output_dir = input_dir / "parsed_texts";
    if (fs::exists(output_dir)) {
        fs::remove_all(output_dir);
    }
    fs::create_directory(output_dir);

    for (const auto& entry : fs::directory_iterator(input_dir)) {
        if (!entry.is_regular_file()) continue;
        fs::path txt_path = entry.path();
        if (txt_path.extension() != ".txt") continue;

        // Préparer le nom original (espaces -> '_')
        string original = txt_path.filename().string();
        replace(original.begin(), original.end(), ' ', '_');

        // Lecture du texte pour parsing
        ifstream ifs(txt_path);
        if (!ifs) {
            cerr << "Impossible d'ouvrir le fichier texte : " << txt_path << "\n";
            continue;
        }

        string line;
        string title;
        string abstract;

        // Extraction du titre : première ligne non vide
        while (getline(ifs, line)) {
            if (!line.empty()) {
                title = line;
                break;
            }
        }

        // Extraction de l'abstract
        regex re("^\\s*(Abstract|ABSTRACT)[:\\s]?(.*)", regex::icase);
        smatch m;
        bool in_abs = false;
        ostringstream abs_stream;
        ifs.clear();
        ifs.seekg(0);

        while (getline(ifs, line)) {
            if (!in_abs) {
                if (regex_search(line, m, re)) {
                    in_abs = true;
                    abs_stream << m[2].str();
                }
            } else {
                if (line.empty()) break;
                abs_stream << " " << line;
            }
        }
        abstract = abs_stream.str();

        // Écriture du fichier parsé
        fs::path out_file = output_dir / (txt_path.stem().string() + string("_parsed.txt"));
        ofstream ofs(out_file);
        ofs << original << "\n";
        ofs << title << "\n";
        ofs << abstract << "\n";
        ofs.close();
    }

    cout << "Traitement terminé. Fichiers générés dans : " << output_dir << "\n";
    return 0;
}

