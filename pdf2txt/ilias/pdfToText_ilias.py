import os
import sys
import subprocess
from pdfminer.high_level import extract_text

def convert_pdf_to_text(pdf_file, method="pdf2txt"):
    """Convertit un fichier PDF en texte avec la méthode spécifiée et enregistre le résultat."""
    if not os.path.isfile(pdf_file):
        print(f"Erreur : Le fichier '{pdf_file}' n'existe pas.")
        sys.exit(1)
    
    output_file = os.path.splitext(pdf_file)[0] + f"_{method}.txt"
    
    try:
        if method == "pdf2txt":
            text = extract_text(pdf_file)
        elif method == "pdftotext":
            result = subprocess.run(["pdftotext", pdf_file, "-"], capture_output=True, text=True)
            text = result.stdout
        else:
            print("Erreur : Méthode non reconnue. Utilisez 'pdf2txt' ou 'pdftotext'.")
            sys.exit(1)
        
        if not text.strip():
            print(f"Avertissement : Aucun texte extrait avec {method}. Le PDF pourrait être scanné ou crypté.")
        
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(text)
        
        print(f"Conversion réussie avec {method} ! Fichier texte généré : {output_file}")
    except Exception as e:
        print(f"Erreur lors de la conversion avec {method} : {e}")
        sys.exit(1)

def main():
    if len(sys.argv) != 2:
        print("Usage : python PdfToText.py <fichier.pdf>")
        sys.exit(1)
    
    pdf_file = sys.argv[1]
    convert_pdf_to_text(pdf_file, "pdf2txt")
    convert_pdf_to_text(pdf_file, "pdftotext")
    
    print("\nComparez les fichiers générés pour choisir la meilleure méthode.")

if __name__ == "__main__":
    main()
