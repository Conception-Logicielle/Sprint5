#!/bin/bash

# Vérifie si pdf2txt est installé
if ! command -v pdf2txt.py &> /dev/null; then
    echo "Erreur : pdf2txt.py n'est pas installé. Installez-le avec 'pip install pdfminer.six'."
    exit 1
fi

# Vérifie si un fichier PDF est fourni en argument
if [ "$#" -ne 1 ]; then
    echo "Usage : $0 fichier.pdf"
    exit 1
fi

PDF_FILE="$1"

# Vérifie si le fichier PDF existe
if [ ! -f "$PDF_FILE" ]; then
    echo "Erreur : Le fichier $PDF_FILE n'existe pas."
    exit 1
fi

# Convertit le PDF en texte
OUTPUT_FILE="${PDF_FILE%.pdf}.txt"
pdf2txt.py "$PDF_FILE" > "$OUTPUT_FILE"

if [ $? -eq 0 ]; then
    echo "Conversion réussie ! Le fichier texte est : $OUTPUT_FILE"
else
    echo "Erreur lors de la conversion."
    exit 1
fi
