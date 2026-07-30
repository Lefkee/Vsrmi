# Örnek Python dosyası
import os
import sys

class Hayvan:
    """Hayvan sınıfı"""

    def __init__(self, isim: str, yas: int):
        self.isim = isim
        self.yas = yas

    def ses_cikar(self) -> str:
        return "..."

class Kedi(Hayvan):
    SAYI = 4  # ayak sayısı

    def ses_cikar(self) -> str:
        return "Miyav!"

def main():
    kediler = [Kedi("Minnoş", 3), Kedi("Pamuk", 5)]
    for kedi in kediler:
        print(f"{kedi.isim}: {kedi.ses_cikar()}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
