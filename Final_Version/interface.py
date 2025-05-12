#!/usr/bin/env python3
import tkinter as tk
from tkinter import filedialog, messagebox, ttk
import subprocess
import threading
import os

class PDFConverterGUI(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("PDF Converter")
        self.geometry("500x300")
        self.configure(bg="#f0f0f0")

        self.pdf_paths = []

        tk.Label(self, text="Convertisseur PDF vers TXT", font=("Helvetica", 16, "bold"), bg="#f0f0f0").pack(pady=10)
        tk.Label(self, text="Sélectionne un ou plusieurs fichiers PDF", font=("Helvetica", 12), bg="#f0f0f0").pack()

        self.file_label = tk.Label(self, text="Aucun fichier sélectionné", font=("Helvetica", 12), bg="#f0f0f0")
        self.file_label.pack(pady=5)

        btn_frame = tk.Frame(self, bg="#f0f0f0")
        btn_frame.pack(pady=10)

        tk.Button(btn_frame, text="Select PDF(s)", command=self.select_files, font=("Helvetica", 12), bg="#4CAF50", fg="white").grid(row=0, column=0, padx=5)
        self.convert_btn = tk.Button(btn_frame, text="Convert", command=self.start_conversion, font=("Helvetica", 12), bg="#2196F3", fg="white", state="disabled")
        self.convert_btn.grid(row=0, column=1, padx=5)

        self.progress = ttk.Progressbar(self, mode="indeterminate")

    def select_files(self):
        paths = filedialog.askopenfilenames(title="Sélectionnez des fichiers PDF", filetypes=[("PDF Files", "*.pdf")])
        if paths:
            self.pdf_paths = list(paths)
            names = [os.path.basename(p) for p in self.pdf_paths]
            self.file_label.config(text="Sélectionné: " + ", ".join(names))
            self.convert_btn.config(state="normal")

    def start_conversion(self):
        if not self.pdf_paths:
            messagebox.showerror("Erreur", "Aucun fichier PDF sélectionné !")
            return

        self.convert_btn.config(state="disabled")
        self.progress.pack(fill='x', padx=20, pady=10)
        self.progress.start(10)
        thread = threading.Thread(target=self.run_conversion)
        thread.start()

    def run_conversion(self):
        try:
            cmd = ["bash", "./main.sh"] + self.pdf_paths
            result = subprocess.run(cmd, capture_output=True, text=True)
            self.progress.stop()
            self.progress.pack_forget()

            if result.returncode == 0:
                self.animate_success()
            else:
                err = result.stderr or result.stdout
                messagebox.showerror("Erreur", f"Conversion échouée :\n{err}")
                self.convert_btn.config(state="normal")
        except Exception as e:
            self.progress.stop()
            self.progress.pack_forget()
            messagebox.showerror("Exception", f"Une erreur s'est produite :\n{e}")
            self.convert_btn.config(state="normal")

    def animate_success(self):
        messagebox.showinfo("Succès", "Tous les fichiers ont été convertis et résumés générés ! 🎉")
        self.convert_btn.config(state="normal")

if __name__ == "__main__":
    app = PDFConverterGUI()
    app.mainloop()
