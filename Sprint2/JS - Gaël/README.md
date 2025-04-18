# 📄 Extraction de métadonnées depuis des PDF

Ce script Node.js permet de traiter un dossier de fichiers PDF pour en extraire automatiquement le **titre** et le **résumé (abstract)**, puis de sauvegarder ces informations dans des fichiers `.txt` au format lisible.

## 🚀 Fonctionnalités

- 🔍 Lecture de fichiers PDF via [`pdf-parse`](https://www.npmjs.com/package/pdf-parse)
- 📁 Traitement automatique de tous les PDF présents dans un dossier source
- 🧠 Extraction simple du **titre** (ligne précédant "abstract") et du **résumé** (ligne suivant "abstract")
- 💾 Génération d’un fichier `.txt` par PDF contenant : nom du fichier, titre et résumé
- ♻️ Nettoyage automatique de l'ancien dossier de sortie

## 🧰 Prérequis

- Node.js (v14+ recommandé)
- Installation des dépendances via npm :

```bash
npm install fs-extra pdf-parse
```

## 📁 Arborescence

```
.
├── script.js             # Le script principal
├── corpus_txt/          # Dossier généré contenant les fichiers .txt
└── ../CORPUS_TRAIN/     # Dossier source contenant les fichiers PDF
```

## ▶️ Utilisation

```bash
node script.js
```

Le script :
1. Supprime l'ancien dossier `corpus_txt` s'il existe
2. Parcourt tous les fichiers `.pdf` dans `../CORPUS_TRAIN`
3. Extrait le titre et le résumé de chaque PDF
4. Crée un fichier `.txt` par PDF dans `corpus_txt/`

## 📌 Remarques

- Le script se base sur la présence du mot **"abstract"** (non sensible à la casse) pour extraire les données.
- Les titres et résumés peuvent ne pas être parfaitement extraits selon la structure du PDF.

## 📎 Exemple de sortie (`exemple.txt`)

```
exemple
Titre du document
Texte du résumé...
```
