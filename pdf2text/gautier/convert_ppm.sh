#!/bin/bash

if [ -z "$1" ] || [ ! -f "$1" ]; then
  echo "❌ Error : Fichier PDF manquant ou introuvable, mon gâté !" >&2
  exit 1
fi

pdftoppm -png -r 300 "$1" "${1%.pdf}_page"

output_txt="${1%.pdf}_OCR.txt"
> "$output_txt"

for img in ${1%.pdf}_page-*.png; do
  echo "🕒 OCR en cours sur $(basename "$img")..."
  tesseract "$img" stdout -l fra >> "$output_txt"
done

echo "✅ Conversion OCR réussie : texte extrait dans $output_txt"