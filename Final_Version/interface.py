#!/usr/bin/env python3
import tkinter as tk
from tkinter import filedialog, messagebox, ttk, scrolledtext
from PIL import Image, ImageTk  # Pillow requis
import subprocess
import threading
import os

class PDFConverterGUI(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("PDF Résumeur")
        self.geometry("920x520")
        self.configure(bg="#1e1e1e")
        self.resizable(False, False)

        self.pub_frame = tk.Frame(self, bg="#1e1e1e", width=160)
        self.pub_frame.pack(side="left", fill="y", padx=(10,5), pady=10)
        self.pub_frame.pack_propagate(False)
        self._init_pub()

        self.main_frame = tk.Frame(self, bg="#1e1e1e")
        self.main_frame.pack(side="left", fill="both", expand=True, padx=5, pady=10)

        self.log_frame = tk.Frame(self, bg="#1e1e1e", width=260)
        self.log_frame.pack(side="left", fill="y", padx=(5,10), pady=10)
        self.log_frame.pack_propagate(False)

        self._init_header()
        self._init_file_section()
        self._init_conversion_section()
        self._init_progress()
        self._init_logs()

        self.pdf_paths = []

    def _init_pub(self):
        tk.Label(self.pub_frame, text="Ta pub ici", bg="#1e1e1e", fg="#ffffff",
                 font=("Helvetica", 10, "italic")).pack(pady=(0,5))
        try:
            img = Image.open("./img/pull_promo.jpg").resize((140, 420), Image.ANTIALIAS)
            self.ad_img = ImageTk.PhotoImage(img)
            tk.Label(self.pub_frame, image=self.ad_img, bg="#1e1e1e").pack()
        except:
            tk.Label(self.pub_frame, text="[Image pub verticale]", bg="#333333", fg="#888888",
                     width=18, height=26, anchor="center").pack()

    def _init_header(self):
        header = tk.Label(self.main_frame, text="📄 PDF Résumeur", font=("Helvetica", 22, "bold"),
                          bg="#1e1e1e", fg="#00ffcc")
        header.pack(pady=(0,20))
        header.bind("<Enter>", lambda e: header.config(fg="#33ffee"))
        header.bind("<Leave>", lambda e: header.config(fg="#00ffcc"))

    def _init_file_section(self):
        section = tk.Frame(self.main_frame, bg="#1e1e1e")
        section.pack(fill="x", padx=10, pady=(0,20))

        tk.Label(section, text="→ 1) Sélection des fichiers PDF", font=("Helvetica", 14, "underline"),
                 bg="#1e1e1e", fg="#ffffff").pack(anchor="w")

        self.file_label = tk.Label(section, text="Aucun fichier sélectionné",
                                   font=("Helvetica", 11), bg="#1e1e1e", fg="#aaaaaa",
                                   wraplength=600, justify="left")
        self.file_label.pack(fill="x", pady=(5,10))

        tk.Button(section, text="📂 Parcourir", command=self.select_files,
                  font=("Helvetica", 11), bg="#4CAF50", fg="white",
                  relief="raised", bd=3, width=12).pack(anchor="w")

    def _init_conversion_section(self):
        section = tk.Frame(self.main_frame, bg="#1e1e1e")
        section.pack(fill="x", padx=10, pady=(0,20))

        tk.Label(section, text="→ 2) Options de conversion", font=("Helvetica", 14, "underline"),
                 bg="#1e1e1e", fg="#ffffff").grid(row=0, column=0, columnspan=2, sticky="w")

        tk.Label(section, text="Mode :", font=("Helvetica", 12),
                 bg="#1e1e1e", fg="#ffffff").grid(row=1, column=0, sticky="e", pady=10)
        self.mode_var = tk.StringVar(value="txt")
        ttk.Combobox(section, textvariable=self.mode_var, values=["txt", "xml"], width=6)\
            .grid(row=1, column=1, sticky="w", padx=(5,0), pady=10)

        self.convert_btn = tk.Button(section, text="⚙️ Convertir & Résumer", command=self.start_conversion,
                                     font=("Helvetica", 12), bg="#2196F3", fg="white",
                                     state="disabled", relief="raised", bd=3, width=20)
        self.convert_btn.grid(row=2, column=0, columnspan=2, pady=(10,0))

    def _init_progress(self):
        self.status_label = tk.Label(self.main_frame, text="", font=("Helvetica", 10),
                                     bg="#1e1e1e", fg="#00ffcc")
        self.progress = ttk.Progressbar(self.main_frame, mode="indeterminate", length=580)

    def _init_logs(self):
        tk.Label(self.log_frame, text="Logs", font=("Helvetica", 12, "bold"),
                 bg="#1e1e1e", fg="#ffffff").pack(pady=(0,5))
        self.output_log = scrolledtext.ScrolledText(self.log_frame, height=28,
                                                    state="disabled", bg="#2d2d2d",
                                                    fg="#ffffff", wrap="word")
        self.output_log.pack(fill="both", expand=True)

    def select_files(self):
        paths = filedialog.askopenfilenames(title="Sélectionne des PDF",
                                            filetypes=[("PDF","*.pdf")])
        if paths:
            self.pdf_paths = list(paths)
            noms = [os.path.basename(p) for p in self.pdf_paths]
            self.file_label.config(text="\n".join(noms))
            self.convert_btn.config(state="normal")

    def start_conversion(self):
        if not self.pdf_paths:
            messagebox.showerror("Erreur", "Aucun PDF sélectionné !")
            return

        self.convert_btn.config(state="disabled")
        self.status_label.config(text="→ Conversion en cours…")
        self.status_label.pack(fill="x", padx=10, pady=(0,5))
        self.progress.pack(fill="x", padx=10, pady=(0,10))
        self.progress.start(10)

        threading.Thread(target=self.run_conversion, daemon=True).start()

    def run_conversion(self):
        args = self.pdf_paths + (["-x"] if self.mode_var.get()=="xml" else [])
        cmd = ["bash", "./main.sh"] + args
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

        self.output_log.config(state="normal")
        self.output_log.delete(1.0, tk.END)

        for line in iter(proc.stdout.readline, ""):
            self.output_log.insert(tk.END, line)
            self.output_log.see(tk.END)

        err = proc.stderr.read()
        if err:
            self.output_log.insert(tk.END, err)
            self.output_log.see(tk.END)

        proc.wait()
        self.progress.stop()
        self.progress.pack_forget()
        self.status_label.pack_forget()
        self.output_log.config(state="disabled")

        if proc.returncode == 0:
            messagebox.showinfo("Succès", "Conversion & résumés terminés ! 🎉")
        else:
            messagebox.showerror("Erreur", "Un souci est survenu, regarde les logs.")

        self.convert_btn.config(state="normal")

if __name__ == "__main__":
    PDFConverterGUI().mainloop()
