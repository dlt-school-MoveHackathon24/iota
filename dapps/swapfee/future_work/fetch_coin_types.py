import subprocess
import re

def execute_command(command):
    """Esegue un comando shell e restituisce l'output."""
    try:
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, shell=True)
        return result.stdout, result.stderr
    except Exception as e:
        print(f"Errore durante l'esecuzione del comando: {e}")
        return None, str(e)

def extract_object_ids(filename):
    """Estrae gli objectId dal file dati in base all'objectType."""
    object_ids = []
    with open(filename, "r") as file:
        lines = file.readlines()
        for i in range(len(lines)):
            # Cerca "objectType" nella riga corrente
            if "objectType" in lines[i] and "::coin::Coin" in lines[i]:
                # Controlla che ci siano abbastanza righe sopra
                if i >= 3:
                    object_id_line = lines[i - 3]  # La riga che contiene l'objectId
                    match = re.search(r'│\s+objectId\s+│\s+(\w+)', object_id_line)
                    if match:
                        object_id = match.group(1).strip()
                        object_ids.append(object_id)
    return object_ids

def fetch_types_for_objects(object_ids):
    """It executes the command for each objectId and saves the type lines in a file."""
    types = []
    for object_id in object_ids:
        command_object_details = f"iota client object \"{object_id}\""
        output_detail, error_detail = execute_command(command_object_details)

        if error_detail:
            print(f"Error while obtaining object details {object_id}: {error_detail}")
            continue

        # Trova la riga "type" e salvala
        for line in output_detail.splitlines():
            if "type" in line:
                types.append(line.strip())

    # Salva i risultati in un file
    with open("types_output.txt", "w") as file:
        for item in types:
            file.write(f"{item}\n")

    print("Types saved in types_output.txt.")

if __name__ == "__main__":
    # Fase 1: Estrai gli oggetti
    object_ids = extract_object_ids("tutti_coin.txt")
    
    # Fase 2: Recupera i tipi
    fetch_types_for_objects(object_ids)
