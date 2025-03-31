#!/bin/bash

# Vérifier si le fichier PDF est fourni
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <document_scientifique.pdf>"
    exit 1
fi

PDF_FILE="$1"
TEXT_FILE="extrait_texte.txt"

# Extraire le texte du PDF avec une meilleure mise en page
pdftotext -layout -nopgbrk "$PDF_FILE" "$TEXT_FILE"
