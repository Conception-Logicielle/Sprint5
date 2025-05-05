#!/bin/bash

# Vérifie si pdftotext est installé
if ! command -v pdftotext &>/dev/null; then
    echo "Erreur : pdftotext n'est pas installé. Installez-le avant d'exécuter ce script."
    exit 1
fi

# Vérifie les arguments
if [ $# -lt 1 ]; then
    echo "Usage : $0 <chemin_du_fichier_ou_du_dossier> [dossier_de_sortie]"
    exit 1
fi

# Variables
INPUT_PATH="$1"
OUTPUT_DIR="${2:-$(pwd)}"

# Vérifie si l'entrée est un fichier ou un dossier
if [[ -f "$INPUT_PATH" ]]; then
    # Si c'est un fichier unique
    FILES=("$INPUT_PATH")
elif [[ -d "$INPUT_PATH" ]]; then
    # Si c'est un dossier, on récupère tous les fichiers PDF
    FILES=("$INPUT_PATH"/*.pdf)
else
    echo "Erreur : '$INPUT_PATH' n'est ni un fichier ni un dossier valide."
    exit 1
fi

# Vérifie si le dossier de sortie existe, sinon le créer
mkdir -p "$OUTPUT_DIR"

# Boucle sur chaque fichier PDF trouvé
for PDF_FILE in "${FILES[@]}"; do
    # Vérifie si le fichier existe et est bien un PDF
    if [[ ! -f "$PDF_FILE" ]]; then
        echo "❌ Fichier introuvable : $PDF_FILE"
        continue
    fi

    BASENAME=$(basename "$PDF_FILE" .pdf)
    OUTPUT_FILE="$OUTPUT_DIR/$BASENAME.txt"

    # Conversion avec options optimisées
    if pdftotext -enc UTF-8 -nopgbrk "$PDF_FILE" "$OUTPUT_FILE"; then
        echo "✅ Conversion réussie : $PDF_FILE → $OUTPUT_FILE"
    else
        echo "❌ Erreur lors de la conversion : $PDF_FILE"
    fi
done