#!/usr/bin/env python3

import os
import sys
import re
from pathlib import Path
from pdfminer.high_level import extract_text

# Nettoie le texte brut : enlève les doublons de lignes vides et d'espaces
def nettoyer_texte(texte):
    texte = re.sub(r'\n{2,}', '\n\n', texte)  # garde max 1 ligne vide entre paragraphes
    texte = re.sub(r' {2,}', ' ', texte)      # réduit les espaces multiples
    return texte.strip()

# Met en majuscule les sections repérées (titre, résumé, etc.)
def detecter_sections(texte):
    sections = [
        "abstract", "introduction", "state of the art", "méthode", "method",
        "experiments", "results", "discussion", "conclusion", "references", "bibliography"
    ]
    lignes = texte.splitlines()
    lignes_nettoyées = []
    for ligne in lignes:
        l = ligne.strip()
        if any(re.match(rf'^\s*{s}[ .:–-]*$', l.lower()) for s in sections):
            lignes_nettoyées.append(l.upper())
        else:
            lignes_nettoyées.append(l)
    return "\n".join(lignes_nettoyées)

# Convertit un PDF en .txt dans le dossier de destination
def convertir_pdf_en_txt(pdf_path, output_dir):
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    pdf_path = Path(pdf_path)

    try:
        texte = extract_text(pdf_path, laparams=None)
        texte = nettoyer_texte(texte)
        texte = detecter_sections(texte)

        txt_path = output_dir / (pdf_path.stem + ".txt")
        with open(txt_path, "w", encoding="utf-8") as f:
            f.write(texte)
        print(f"[OK] Converti : {pdf_path.name} --> {txt_path.name}")
    except Exception as e:
        print(f"[ERR] Erreur avec {pdf_path.name} : {e}", file=sys.stderr)
        sys.exit(1)

# Point d'entrée du script
if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage : pdfminer_six.py <fichier.pdf> <dossier_de_sortie>")
        sys.exit(1)

    pdf_file = sys.argv[1]
    output_folder = sys.argv[2]

    if not os.path.isfile(pdf_file):
        print(f"❌ Le fichier spécifié n'existe pas : {pdf_file}")
        sys.exit(1)

    convertir_pdf_en_txt(pdf_file, output_folder)
