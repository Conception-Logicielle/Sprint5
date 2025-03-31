#!/bin/bash

if [ -z "$1" ] || [ ! -f "$1" ]; then
  echo "❌ Error : Fichier PDF manquant ou introuvable, mon gâté !" >&2
  exit 1
fi

pdftotext -layout -enc UTF-8 -nopgbrk -eol unix "$1" "${1%.pdf}.txt"

if [ $? -eq 0 ]; then
  echo "✅ Conversion réussie : ${1%.pdf}.txt créé (caractères spéciaux préservés)."
else
  echo "❌ Erreur lors de la conversion."
fi
