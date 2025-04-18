import os
import time

# Fonction pour analyser le texte et extraire le titre et le résumé
def extract_title_and_abstract(text):
    title = None
    abstract = None
    
    # Extraction du titre (première ligne)
    lines = text.split('\n')
    if len(lines) > 0:
        title = lines[0]  # Le titre est généralement la première ligne
    
    for i, line in enumerate(lines):
        if 'abstract' in line.lower():  # Recherche du mot "abstract"
            abstract_lines = []
            # On passe à la ligne suivante
            j = i + 1
            while j < len(lines):
                line_content = lines[j]
                #print(f"Ligne {j}: {line_content}")  # Affichage pour déboguer
                
                # Compter le nombre d'espaces au début de la ligne
                leading_spaces = len(line_content) - len(line_content.lstrip(' '))
                
                # Si la ligne contient plus de 5 espaces, on arrête l'extraction (sauf si c'est la première ligne après "abstract")
                # Donc si l'abstract n'est pas dutout collé a gauche ca le passera
                # Mais ceci permet de ne pas recuperer le double colonne
                # Et de  bien clotuer l'abstract
                if leading_spaces > 5 and len(abstract_lines) > 0:
                    print(f"End of abstract at line {j} (more than 5 spaces).")
                    break
                
                # Si la ligne est vide, on arrête l'extraction (sauf premiere ligne)
                elif len(line_content.strip()) == 0 and j != i + 1:
                    print(f"End of abstract at line {j} (empty line).")
                    break
                
                # Si la ligne contient plus de 5 espaces au début, on passe à la suivante
                elif leading_spaces > 5:
                    print(f"Skipping line {j} (more than 5 spaces).")
                else:
                    # Si la ligne contient moins de 5 espaces au début, on la prend en compte
                    # Maintenant on vérifie s'il y a des long espaces dans la ligne
                    if line_content.count(' ') > 3:
                        # Si la ligne contient des long espaces à l'intérieur, on garde uniquement la partie avant ces espaces
                        first_part = line_content.split('      ')[0]  # Récupérer ce qu'il y a avant les long espaces
                        abstract_lines.append(first_part.strip())
                    else:
                        # Sinon, on garde la ligne entière
                        abstract_lines.append(line_content.strip())
                
                # Passer à la ligne suivante
                j += 1

            # Joindre toutes les lignes d'abstract pour les mettre en une seule ligne
            abstract = ' '.join(abstract_lines)
            break
    
    # Si aucun abstract n'est trouvé, on donne un message par défaut
    if not abstract:
        abstract = "Aucun résumé (abstract) trouvé."
    
    return title, abstract

# Dossier contenant les fichiers texte
input_folder = '../corpus_txt'
output_folder = './test'  # Le dossier de sortie

# Liste des fichiers dans le dossier d'entrée
for filename in os.listdir(input_folder):
    if filename.endswith('.txt'):
        start_time = time.time()
        file_path = os.path.join(input_folder, filename)
        with open(file_path, 'r', encoding='utf-8') as file:
            content = file.read()

        # Extraire le titre et l'abstract
        title, abstract = extract_title_and_abstract(content)

        # Créer le fichier de sortie dans le dossier 'test' avec les informations extraites
        if not os.path.exists(output_folder):
            os.makedirs(output_folder)  # Créer le dossier 'test' si il n'existe pas déjà

        output_file = os.path.join(output_folder, filename.replace('.txt', '_processed.txt'))
        with open(output_file, 'w', encoding='utf-8') as out_file:
            out_file.write(f"{filename}\n")
            out_file.write(f"{title}\n")
            out_file.write(f"{abstract}\n")

        end_time = time.time()
        elapsed_time = end_time - start_time
        print(f"✅ Traitement du fichier {filename} terminé en {elapsed_time:.5f} secondes.")
