#!/usr/bin/env python3
import tkinter as tk
from tkinter import filedialog, messagebox
import subprocess
import os
import shutil

class PDFConverterGUI(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Mini PDF Converter")
        self.geometry("500x350")
        self.configure(bg="#f0f0f0")

        self.pdf_path = None

        title_label = tk.Label(
            self,
            text="Convertisseur PDF vers TXT",
            font=("Helvetica", 16, "bold"),
            bg="#f0f0f0",
            fg="#333"
        )
        title_label.pack(pady=15)

        instruct_label = tk.Label(
            self,
            text="Glissez/déposez ou cliquez sur 'Select PDF'",
            font=("Helvetica", 12),
            bg="#f0f0f0",
            fg="#555"
        )
        instruct_label.pack(pady=10)

        self.file_label = tk.Label(
            self,
            text="Aucun fichier sélectionné",
            font=("Helvetica", 12),
            bg="#f0f0f0",
            fg="#555"
        )
        self.file_label.pack(pady=5)

        select_btn = tk.Button(
            self,
            text="Select PDF",
            command=self.select_file,
            font=("Helvetica", 12),
            bg="#4CAF50",
            fg="white",
            padx=10,
            pady=5
        )
        select_btn.pack(pady=10)

        self.convert_btn = tk.Button(
            self,
            text="Convert",
            command=self.convert_pdf,
            font=("Helvetica", 12),
            bg="#2196F3",
            fg="white",
            padx=10,
            pady=5
        )
        self.download_btn = tk.Button(
            self,
            text="Download TXT",
            command=self.download_txt,
            font=("Helvetica", 12),
            bg="#FF5722",
            fg="white",
            padx=10,
            pady=5
        )

    def select_file(self):
        file_path = filedialog.askopenfilename(
            title="Sélectionnez un fichier PDF",
            filetypes=[("PDF Files", "*.pdf")]
        )
        if file_path:
            self.pdf_path = file_path
            self.file_label.config(text=f"Fichier sélectionné : {os.path.basename(file_path)}")
            if not self.convert_btn.winfo_ismapped():
                self.convert_btn.pack(pady=10)

    def convert_pdf(self):
        if not self.pdf_path:
            messagebox.showerror("Erreur", "Aucun fichier PDF sélectionné !")
            return

        try:
            result = subprocess.run(["./convert.bash", self.pdf_path], capture_output=True, text=True)
            if result.returncode == 0:
                messagebox.showinfo("Succès", "Conversion réussie !")
                if not self.download_btn.winfo_ismapped():
                    self.download_btn.pack(pady=10)
            else:
                err_msg = result.stderr if result.stderr else "Conversion échouée sans message d'erreur."
                messagebox.showerror("Erreur", f"Erreur lors de la conversion:\n{err_msg}")
        except Exception as e:
            messagebox.showerror("Exception", f"Une erreur s'est produite :\n{e}")

    def download_txt(self):
        txt_file = os.path.splitext(self.pdf_path)[0] + ".txt"
        if os.path.exists(txt_file):
            save_path = filedialog.asksaveasfilename(
                initialfile=os.path.basename(txt_file),
                defaultextension=".txt",
                filetypes=[("Text Files", "*.txt")]
            )
            if save_path:
                try:
                    shutil.copy(txt_file, save_path)
                    messagebox.showinfo("Téléchargé", f"Fichier sauvegardé à :\n{save_path}")
                except Exception as e:
                    messagebox.showerror("Erreur", f"Erreur lors de la sauvegarde :\n{e}")
        else:
            messagebox.showerror("Erreur", "Fichier converti introuvable !")

if __name__ == "__main__":
    app = PDFConverterGUI()
    app.mainloop()