import re

def extract_types_from_file(input_file, output_file):
    """Extracts types between angle brackets from a file and saves them in a new file, enclosing them in single quotes, avoiding duplicates."""
    types = set()  # Utilizza un set per evitare duplicati
    with open(input_file, "r") as file:
        lines = file.readlines()
        for line in lines:
            # Trova il contenuto tra le parentesi angolari
            match = re.search(r'<(.*?)>', line)
            if match:
                # Aggiunge il contenuto estratto al set, racchiudendolo tra apici singoli
                types.add(f"'{match.group(1)}'")

    # Salva i risultati nel file di output
    with open(output_file, "w") as file:
        for item in sorted(types):  # Ordina i tipi prima di scriverli
            file.write(f"{item}\n")

    print(f"Types extracted and saved in {output_file}.")

if __name__ == "__main__":
    extract_types_from_file("types_output.txt", "tutti_i_type_coin.txt")
