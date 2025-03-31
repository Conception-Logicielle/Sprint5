## 📚 Parseur d'Articles Scientifiques en Texte

Ce projet convertit automatiquement des articles scientifiques au format PDF en texte brut.

---

### 🧰 Prérequis

il est necessaire d’avoir :

- Un environnement **Linux/WSL** avec `bash`  
- **Poppler-utils** (pour utiliser `pdftotext`)  
  Installe-le si nécessaire :
  ```bash
  sudo apt update
  sudo apt install poppler-utils
  ```

---

### 📁 Structure

```
- CORPUS_TRAIN   # Dossier contenant les fichiers PDF à traiter
- Final_Version
  ├── main.sh               # Script principal de conversion + mise en forme
  └── corpus_txt/             # Sortie texte générée automatiquement
```

> 📌 Les fichiers PDF doivent être placés dans `../CORPUS_TRAIN`  
> Le script générera un `.txt` par PDF dans `./corpus_txt`

---

### 🚀 Lancer le script

```bash
chmod +x quentin.sh
./main.sh
```