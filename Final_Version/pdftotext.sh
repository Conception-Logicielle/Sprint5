#!/bin/bash

if ! command -v pdftotext &>/dev/null; then
    echo "Erreur : pdftotext n'est pas installé. Installez-le avant d'exécuter ce script."
    exit 1
fi

if [ $# -lt 1 ]; then
    echo "Usage : $0 <chemin_du_fichier_ou_du_dossier> [dossier_de_sortie]"
    exit 1
fi

INPUT_PATH="$1"
OUTPUT_DIR="${2:-$(pwd)}"

if [[ -f "$INPUT_PATH" ]]; then
    FILES=("$INPUT_PATH")
elif [[ -d "$INPUT_PATH" ]]; then
    FILES=("$INPUT_PATH"/*.pdf)
else
    echo "Erreur : '$INPUT_PATH' n'est ni un fichier ni un dossier valide."
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

for PDF_FILE in "${FILES[@]}"; do
    if [[ ! -f "$PDF_FILE" ]]; then
        echo "❌ Fichier introuvable : $PDF_FILE"
        continue
    fi

    BASENAME=$(basename "$PDF_FILE" .pdf)
    OUTPUT_FILE="$OUTPUT_DIR/$BASENAME.txt"

    if pdftotext -enc UTF-8 -nopgbrk "$PDF_FILE" "$OUTPUT_FILE"; then
        echo "Conversion réussie : $PDF_FILE → $OUTPUT_FILE"
    else
        echo "Erreur lors de la conversion : $PDF_FILE"
    fi
done